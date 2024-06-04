use self::heap_allocator::HEAP_SIZE;
use self::heap_allocator::HEAP_START;

use self::page_table::Page;

use self::page_table::flush_page_table;

use self::frame::Frame;
use self::frame::FrameAllocator;
use self::page_table::PageTableEntry;

use self::page_table::Level3;
use self::page_table::Level4;

use self::page_table::P4;

use self::frame::PageSize;

use self::frame::Allocator;

use self::page_table::PageTable;

use crate::{MultibootInfo, KERNEL_BASE};

pub mod frame;
pub mod heap_allocator;
mod page_table;

pub fn init<A>(allocator: &mut A)
where
    A: FrameAllocator,
{
    // map heap
    let heap_start_page = Page::new_small_page(HEAP_START as u64);
    let heap_end_page = Page::new_small_page((HEAP_START + HEAP_SIZE) as u64);

    for page in Page::range_inclusive(heap_start_page, heap_end_page) {
        map(page, allocator.allocate_frame().unwrap(), allocator).set_writable(true);
    }
}

pub fn read_page() {
    let cr3: u64;
    unsafe {
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack, preserves_flags));
    }
    log!("cr3: {cr3:#X}");
    let p4 = unsafe { &*P4 };
    for i in 0..512 {
        if !p4[i].is_unused() {
            log!("pml4 entry {} -> physical addr {:#X}", i, p4[i].addr());
        }
    }
    let pml4 = cr3 + KERNEL_BASE;
    let page_table_4 = pml4 as *const PageTable<Level4>;
    for (i, entry) in unsafe { (*page_table_4).entries.iter().enumerate() } {
        if !entry.is_unused() {
            log!("pml4 entry {} -> physical addr {:#X}", i, entry.addr());
            if entry.addr() != cr3 {
                let pdpt = entry.addr() + KERNEL_BASE;
                let pdpt = pdpt as *const PageTable<Level3>;
                for (i, entry) in unsafe { (*pdpt).entries.iter().enumerate() } {
                    if !entry.is_unused() {
                        log!("  pdpt entry {} -> physical addr {:#X}", i, entry.addr());
                    }
                }
            }
        }
    }
}

pub fn virt_to_physical(virt_addr: u64) -> u64 {
    assert!(
        virt_addr < 0x0000_8000_0000_0000 || virt_addr >= 0xffff_8000_0000_0000,
        "invalid address: {:#X}",
        virt_addr
    );

    let cr3: u64;
    unsafe {
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack, preserves_flags));
    }
    let pml4 = cr3 + KERNEL_BASE;
    let mut table = pml4 as *const PageTable<Level4>;
    let page_index = [
        (virt_addr & 0o777 << 39) >> 39,
        (virt_addr & 0o777 << 30) >> 30,
        (virt_addr & 0o777 << 21) >> 21,
        (virt_addr & 0o777 << 12) >> 12,
    ];
    for (i, index) in page_index.iter().enumerate() {
        // log!("table addr {:#X}", table as u64);
        let entry = unsafe { &(*table).entries[*index as usize] };
        assert!(
            !entry.is_unused(),
            "try to access unused page, addr: {:#X}",
            virt_addr
        );
        if (i == 1 || i == 2) && entry.is_huge() {
            return entry.addr() + virt_addr & if i == 1 { 0x3fff_ffff } else { 0x001f_ffff };
        }

        table = (entry.addr() + KERNEL_BASE) as *const PageTable<Level4>;
        if i == 3 {
            return entry.addr() + virt_addr & 0x0fff;
        }
    }
    unreachable!()
}

pub fn map<A>(page: Page, frame: Frame, allocator: &mut A) -> &mut PageTableEntry
where
    A: FrameAllocator,
{
    assert!(
        page.size == PageSize::Small,
        "UNIMPLEMENTED: only 4kb mapping is support currently"
    );
    let p4 = unsafe { &mut *P4 };
    let p3 = p4.next_table_create(page.p4_index(), allocator);
    let p2 = p3.next_table_create(page.p3_index(), allocator);
    let p1 = p2.next_table_create(page.p2_index(), allocator);

    assert!(p1[page.p1_index()].is_unused());
    let entry = p1[page.p1_index()].set_addr(frame.addr).set_present(true);

    flush_page_table();

    entry
}

pub fn unmap<A>(virt_addr: u64, allocator: &mut A)
where
    A: FrameAllocator,
{
    // to test if this addr is valid
    virt_to_physical(virt_addr);
    let p4 = unsafe { &mut *P4 };

    let page_index = [
        (virt_addr & 0o777 << 39) >> 39,
        (virt_addr & 0o777 << 30) >> 30,
        (virt_addr & 0o777 << 21) >> 21,
        (virt_addr & 0o777 << 12) >> 12,
    ];
    let p1 = p4
        .next_table_mut(page_index[0] as usize)
        .and_then(|p3| p3.next_table_mut(page_index[1] as usize))
        .and_then(|p2| p2.next_table_mut(page_index[2] as usize))
        .expect("UNIMPLEMENTED: unmap huge page");

    let frame = Frame {
        addr: p1[page_index[3] as usize].addr(),
        size: PageSize::Small,
    };
    // TODO: clear p2 p3 p4 if they are empty
    p1[page_index[3] as usize].set_unused();

    allocator.deallocate_frame(frame);

    flush_page_table();
}

pub fn test_allocator(info: *const MultibootInfo, kernel_range: (u64, u64)) {
    let mut allocator = Allocator::new(info, kernel_range);
    let f = allocator.new_frame();
    assert!(f.size == PageSize::Small);
    assert!(
        f.addr == (kernel_range.1 & 0xffff_f000) + 0x1000,
        "wrong address allocation addr: {:#X}",
        f.addr
    );
}

pub fn test_map<A>(allocator: &mut A)
where
    A: FrameAllocator,
{
    let virt_addr = 0xffff_0000;
    let frame = allocator.allocate_frame().expect("no more frames");
    let physical_addr = frame.addr;
    map(Page::new_small_page(virt_addr), frame, allocator).set_writable(true);
    log!("map virt addr {:#X} to {:#X}", virt_addr, physical_addr);

    let a = virt_addr as *mut u64;
    log!("a value {:#X}", unsafe { *a });
    unsafe {
        *a = 1;
    }
    log!("a value {:#X}", unsafe { *a });

    // unmap(virt_addr, allocator);
    // log!("hello");
    // FIXME: expect page fault, but got stuck
    // log!("a value {:#X}", unsafe { *a });
}
