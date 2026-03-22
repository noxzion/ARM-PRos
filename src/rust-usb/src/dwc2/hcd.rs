//! DWC2 Host Controller Driver — init, port control, and transfer API.

use super::regs::*;
use super::channel;
use crate::log;

pub struct Dwc2Hcd {
    base: usize,
    port_speed: u32, // 0=HS, 1=FS, 2=LS
}

impl Dwc2Hcd {
    pub fn new(base: usize) -> Self {
        Self { base, port_speed: 1 } // default FS
    }

    #[inline]
    unsafe fn reg(&self, offset: usize) -> u32 {
        read32(self.base + offset)
    }

    #[inline]
    unsafe fn set_reg(&self, offset: usize, val: u32) {
        write32(self.base + offset, val);
    }

    pub fn read_snpsid(&self) -> u32 {
        unsafe { self.reg(GSNPSID) }
    }

    pub fn delay_ms(&self, ms: u32) {
        for _ in 0..ms {
            for _ in 0..10_000u32 {
                unsafe { core::arch::asm!("nop"); }
            }
        }
    }

    pub fn core_reset(&mut self) {
        log("[DWC2-RS] Core reset...\n\r");
        unsafe {
            // Wait AHB idle
            let mut timeout: u32 = 1_000_000;
            while self.reg(GRSTCTL) & GRSTCTL_AHBIDLE == 0 {
                timeout -= 1;
                if timeout == 0 {
                    log("[DWC2-RS] ERROR: AHB not idle!\n\r");
                    return;
                }
            }

            // Core soft reset
            let v = self.reg(GRSTCTL) | GRSTCTL_CSRST;
            self.set_reg(GRSTCTL, v);

            timeout = 1_000_000;
            while self.reg(GRSTCTL) & GRSTCTL_CSRST != 0 {
                timeout -= 1;
                if timeout == 0 {
                    log("[DWC2-RS] ERROR: Core reset timeout!\n\r");
                    return;
                }
            }
        }
        self.delay_ms(100);
        log("[DWC2-RS] Core reset complete\n\r");
    }

    pub fn phy_init(&mut self) {
        log("[DWC2-RS] Initializing PHY...\n\r");
        unsafe {
            let mut gccfg = self.reg(GCCFG);
            gccfg &= !GCCFG_PWRDOWN;
            gccfg |= GCCFG_VBUSVLD | GCCFG_BVALID;
            self.set_reg(GCCFG, gccfg);
        }
        self.delay_ms(50);
        log("[DWC2-RS] PHY initialized\n\r");
    }

    pub fn host_init(&mut self) {
        log("[DWC2-RS] Initializing host mode...\n\r");
        unsafe {
            // Force host mode
            let mut gusbcfg = self.reg(GUSBCFG);
            gusbcfg |= GUSBCFG_FHMOD;
            gusbcfg &= !GUSBCFG_FDMOD;
            self.set_reg(GUSBCFG, gusbcfg);
            self.delay_ms(50);

            // Configure FIFO sizes
            self.set_reg(GRXFSIZ, 256);         // 1024 bytes RX FIFO
            self.set_reg(GNPTXFSIZ, (256 << 16) | 256); // NP TX FIFO: size=256, offset=256

            // HCFG — clock select
            let mut hcfg = self.reg(HCFG);
            hcfg &= !0x3; // clear FSLSPCLKSEL
            self.set_reg(HCFG, hcfg);

            // Clear all pending interrupts
            self.set_reg(GINTSTS, 0xFFFF_FFFF);

            // Enable host channel interrupt
            self.set_reg(GINTMSK,
                (1 << 24) | // HCINT
                (1 << 25)   // PRTINT
            );

            // Enable global interrupts
            let gahb = self.reg(GAHBCFG) | GAHBCFG_GLBLINTRMSK;
            self.set_reg(GAHBCFG, gahb);

            let mut hprt = self.reg(HPRT0);
            hprt &= !HPRT0_W1C_MASK;
            hprt |= HPRT0_PRTPWR;
            self.set_reg(HPRT0, hprt);
        }
        self.delay_ms(50);
        log("[DWC2-RS] Host mode initialized\n\r");
    }

