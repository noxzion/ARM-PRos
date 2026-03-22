#![allow(dead_code)]
//! DWC2 Host Channel management.
//!
//! Each DWC2 has up to 16 host channels. We use channel 0 for control
//! transfers and channel 1 for interrupt (HID) transfers.

use super::regs::*;

pub const CH_CONTROL: usize = 0;
pub const CH_INTERRUPT: usize = 1;
pub const MAX_CHANNELS: usize = 8;

#[inline]
pub fn ch_reg(base: usize, ch: usize, offset: usize) -> usize {
    base + HCCHAR + ch * CHANNEL_STRIDE + (offset - HCCHAR)
}

pub unsafe fn configure_channel(
    base: usize,
    ch: usize,
    dev_addr: u8,
    ep_num: u8,
    ep_dir_in: bool,
    ep_type: u32,
    max_pkt_size: u16,
    low_speed: bool,
) {
    let hcchar_addr = base + HCCHAR + ch * CHANNEL_STRIDE;

    let mut hcchar: u32 = 0;
    hcchar |= (max_pkt_size as u32) & HCCHAR_MPS_MASK;
    hcchar |= (ep_num as u32) << HCCHAR_EPNUM_SHIFT;
    if ep_dir_in { hcchar |= HCCHAR_EPDIR_IN; }
    if low_speed { hcchar |= HCCHAR_LSDEV; }
    hcchar |= ep_type << HCCHAR_EPTYPE_SHIFT;
    hcchar |= 1 << HCCHAR_MC_SHIFT;
    hcchar |= (dev_addr as u32) << HCCHAR_DEVADDR_SHIFT;

    write32(hcchar_addr, hcchar);
}

pub unsafe fn do_transfer(
    base: usize,
    ch: usize,
    pid: u32,
    data: *mut u8,
    len: usize,
) -> bool {
    let ch_base = base + ch * CHANNEL_STRIDE;
    let hcint_addr = ch_base + HCINT;
    let hcintmsk_addr = ch_base + HCINTMSK;
    let hctsiz_addr = ch_base + HCTSIZ;
    let hcdma_addr = ch_base + HCDMA;
    let hcchar_addr = ch_base + HCCHAR;

    write32(hcint_addr, 0xFFFF_FFFF);
    write32(hcintmsk_addr,
        HCINT_XFERCOMP | HCINT_CHHLTD | HCINT_STALL |
        HCINT_NAK | HCINT_ACK | HCINT_XACTERR | HCINT_BBLERR |
        HCINT_DATATGLERR | HCINT_AHBERR
    );

    let pkt_cnt = if len == 0 { 1 } else { (len + 63) / 64 }; // assume MPS=64
    let tsiz = (len as u32 & HCTSIZ_XFERSIZE_MASK)
        | ((pkt_cnt as u32) << HCTSIZ_PKTCNT_SHIFT)
        | pid;
    write32(hctsiz_addr, tsiz);

    if !data.is_null() {
        write32(hcdma_addr, data as u32);
    }

    let mut hcchar = read32(hcchar_addr);
    hcchar |= HCCHAR_CHEN;
    hcchar &= !HCCHAR_CHDIS;
    write32(hcchar_addr, hcchar);

    wait_channel_complete(base, ch)
}

unsafe fn wait_channel_complete(base: usize, ch: usize) -> bool {
    let hcint_addr = base + ch * CHANNEL_STRIDE + HCINT;
    let mut timeout: u32 = 500_000;

    loop {
        let hcint = read32(hcint_addr);

        if hcint & HCINT_XFERCOMP != 0 {
            write32(hcint_addr, 0xFFFF_FFFF);
            return true;
        }
        if hcint & HCINT_CHHLTD != 0 {
            if hcint & HCINT_ACK != 0 {
                write32(hcint_addr, 0xFFFF_FFFF);
                return true;
            }
            write32(hcint_addr, 0xFFFF_FFFF);
            return false;
        }
        if hcint & (HCINT_STALL | HCINT_AHBERR | HCINT_BBLERR) != 0 {
            write32(hcint_addr, 0xFFFF_FFFF);
            return false;
        }
        if hcint & HCINT_NAK != 0 {
            write32(hcint_addr, HCINT_NAK);
            write32(hcint_addr, 0xFFFF_FFFF);
            return false;
        }

        timeout -= 1;
        if timeout == 0 {
            let hcchar_addr = base + ch * CHANNEL_STRIDE + HCCHAR;
            let mut hcchar = read32(hcchar_addr);
            hcchar |= HCCHAR_CHDIS;
            write32(hcchar_addr, hcchar);
            write32(hcint_addr, 0xFFFF_FFFF);
            return false;
        }

        core::hint::spin_loop();
    }
}
