#![no_std]
#![allow(static_mut_refs)]

mod dwc2;
mod usb;
mod hid;

use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, Ordering};

use dwc2::Dwc2Hcd;
use hid::keyboard::Keyboard;

/// Global HCD instance
static mut HCD: Option<Dwc2Hcd> = None;
/// Global keyboard instance
static mut KEYBOARD: Option<Keyboard> = None;
/// Whether keyboard is ready
static KEYBOARD_READY: AtomicBool = AtomicBool::new(false);

// ─── FFI helpers: call into C UART for debug output ───

extern "C" {
    fn uart_puts(s: *const u8);
    fn uart_puthex(val: u64);
}

pub fn log(s: &str) {
    unsafe {
        // We need null-terminated strings for C
        // Use a small stack buffer
        let bytes = s.as_bytes();
        let mut buf = [0u8; 256];
        let len = if bytes.len() < 255 { bytes.len() } else { 255 };
        buf[..len].copy_from_slice(&bytes[..len]);
        buf[len] = 0;
        uart_puts(buf.as_ptr());
    }
}

pub fn log_hex(val: u64) {
    unsafe { uart_puthex(val); }
}

// ─── FFI exports ───

#[no_mangle]
pub extern "C" fn usb_init() {
    log("[DWC2-RS] Starting USB DWC2 initialization...\n\r");

    let mut hcd = Dwc2Hcd::new(0x3F98_0000);

    log("[DWC2-RS] USB base: ");
    log_hex(0x3F98_0000);
    log("\n\r");

    let snpsid = hcd.read_snpsid();
    log("[DWC2-RS] GSNPSID: ");
    log_hex(snpsid as u64);
    log("\n\r");

    if snpsid == 0xFFFF_FFFF || snpsid == 0 {
        log("[DWC2-RS] WARNING: Device ID invalid - USB may not be emulated\n\r");
        log("[DWC2-RS] Continuing anyway...\n\r");
    }

    hcd.core_reset();
    hcd.phy_init();
    hcd.host_init();
    hcd.port_reset();

    let connected = hcd.port_connected();
    log("[DWC2-RS] Port connected: ");
    if connected {
        log("YES\n\r");
    } else {
        log("NO\n\r");
    }

    unsafe { HCD = Some(hcd); }

    log("[DWC2-RS] USB initialization complete\n\r");
}

