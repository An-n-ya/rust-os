pub const BLOCK_SIZE: usize = 512;

pub struct Buf {
    pub flag: Flag,
    pub device: Device,
    pub block_num: usize,
    pub data: [u8; BLOCK_SIZE],
}

pub enum Flag {
    Read,
    Dirty,
}

pub struct Device {
    pub channel: u8,
    pub master: bool,
}

impl Buf {
    pub fn new(block_num: usize) -> Self {
        // currently only support reading from index1 disk
        Self {
            flag: Flag::Read,
            device: Device {
                channel: 0,
                master: false,
            },
            block_num,
            data: [0; 512],
        }
    }
}
