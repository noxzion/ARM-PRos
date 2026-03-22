#![allow(dead_code)]
//! DWC2 (Synopsys DesignWare USB 2.0 OTG) register definitions
//! for Raspberry Pi 3 at base 0x3F980000.

use core::ptr;

#[inline(always)]
pub unsafe fn read32(addr: usize) -> u32 {
    ptr::read_volatile(addr as *const u32)
}

#[inline(always)]
pub unsafe fn write32(addr: usize, val: u32) {
    ptr::write_volatile(addr as *mut u32, val);
}

// Global register offsets
pub const GOTGCTL: usize   = 0x000;
pub const GOTGINT: usize   = 0x004;
pub const GAHBCFG: usize   = 0x008;
pub const GUSBCFG: usize   = 0x00C;
pub const GRSTCTL: usize   = 0x010;
pub const GINTSTS: usize   = 0x014;
pub const GINTMSK: usize   = 0x018;
pub const GRXSTSR: usize   = 0x01C;
pub const GRXFSIZ: usize   = 0x024;
pub const GNPTXFSIZ: usize = 0x028;
pub const GNPTXSTS: usize  = 0x02C;
pub const GCCFG: usize     = 0x100;
pub const GSNPSID: usize   = 0x140;

// Host register offsets
pub const HCFG: usize      = 0x400;
pub const HFIR: usize      = 0x404;
pub const HFNUM: usize     = 0x408;
pub const HPTXSTS: usize   = 0x410;
pub const HAINT: usize     = 0x414;
pub const HAINTMSK: usize  = 0x418;
pub const HPRT0: usize     = 0x440;

// Host channel register offsets (per channel, stride = 0x20)
pub const HCCHAR: usize    = 0x500;
pub const HCSPLT: usize    = 0x504;
pub const HCINT: usize     = 0x508;
pub const HCINTMSK: usize  = 0x50C;
pub const HCTSIZ: usize    = 0x510;
pub const HCDMA: usize     = 0x514;

pub const CHANNEL_STRIDE: usize = 0x20;

// GRSTCTL bits
pub const GRSTCTL_CSRST: u32    = 1 << 0;
pub const GRSTCTL_AHBIDLE: u32  = 1 << 31;

// GAHBCFG bits
pub const GAHBCFG_GLBLINTRMSK: u32 = 1 << 0;
pub const GAHBCFG_DMAEN: u32       = 1 << 5;

// GUSBCFG bits
pub const GUSBCFG_FHMOD: u32  = 1 << 29;
pub const GUSBCFG_FDMOD: u32  = 1 << 30;
pub const GUSBCFG_PHYSEL: u32 = 1 << 6;

// GCCFG bits
pub const GCCFG_PWRDOWN: u32 = 1 << 16;
pub const GCCFG_VBUSVLD: u32 = 1 << 21;
pub const GCCFG_BVALID: u32  = 1 << 23;

// HPRT0 bits
pub const HPRT0_PRTCONNSTS: u32   = 1 << 0;
pub const HPRT0_PRTCONNDET: u32   = 1 << 1;
pub const HPRT0_PRTENA: u32       = 1 << 2;
pub const HPRT0_PRTENCHG: u32     = 1 << 3;
pub const HPRT0_PRTRST: u32       = 1 << 8;
pub const HPRT0_PRTPWR: u32       = 1 << 12;
pub const HPRT0_PRTSPD_MASK: u32  = 0x3 << 17;
pub const HPRT0_PRTSPD_HS: u32    = 0 << 17;
pub const HPRT0_PRTSPD_FS: u32    = 1 << 17;
pub const HPRT0_PRTSPD_LS: u32    = 2 << 17;

// W1C bits in HPRT0 — must NOT be written back as 1 accidentally
pub const HPRT0_W1C_MASK: u32 = HPRT0_PRTCONNDET | HPRT0_PRTENA | HPRT0_PRTENCHG | (1 << 5);

// HCCHAR bits
pub const HCCHAR_MPS_MASK: u32     = 0x7FF;
pub const HCCHAR_EPNUM_SHIFT: u32  = 11;
pub const HCCHAR_EPDIR_IN: u32     = 1 << 15;
pub const HCCHAR_LSDEV: u32        = 1 << 17;
pub const HCCHAR_EPTYPE_SHIFT: u32 = 18;
pub const HCCHAR_MC_SHIFT: u32     = 20;
pub const HCCHAR_DEVADDR_SHIFT: u32 = 22;
pub const HCCHAR_CHEN: u32         = 1 << 31;
pub const HCCHAR_CHDIS: u32        = 1 << 30;

// Endpoint types
pub const EPTYPE_CONTROL: u32    = 0;
pub const EPTYPE_ISOCHRONOUS: u32 = 1;
pub const EPTYPE_BULK: u32       = 2;
pub const EPTYPE_INTERRUPT: u32  = 3;

// HCTSIZ bits
pub const HCTSIZ_XFERSIZE_MASK: u32 = 0x7FFFF;
pub const HCTSIZ_PKTCNT_SHIFT: u32  = 19;
pub const HCTSIZ_PID_SHIFT: u32     = 29;
pub const HCTSIZ_PID_DATA0: u32     = 0 << 29;
pub const HCTSIZ_PID_DATA1: u32     = 2 << 29;
pub const HCTSIZ_PID_SETUP: u32     = 3 << 29;

// HCINT bits
pub const HCINT_XFERCOMP: u32  = 1 << 0;
pub const HCINT_CHHLTD: u32    = 1 << 1;
pub const HCINT_AHBERR: u32    = 1 << 2;
pub const HCINT_STALL: u32     = 1 << 3;
pub const HCINT_NAK: u32       = 1 << 4;
pub const HCINT_ACK: u32       = 1 << 5;
pub const HCINT_XACTERR: u32   = 1 << 7;
pub const HCINT_BBLERR: u32    = 1 << 8;
pub const HCINT_DATATGLERR: u32 = 1 << 10;
