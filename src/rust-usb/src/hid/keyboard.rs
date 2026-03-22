//! HID Boot Protocol keyboard driver.
//!
//! Boot protocol report (8 bytes):
//!   [0] modifier keys (Ctrl, Shift, Alt, etc.)
//!   [1] reserved (always 0)
//!   [2..7] up to 6 keycodes

use crate::dwc2::Dwc2Hcd;

pub struct Keyboard {
    dev_addr: u8,
    ep_num: u8,
    _iface: u8,
    toggle: bool,
    prev_keys: [u8; 6],
    key_buf: [u8; 16],
    key_head: usize,
    key_tail: usize,
}

impl Keyboard {
    pub fn new(dev_addr: u8, ep_num: u8, iface: u8) -> Self {
        Self {
            dev_addr,
            ep_num,
            _iface: iface,
            toggle: false,
            prev_keys: [0; 6],
            key_buf: [0; 16],
            key_head: 0,
            key_tail: 0,
        }
    }

    pub fn poll(&mut self, hcd: &mut Dwc2Hcd) -> u8 {
        if self.key_head != self.key_tail {
            let ch = self.key_buf[self.key_tail];
            self.key_tail = (self.key_tail + 1) & 0xF;
            return ch;
        }

        let mut report = [0u8; 8];
        if !hcd.interrupt_transfer_in(
            self.dev_addr,
            self.ep_num,
            &mut report,
            &mut self.toggle,
        ) {
            return 0;
        }

        let modifier = report[0];

        for i in 2..8 {
            let key = report[i];
            if key == 0 || key == 1 {
                continue;
            }

            let mut was_pressed = false;
            for prev in &self.prev_keys {
                if *prev == key {
                    was_pressed = true;
                    break;
                }
            }

            if !was_pressed {
                let ascii = scancode_to_ascii(key, modifier);
                if ascii != 0 {
                    self.push_key(ascii);
                }
            }
        }

        self.prev_keys.copy_from_slice(&report[2..8]);

        if self.key_head != self.key_tail {
            let ch = self.key_buf[self.key_tail];
            self.key_tail = (self.key_tail + 1) & 0xF;
            ch
        } else {
            0
        }
    }

    fn push_key(&mut self, ch: u8) {
        let next = (self.key_head + 1) & 0xF;
        if next != self.key_tail {
            self.key_buf[self.key_head] = ch;
            self.key_head = next;
        }
    }
}

fn scancode_to_ascii(scancode: u8, modifier: u8) -> u8 {
    let shift = (modifier & 0x22) != 0;
    #[rustfmt::skip]
    const NORMAL: [u8; 57] = [
        0, 0, 0, 0,
        b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j',
        b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't',
        b'u', b'v', b'w', b'x', b'y', b'z',
        b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0',
        b'\r', 0x1B, 0x08, b'\t', b' ',
        b'-', b'=', b'[', b']', b'\\',
        0, // non-US #
        b';', b'\'', b'`', b',', b'.', b'/',
    ];

    #[rustfmt::skip]
    const SHIFTED: [u8; 57] = [
        0, 0, 0, 0,
        b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J',
        b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T',
        b'U', b'V', b'W', b'X', b'Y', b'Z',
        b'!', b'@', b'#', b'$', b'%', b'^', b'&', b'*', b'(', b')',
        b'\r', 0x1B, 0x08, b'\t', b' ',
        b'_', b'+', b'{', b'}', b'|',
        0,
        b':', b'"', b'~', b'<', b'>', b'?',
    ];

    let sc = scancode as usize;
    if sc >= 57 { return 0; }

    if shift { SHIFTED[sc] } else { NORMAL[sc] }
}
