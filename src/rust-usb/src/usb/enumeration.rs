//! USB device enumeration: GET_DESCRIPTOR, SET_ADDRESS, SET_CONFIGURATION, etc.

use crate::dwc2::Dwc2Hcd;
use crate::log;
use super::descriptors::*;

/// Get device descriptor (18 bytes) from a device at `addr`.
pub fn get_device_descriptor(hcd: &mut Dwc2Hcd, addr: u8, buf: &mut [u8; 18]) -> bool {
    let setup = make_setup(
        DIR_IN | TYPE_STANDARD | RECIP_DEVICE,
        REQ_GET_DESCRIPTOR,
        (DESC_DEVICE as u16) << 8,
        0,
        18,
    );
    hcd.control_transfer_in(addr, &setup, buf)
}

/// Set device address.
pub fn set_address(hcd: &mut Dwc2Hcd, new_addr: u8) -> bool {
    log("[DWC2-RS] Setting address to ");
    crate::log_hex(new_addr as u64);
    log("\n\r");

    let setup = make_setup(
        DIR_OUT | TYPE_STANDARD | RECIP_DEVICE,
        REQ_SET_ADDRESS,
        new_addr as u16,
        0,
        0,
    );
    hcd.control_transfer_out_nodata(0, &setup)
}

/// Get configuration descriptor. Returns actual length read, or 0 on failure.
pub fn get_config_descriptor(hcd: &mut Dwc2Hcd, addr: u8, buf: &mut [u8]) -> usize {
    // First, read just 9 bytes to get wTotalLength
    let mut header = [0u8; 9];
    let setup = make_setup(
        DIR_IN | TYPE_STANDARD | RECIP_DEVICE,
        REQ_GET_DESCRIPTOR,
        (DESC_CONFIGURATION as u16) << 8,
        0,
        9,
    );
    if !hcd.control_transfer_in(addr, &setup, &mut header) {
        return 0;
    }

    let total_len = u16::from_le_bytes([header[2], header[3]]) as usize;
    let read_len = if total_len < buf.len() { total_len } else { buf.len() };

    // Now read the full config descriptor
    let setup = make_setup(
        DIR_IN | TYPE_STANDARD | RECIP_DEVICE,
        REQ_GET_DESCRIPTOR,
        (DESC_CONFIGURATION as u16) << 8,
        0,
        read_len as u16,
    );
    if !hcd.control_transfer_in(addr, &setup, &mut buf[..read_len]) {
        return 0;
    }

    read_len
}

/// Set configuration.
pub fn set_configuration(hcd: &mut Dwc2Hcd, addr: u8, config_value: u8) -> bool {
    log("[DWC2-RS] Setting configuration ");
    crate::log_hex(config_value as u64);
    log("\n\r");

    let setup = make_setup(
        DIR_OUT | TYPE_STANDARD | RECIP_DEVICE,
        REQ_SET_CONFIGURATION,
        config_value as u16,
        0,
        0,
    );
    hcd.control_transfer_out_nodata(addr, &setup)
}

/// Set HID boot protocol (protocol=0 for boot, 1 for report).
pub fn set_boot_protocol(hcd: &mut Dwc2Hcd, addr: u8, iface: u8) {
    log("[DWC2-RS] Setting boot protocol\n\r");
    let setup = make_setup(
        DIR_OUT | TYPE_CLASS | RECIP_INTERFACE,
        HID_REQ_SET_PROTOCOL,
        0, // 0 = boot protocol
        iface as u16,
        0,
    );
    let _ = hcd.control_transfer_out_nodata(addr, &setup);
}

/// Set idle rate.
pub fn set_idle(hcd: &mut Dwc2Hcd, addr: u8, iface: u8, rate: u8) {
    let setup = make_setup(
        DIR_OUT | TYPE_CLASS | RECIP_INTERFACE,
        HID_REQ_SET_IDLE,
        (rate as u16) << 8,
        iface as u16,
        0,
    );
    let _ = hcd.control_transfer_out_nodata(addr, &setup);
}

/// Find HID keyboard in config descriptor buffer.
pub fn find_hid_keyboard(config_buf: &[u8]) -> Option<HidKeyboardInfo> {
    parse_config_for_hid_keyboard(config_buf)
}

pub fn hub_set_port_power(hcd: &mut Dwc2Hcd, hub_addr: u8, port: u8) {
    let setup = make_setup(
        super::descriptors::HUB_SET_PORT_FEATURE_REQ,
        super::descriptors::HUB_REQ_SET_FEATURE,
        super::descriptors::FEATURE_PORT_POWER,
        port as u16,
        0,
    );
    let _ = hcd.control_transfer_out_nodata(hub_addr, &setup);
}

pub fn hub_set_port_reset(hcd: &mut Dwc2Hcd, hub_addr: u8, port: u8) {
    let setup = make_setup(
        super::descriptors::HUB_SET_PORT_FEATURE_REQ,
        super::descriptors::HUB_REQ_SET_FEATURE,
        super::descriptors::FEATURE_PORT_RESET,
        port as u16,
        0,
    );
    let _ = hcd.control_transfer_out_nodata(hub_addr, &setup);
}

pub fn hub_get_port_status(hcd: &mut Dwc2Hcd, hub_addr: u8, port: u8) -> u32 {
    let mut buf = [0u8; 4];
    let setup = make_setup(
        super::descriptors::HUB_GET_PORT_STATUS_REQ,
        super::descriptors::HUB_REQ_GET_STATUS,
        0,
        port as u16,
        4,
    );
    if hcd.control_transfer_in(hub_addr, &setup, &mut buf) {
        u32::from_le_bytes(buf)
    } else {
        0
    }
}
