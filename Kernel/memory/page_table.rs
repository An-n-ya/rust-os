#[repr(C)]
pub struct PageTableEntry(u64);

#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: [PageTableEntry; 512],
}

impl PageTableEntry {
    pub fn is_unused(&self) -> bool {
        self.0 == 0
    }
    pub fn addr(&self) -> u64 {
        self.0 & 0x000f_ffff_ffff_f000
    }
    pub fn is_present(&self) -> bool {
        self.0 & 1 != 0
    }
    pub fn is_writable(&self) -> bool {
        self.0 & 1 << 1 != 0
    }
    pub fn is_huge(&self) -> bool {
        self.0 & 1 << 7 != 0
    }
}
