use crate::{MultibootInfo, KERNEL_BASE};

#[derive(PartialEq, Eq)]
pub struct Frame {
    pub addr: u64,
    pub size: PageSize,
}

#[derive(Clone, Copy)]
struct Area {
    size: u32,
    addr: u64,
    len: u64,
    typ: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PageSize {
    Small = 0x1000,
    Medium = 0x0020_0000,
    Large = 0x4000_0000,
}

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}

pub struct Allocator {
    areas: AreaIterator,
    kernel_start: u64,
    kernel_end: u64,
    multiboot_start: u64,
    multiboot_end: u64,
    cur_area: Area,
    current_addr: u64,
}

struct AreaIterator {
    addr: u64,
    cur: usize,
    length: usize,
}

impl Frame {
    // not included
    pub fn end_addr(&self) -> u64 {
        self.addr + self.size as u64 - 1
    }
    pub fn new_small_page(addr: u64) -> Self {
        Self {
            addr,
            size: PageSize::Small,
        }
    }
}

impl Iterator for AreaIterator {
    type Item = Area;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur >= self.length {
            return None;
        }
        let entry =
            (self.addr + KERNEL_BASE + core::mem::size_of::<Area>() as u64 * self.cur as u64)
                as *const Area;
        let entry = unsafe { *entry };
        self.cur += 1;
        Some(entry)
    }
}

impl Allocator {
    pub fn new(info: *const MultibootInfo, kernel_range: (u64, u64)) -> Self {
        let (mmap_length, mmap_addr) = unsafe { ((*info).mmap_length, (*info).mmap_addr) };
        let mut areas = AreaIterator {
            addr: mmap_addr as u64,
            cur: 0,
            length: mmap_length as usize,
        };
        let multiboot_start = info as u64;
        let multiboot_end = multiboot_start + core::mem::size_of::<MultibootInfo>() as u64;
        let area = areas.next().unwrap();
        Self {
            cur_area: area,
            areas,
            kernel_start: kernel_range.0,
            kernel_end: kernel_range.1,
            multiboot_start,
            multiboot_end,
            current_addr: 0x100000,
        }
    }

    pub fn new_frame(&mut self) -> Frame {
        self.new_frame_with_type(PageSize::Small)
    }

    pub fn new_frame_with_type(&mut self, typ: PageSize) -> Frame {
        let mut area = self.cur_area;
        while self.in_kernel(self.current_addr, self.current_addr + typ as u64)
            || self.in_multiboot(self.current_addr, self.current_addr + typ as u64)
        {
            self.current_addr += typ as u64;
        }
        while !area.contain(self.current_addr) {
            area = self.areas.next().expect("out of physical memory");
        }
        self.cur_area = area;
        let frame = Frame {
            addr: self.current_addr,
            size: typ,
        };
        self.current_addr += typ as u64;

        frame
    }

    fn in_kernel(&self, start: u64, end: u64) -> bool {
        !(end < self.kernel_start || start > self.kernel_end)
    }
    fn in_multiboot(&self, start: u64, end: u64) -> bool {
        !(end < self.multiboot_start || start > self.multiboot_end)
    }
}

impl FrameAllocator for Allocator {
    fn allocate_frame(&mut self) -> Option<Frame> {
        Some(self.new_frame())
    }

    fn deallocate_frame(&mut self, _frame: Frame) {
        // TODO
        // todo!()
    }
}

impl Area {
    pub fn end_addr(&self) -> u64 {
        self.addr + self.len - 1
    }
    pub fn contain(&self, addr: u64) -> bool {
        addr >= self.addr && addr <= self.end_addr()
    }
}
