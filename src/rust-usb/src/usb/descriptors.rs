#![allow(dead_code)]
//! USB descriptor constants and parsing helpers.

// Descriptor types
pub const DESC_DEVICE: u8 = 0x01;
pub const DESC_CONFIGURATION: u8 = 0x02;
pub const DESC_INTERFACE: u8 = 0x04;
pub const DESC_ENDPOINT: u8 = 0x05;
pub const DESC_HID: u8 = 0x21;

// Request types
pub const DIR_IN: u8 = 0x80;
pub const DIR_OUT: u8 = 0x00;
pub const TYPE_STANDARD: u8 = 0x00;
pub const TYPE_CLASS: u8 = 0x20;
pub const RECIP_DEVICE: u8 = 0x00;
pub const RECIP_INTERFACE: u8 = 0x01;

// Standard requests
pub const REQ_GET_DESCRIPTOR: u8 = 0x06;
pub const REQ_SET_ADDRESS: u8 = 0x05;
pub const REQ_SET_CONFIGURATION: u8 = 0x09;

// HID requests
pub const HID_REQ_SET_PROTOCOL: u8 = 0x0B;
pub const HID_REQ_SET_IDLE: u8 = 0x0A;
pub const HID_REQ_GET_REPORT: u8 = 0x01;

// Hub class
pub const CLASS_HUB: u8 = 0x09;

// Feature selectors
pub const FEATURE_PORT_POWER: u16 = 8;
pub const FEATURE_PORT_RESET: u16 = 4;

// Hub requests
pub const HUB_REQ_SET_FEATURE: u8 = 0x03;
pub const HUB_REQ_GET_STATUS: u8 = 0x00;

// Request types for Hub
pub const RECIP_OTHER: u8 = 0x03;
pub const HUB_SET_PORT_FEATURE_REQ: u8 = DIR_OUT | TYPE_CLASS | RECIP_OTHER;
pub const HUB_GET_PORT_STATUS_REQ: u8 = DIR_IN | TYPE_CLASS | RECIP_OTHER;

// HID class
pub const CLASS_HID: u8 = 0x03;
pub const SUBCLASS_BOOT: u8 = 0x01;
pub const PROTOCOL_KEYBOARD: u8 = 0x01;

/// Build an 8-byte SETUP packet.
pub fn make_setup(
    bm_request_type: u8,
    b_request: u8,
    w_value: u16,
    w_index: u16,
    w_length: u16,
) -> [u8; 8] {
    [
        bm_request_type,
        b_request,
        (w_value & 0xFF) as u8,
        (w_value >> 8) as u8,
        (w_index & 0xFF) as u8,
        (w_index >> 8) as u8,
        (w_length & 0xFF) as u8,
        (w_length >> 8) as u8,
    ]
}

/// Info about a HID keyboard found in a configuration descriptor.
pub struct HidKeyboardInfo {
    pub interface: u8,
    pub endpoint_in: u8,
}

/// Walk a config descriptor blob and find an interface with HID boot keyboard.
pub fn parse_config_for_hid_keyboard(buf: &[u8]) -> Option<HidKeyboardInfo> {
    let mut i = 0;
    let mut current_iface: Option<u8> = None;
    let mut is_hid_keyboard = false;

    while i + 1 < buf.len() {
        let b_length = buf[i] as usize;
        let b_desc_type = buf[i + 1];

        if b_length < 2 || i + b_length > buf.len() {
            break;
        }

        if b_desc_type == DESC_INTERFACE && b_length >= 9 {
            let iface_num = buf[i + 2];
            let iface_class = buf[i + 5];
            let iface_subclass = buf[i + 6];
            let iface_protocol = buf[i + 7];

            current_iface = Some(iface_num);
            is_hid_keyboard = iface_class == CLASS_HID
                && iface_subclass == SUBCLASS_BOOT
                && iface_protocol == PROTOCOL_KEYBOARD;
        }

        if b_desc_type == DESC_ENDPOINT && b_length >= 7 && is_hid_keyboard {
            let ep_addr = buf[i + 2];
            let ep_attrs = buf[i + 3];

            // Interrupt IN endpoint
            if ep_addr & 0x80 != 0 && (ep_attrs & 0x03) == 0x03 {
                return Some(HidKeyboardInfo {
                    interface: current_iface.unwrap_or(0),
                    endpoint_in: ep_addr & 0x0F,
                });
            }
        }

        i += b_length;
    }

    None
}
