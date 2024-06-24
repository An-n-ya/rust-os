use core::sync::atomic::AtomicBool;

use crate::utils::port::Port;

use super::buf::{Buf, BLOCK_SIZE};

static mut IDE_LOCK: AtomicBool = AtomicBool::new(false);

const SECTOR_SIZE: usize = 512;

const IDE_BUSY: u8 = 0x80;
const IDE_READY: u8 = 0x40;

const DATA_PORT: Port = Port::new(0x1f0);
const CMD_PORT: Port = Port::new(0x1f7);
const DEV_PORT: Port = Port::new(0x1f6);
const LSB1_PORT: Port = Port::new(0x1f3);
const LSB2_PORT: Port = Port::new(0x1f4);
const LSB3_PORT: Port = Port::new(0x1f5);

pub fn ide_wait() {
    loop {
        if CMD_PORT.read_u8() & (IDE_BUSY | IDE_READY) == IDE_READY {
            break;
        }
    }
}

pub fn ide_init() {
    // check if disk 1 is present
    // 0xe0 5 and 7 bit must be 1, 6 bit 1 -> LBA, 0 -> CHS
    DEV_PORT.write_u8(0xe0 | (1 << 4));
    let mut flag = false;
    for _ in 0..1000 {
        if CMD_PORT.read_u8() != 0 {
            println!("disk1 detected!");
            flag = true;
            break;
        }
    }
    if !flag {
        println!("disk1 undetected!");
    }
    // back to disk0
    DEV_PORT.write_u8(0xe0 | (0 << 4));
}

pub fn ide_start(buf: &Buf) {
    // FIXME: how to restrain the block_num
    let sector_per_block = BLOCK_SIZE / SECTOR_SIZE;
    let sector = buf.block_num * sector_per_block;

    let slave_flag = {
        if !buf.device.master {
            1 << 4
        } else {
            0
        }
    };
    LSB1_PORT.write_u8((sector & 0xff) as u8);
    LSB2_PORT.write_u8(((sector >> 8) & 0xff) as u8);
    LSB2_PORT.write_u8(((sector >> 16) & 0xff) as u8);

    // drive select
    DEV_PORT.write_u8(0xe0 | slave_flag | ((sector >> 24) & 0x0f) as u8);
    ide_wait();

    match buf.flag {
        super::buf::Flag::Read => {
            CMD_PORT.write_u8(0x20);
            // wait interrupt
            unsafe {
                IDE_LOCK.store(true, core::sync::atomic::Ordering::Release);

                loop {
                    if !IDE_LOCK.load(core::sync::atomic::Ordering::Acquire) {
                        break;
                    }
                }
            }

            DATA_PORT.read_u32_to(&buf.data as *const _ as *const u32, BLOCK_SIZE / 4);
        }
        super::buf::Flag::Dirty => todo!(),
    }
}

pub fn ide_intr() {
    unsafe {
        CMD_PORT.read_u8();
        IDE_LOCK.store(false, core::sync::atomic::Ordering::Release);
    }
}
