use crate::utils::port::Port;

const SECTOR_SIZE: usize = 512;

const IDE_BUSY: u8 = 0x80;
const IDE_READY: u8 = 0x40;

const CMD_PORT: Port = Port::new(0x1f7);
const DEV_PORT: Port = Port::new(0x1f6);

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