    pub fn port_reset(&mut self) {
        log("[DWC2-RS] Port reset...\n\r");
        unsafe {
            // Assert reset
            let mut hprt = self.reg(HPRT0);
            hprt &= !HPRT0_W1C_MASK;
            hprt |= HPRT0_PRTRST;
            self.set_reg(HPRT0, hprt);

            self.delay_ms(60); // USB spec: 10-50ms reset

            // Deassert reset
            hprt = self.reg(HPRT0);
            hprt &= !HPRT0_W1C_MASK;
            hprt &= !HPRT0_PRTRST;
            self.set_reg(HPRT0, hprt);

            self.delay_ms(20);

            let hprt = self.reg(HPRT0);
            self.port_speed = (hprt & HPRT0_PRTSPD_MASK) >> 17;

            let speed_str = match self.port_speed {
                0 => "High Speed\n\r",
                1 => "Full Speed\n\r",
                2 => "Low Speed\n\r",
                _ => "Unknown\n\r",
            };
            log("[DWC2-RS] Port speed: ");
            log(speed_str);
        }
        log("[DWC2-RS] Port reset complete\n\r");
    }

    pub fn port_connected(&self) -> bool {
        unsafe { self.reg(HPRT0) & HPRT0_PRTCONNSTS != 0 }
    }

    pub fn is_low_speed(&self) -> bool {
        self.port_speed == 2
    }

    /// Perform a SETUP + optional DATA IN + STATUS control transfer.
    pub fn control_transfer_in(
        &mut self,
        dev_addr: u8,
        setup_data: &[u8; 8],
        data_buf: &mut [u8],
    ) -> bool {
        unsafe {
            let ch = channel::CH_CONTROL;
            let ls = self.is_low_speed();

            channel::configure_channel(
                self.base, ch, dev_addr, 0, false,
                EPTYPE_CONTROL, 64, ls,
            );

            let mut setup_aligned = [0u8; 8];
            setup_aligned.copy_from_slice(setup_data);
            if !channel::do_transfer(
                self.base, ch, HCTSIZ_PID_SETUP,
                setup_aligned.as_mut_ptr(), 8,
            ) {
                log("[DWC2-RS] SETUP phase failed\n\r");
                return false;
            }

            if !data_buf.is_empty() {
                channel::configure_channel(
                    self.base, ch, dev_addr, 0, true,
                    EPTYPE_CONTROL, 64, ls,
                );

                if !channel::do_transfer(
                    self.base, ch, HCTSIZ_PID_DATA1,
                    data_buf.as_mut_ptr(), data_buf.len(),
                ) {
                    log("[DWC2-RS] DATA IN phase failed\n\r");
                    return false;
                }
            }

            channel::configure_channel(
                self.base, ch, dev_addr, 0, false,
                EPTYPE_CONTROL, 64, ls,
            );

            if !channel::do_transfer(
                self.base, ch, HCTSIZ_PID_DATA1,
                core::ptr::null_mut(), 0,
            ) {
                log("[DWC2-RS] STATUS phase failed\n\r");
                return false;
            }

            true
        }
    }

    /// Perform a SETUP + STATUS OUT control transfer (no data phase).
    pub fn control_transfer_out_nodata(
        &mut self,
        dev_addr: u8,
        setup_data: &[u8; 8],
    ) -> bool {
        unsafe {
            let ch = channel::CH_CONTROL;
            let ls = self.is_low_speed();

            channel::configure_channel(
                self.base, ch, dev_addr, 0, false,
                EPTYPE_CONTROL, 64, ls,
            );

            let mut setup_aligned = [0u8; 8];
            setup_aligned.copy_from_slice(setup_data);
            if !channel::do_transfer(
                self.base, ch, HCTSIZ_PID_SETUP,
                setup_aligned.as_mut_ptr(), 8,
            ) {
                log("[DWC2-RS] SETUP phase failed\n\r");
                return false;
            }

            channel::configure_channel(
                self.base, ch, dev_addr, 0, true,
                EPTYPE_CONTROL, 64, ls,
            );

            if !channel::do_transfer(
                self.base, ch, HCTSIZ_PID_DATA1,
                core::ptr::null_mut(), 0,
            ) {
                log("[DWC2-RS] STATUS phase failed\n\r");
                return false;
            }

            true
        }
    }

    /// Perform an interrupt IN transfer (for HID polling).
    pub fn interrupt_transfer_in(
        &mut self,
        dev_addr: u8,
        ep_num: u8,
        data_buf: &mut [u8],
        toggle: &mut bool,
    ) -> bool {
        unsafe {
            let ch = channel::CH_INTERRUPT;
            let ls = self.is_low_speed();

            channel::configure_channel(
                self.base, ch, dev_addr, ep_num, true,
                EPTYPE_INTERRUPT, 8, ls, // HID keyboard MPS = 8
            );

            let pid = if *toggle { HCTSIZ_PID_DATA1 } else { HCTSIZ_PID_DATA0 };

            let ok = channel::do_transfer(
                self.base, ch, pid,
                data_buf.as_mut_ptr(), data_buf.len(),
            );

            if ok {
                *toggle = !*toggle;
            }

            ok
        }
    }
}