#[no_mangle]
pub extern "C" fn usb_enumerate_device() {
    log("[DWC2-RS] Starting device enumeration...\n\r");

    let hcd = unsafe {
        match HCD.as_mut() {
            Some(h) => h,
            None => {
                log("[DWC2-RS] ERROR: HCD not initialized\n\r");
                return;
            }
        }
    };

    if !hcd.port_connected() {
        log("[DWC2-RS] No device connected, skipping enumeration\n\r");
        return;
    }

    // Get device descriptor at address 0
    let mut dev_desc = [0u8; 18];
    let ok = usb::enumeration::get_device_descriptor(hcd, 0, &mut dev_desc);
    if !ok {
        log("[DWC2-RS] Failed to get device descriptor\n\r");
        return;
    }

    log("[DWC2-RS] Device Descriptor:\n\r");
    log("  bDeviceClass: 0x");
    log_hex(dev_desc[4] as u64);
    log("\n\r  idVendor: 0x");
    log_hex(u16::from_le_bytes([dev_desc[8], dev_desc[9]]) as u64);
    log("  idProduct: 0x");
    log_hex(u16::from_le_bytes([dev_desc[10], dev_desc[11]]) as u64);
    log("\n\r");

    if dev_desc[4] == usb::descriptors::CLASS_HUB {
        log("[DWC2-RS] Found USB Hub. Traversing...\n\r");
        
        if !usb::enumeration::set_address(hcd, 1) {
            log("[DWC2-RS] Failed to set hub address\n\r");
            return;
        }
        hcd.delay_ms(10);
        
        let mut config_buf = [0u8; 64];
        let config_len = usb::enumeration::get_config_descriptor(hcd, 1, &mut config_buf);
        if config_len > 0 {
            usb::enumeration::set_configuration(hcd, 1, config_buf[5]);
        }
        
        log("[DWC2-RS] Powering on Hub ports...\n\r");
        for port in 1..=4 {
            usb::enumeration::hub_set_port_power(hcd, 1, port);
        }
        hcd.delay_ms(50);
        
        let mut connected_port = 0;
        for port in 1..=4 {
            let status = usb::enumeration::hub_get_port_status(hcd, 1, port);
            if (status & 0x00000001) != 0 {
                connected_port = port;
                log("[DWC2-RS] Device connected on Hub port ");
                log_hex(port as u64);
                log("\n\r");
                break;
            }
        }
        
        if connected_port == 0 {
            log("[DWC2-RS] No devices found on Hub ports\n\r");
            return;
        }
        
        log("[DWC2-RS] Resetting Hub port...\n\r");
        usb::enumeration::hub_set_port_reset(hcd, 1, connected_port);
        hcd.delay_ms(50);
        
        let mut dev2_desc = [0u8; 18];
        let ok = usb::enumeration::get_device_descriptor(hcd, 0, &mut dev2_desc);
        if !ok {
            log("[DWC2-RS] Failed to get device descriptor behind Hub\n\r");
            return;
        }
        
        log("[DWC2-RS] Downstream Device Descriptor:\n\r");
        log("  bDeviceClass: 0x");
        log_hex(dev2_desc[4] as u64);
        log("\n\r  idVendor: 0x");
        log_hex(u16::from_le_bytes([dev2_desc[8], dev2_desc[9]]) as u64);
        log("  idProduct: 0x");
        log_hex(u16::from_le_bytes([dev2_desc[10], dev2_desc[11]]) as u64);
        log("\n\r");
        
        if !usb::enumeration::set_address(hcd, 2) {
            log("[DWC2-RS] Failed to set downstream device address\n\r");
            return;
        }
        hcd.delay_ms(10);
        
        let config_len2 = usb::enumeration::get_config_descriptor(hcd, 2, &mut config_buf);
        if config_len2 == 0 {
            log("[DWC2-RS] Failed to get downstream config descriptor\n\r");
            return;
        }
        
        let hid_info = usb::enumeration::find_hid_keyboard(&config_buf[..config_len2]);
        match hid_info {
            Some(info) => {
                log("[DWC2-RS] Found HID keyboard behind Hub: interface=");
                log_hex(info.interface as u64);
                log(" endpoint_in=0x");
                log_hex(info.endpoint_in as u64);
                log("\n\r");

                if !usb::enumeration::set_configuration(hcd, 2, config_buf[5]) {
                    log("[DWC2-RS] Failed to set downstream configuration\n\r");
                    return;
                }

                usb::enumeration::set_boot_protocol(hcd, 2, info.interface);
                usb::enumeration::set_idle(hcd, 2, info.interface, 0);

                let kbd = Keyboard::new(2, info.endpoint_in, info.interface);
                unsafe { KEYBOARD = Some(kbd); }
                KEYBOARD_READY.store(true, Ordering::Release);
                log("[DWC2-RS] HID keyboard configured and ready\n\r");
            }
            None => {
                log("[DWC2-RS] No HID keyboard interface found behind Hub\n\r");
            }
        }
        
        log("[DWC2-RS] Device enumeration complete\n\r");
        return;
    }

    // Set address to 1
    if !usb::enumeration::set_address(hcd, 1) {
        log("[DWC2-RS] Failed to set address\n\r");
        return;
    }
    hcd.delay_ms(10);

    // Get config descriptor at address 1
    let mut config_buf = [0u8; 64];
    let config_len = usb::enumeration::get_config_descriptor(hcd, 1, &mut config_buf);
    if config_len == 0 {
        log("[DWC2-RS] Failed to get config descriptor\n\r");
        return;
    }

    // Parse config for HID interface
    let hid_info = usb::enumeration::find_hid_keyboard(&config_buf[..config_len]);
    match hid_info {
        Some(info) => {
            log("[DWC2-RS] Found HID keyboard: interface=");
            log_hex(info.interface as u64);
            log(" endpoint_in=0x");
            log_hex(info.endpoint_in as u64);
            log("\n\r");

            // Set configuration
            if !usb::enumeration::set_configuration(hcd, 1, config_buf[5]) {
                log("[DWC2-RS] Failed to set configuration\n\r");
                return;
            }

            // Set boot protocol
            usb::enumeration::set_boot_protocol(hcd, 1, info.interface);

            // Set idle (rate=0 → only report on change)
            usb::enumeration::set_idle(hcd, 1, info.interface, 0);

            let kbd = Keyboard::new(1, info.endpoint_in, info.interface);
            unsafe { KEYBOARD = Some(kbd); }
            KEYBOARD_READY.store(true, Ordering::Release);

            log("[DWC2-RS] HID keyboard configured and ready\n\r");
        }
        None => {
            log("[DWC2-RS] No HID keyboard interface found\n\r");
        }
    }

    log("[DWC2-RS] Device enumeration complete\n\r");
}

#[no_mangle]
pub extern "C" fn usb_keyboard_connected() -> i32 {
    if KEYBOARD_READY.load(Ordering::Acquire) { 1 } else { 0 }
}

#[no_mangle]
pub extern "C" fn usb_keyboard_getc() -> u8 {
    if !KEYBOARD_READY.load(Ordering::Acquire) {
        return 0;
    }

    let hcd = unsafe {
        match HCD.as_mut() {
            Some(h) => h,
            None => return 0,
        }
    };

    let kbd = unsafe {
        match KEYBOARD.as_mut() {
            Some(k) => k,
            None => return 0,
        }
    };

    kbd.poll(hcd)
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    log("[DWC2-RS] PANIC!\n\r");
    loop {
        unsafe { core::arch::asm!("wfe"); }
    }
}
