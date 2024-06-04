use core::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use super::frame::FrameAllocator;

pub const P4: *mut PageTable<Level4> = 0o177777_776_776_776_776_0000 as *mut _;

#[repr(C)]
pub struct PageTableEntry(u64);

#[repr(C, align(4096))]
pub struct PageTable<L: TableLevel> {
    pub entries: [PageTableEntry; 512],
    level: PhantomData<L>,
}

pub trait TableLevel {}
pub trait HierarchicalLevel: TableLevel {
    type NextLevel: TableLevel;
}
pub enum Level4 {}
pub enum Level3 {}
pub enum Level2 {}
pub enum Level1 {}
impl TableLevel for Level4 {}
impl TableLevel for Level3 {}
impl TableLevel for Level2 {}
impl TableLevel for Level1 {}
impl HierarchicalLevel for Level4 {
    type NextLevel = Level3;
}
impl HierarchicalLevel for Level3 {
    type NextLevel = Level2;
}
impl HierarchicalLevel for Level2 {
    type NextLevel = Level1;
}

impl PageTableEntry {
    pub fn is_unused(&self) -> bool {
        self.0 == 0
    }
    pub fn set_unused(&mut self) {
        self.0 = 0
    }
    pub fn addr(&self) -> u64 {
        self.0 & 0x000f_ffff_ffff_f000
    }
    pub fn set_addr(&mut self, addr: u64) -> &mut Self {
        assert!(
            addr % 4096 == 0,
            "addr should be align to 4K, got addr {:#X}",
            addr
        );
        self.0 &= !(0x000f_ffff_ffff_f000);
        self.0 |= addr & 0x000f_ffff_ffff_f000;
        self
    }
    pub fn is_present(&self) -> bool {
        self.0 & 1 != 0
    }
    pub fn set_present(&mut self, flag: bool) -> &mut Self {
        if flag {
            self.0 |= 1;
        } else {
            self.0 &= !(1);
        }
        self
    }
    pub fn is_writable(&self) -> bool {
        self.0 & 1 << 1 != 0
    }
    pub fn set_writable(&mut self, flag: bool) -> &mut Self {
        if flag {
            self.0 |= 1 << 1;
        } else {
            self.0 &= !(1 << 1);
        }
        self
    }
    pub fn is_huge(&self) -> bool {
        self.0 & 1 << 7 != 0
    }
    pub fn set_huge(&mut self, flag: bool) -> &mut Self {
        if flag {
            self.0 |= 1 << 7;
        } else {
            self.0 &= !(1 << 7);
        }
        self
    }
}

impl<L> Index<usize> for PageTable<L>
where
    L: TableLevel,
{
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl<L> IndexMut<usize> for PageTable<L>
where
    L: TableLevel,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

impl<L> PageTable<L>
where
    L: TableLevel,
{
    pub fn reset(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_unused();
        }
    }

    fn next_table_address(&self, index: usize) -> Option<u64> {
        assert!(index < 512);
        if self[index].is_present() && !self[index].is_huge() {
            let table_address = self as *const _ as u64;
            let index = index as u64;
            Some((table_address << 9) | (index << 12) | (1 << 48))
        } else {
            None
        }
    }
}
impl<L> PageTable<L>
where
    L: HierarchicalLevel,
{
    pub fn next_table_is_huge(&self, index: usize) -> bool {
        assert!(index < 512);
        self[index].is_huge()
    }
    pub fn next_table(&self, index: usize) -> Option<&PageTable<L::NextLevel>> {
        self.next_table_address(index)
            .map(|addr| unsafe { &*(addr as *const _) })
    }
    pub fn next_table_mut(&self, index: usize) -> Option<&mut PageTable<L::NextLevel>> {
        self.next_table_address(index)
            .map(|addr| unsafe { &mut *(addr as *mut _) })
    }

    pub fn next_table_create<A>(
        &mut self,
        index: usize,
        allocator: &mut A,
    ) -> &mut PageTable<L::NextLevel>
    where
        A: FrameAllocator,
    {
        if self.next_table(index).is_none() {
            assert!(
                !self[index].is_huge(),
                "UNIMPLEMENTED: mapping to huge pages"
            );
            let frame = allocator.allocate_frame().expect("no frame available");
            self.entries[index]
                .set_addr(frame.addr)
                .set_present(true)
                .set_writable(true);
            self.next_table_mut(index).unwrap().reset();
        }
        self.next_table_mut(index).unwrap()
    }
}

pub fn flush_page_table() {
    let mut cr3: u64;
    unsafe {
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack, preserves_flags));
        core::arch::asm!("mov cr3, {}", in(reg) cr3, options(nomem, nostack, preserves_flags));
    }
}
