use self::heap_allocator::HEAP_SIZE;
use self::heap_allocator::HEAP_START;

use self::page_table::Page;

use self::page_table::flush_page_table;

use self::frame::Frame;
use self::frame::FrameAllocator;
use self::page_table::PageTableEntry;

use self::page_table::P4;

use self::frame::PageSize;

use self::frame::Allocator;

use crate::MultibootInfo;

pub mod frame;
pub mod gdt;
pub mod heap_allocator;
pub mod page_table;

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
    let mut addr = 0;
    for i in 0..512 {
        if !p4[i].is_unused() {
            log!("pml4 entry {} -> physical addr {:#X}", i, p4[i].addr());
            if i != 510 {
                let p3 = p4.next_table(i).unwrap();
                addr &= !(0o777 << 12 + 9 + 9 + 9);
                addr |= i << 39;
                for i in 0..512 {
                    if !p3[i].is_unused() && !p3[i].is_huge() {
                        log!("  pdpt entry {} -> physical addr {:#X}", i, p3[i].addr());
                        addr &= !(0o777 << 12 + 9 + 9);
                        addr |= i << 30;
                        if i != 510 {
                            let p2 = p3.next_table(i).unwrap();
                            for i in 0..512 {
                                if !p2[i].is_unused() && !p2[i].is_huge() {
                                    log!("    pd entry {} -> physical addr {:#X}", i, p2[i].addr());
                                    addr &= !(0o777 << 12 + 9);
                                    addr |= i << 21;
                                    if i != 510 {
                                        let p1 = p2.next_table(i).unwrap();
                                        for i in 0..512 {
                                            if !p1[i].is_unused() {
                                                addr &= !(0o777 << 12);
                                                addr |= i << 12;
                                                let mut flags = ['-', '-', '-'];
                                                if p1[i].is_writable() {
                                                    flags[0] = 'w';
                                                }
                                                if p1[i].is_present() {
                                                    flags[1] = 'p';
                                                }
                                                if p1[i].is_user() {
                                                    flags[2] = 'u';
                                                }
                                                log!(
                                                    "      {:#X}-{:#X} {}{}{} ->  {:#X}",
                                                    addr,
                                                    addr + 0x1000,
                                                    flags[0],
                                                    flags[1],
                                                    flags[2],
                                                    p1[i].addr()
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
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

    let p4 = unsafe { &*P4 };
    let page = Page::new_small_page(virt_addr);
    // log!("translate virt_addr {:#X}", virt_addr);
    let p3 = p4.next_table(page.p4_index()).unwrap();
    if p3[page.p3_index()].is_huge() {
        return p3[page.p3_index()].addr() + virt_addr & 0x3fff_ffff;
    }
    let p2 = p3.next_table(page.p3_index()).unwrap();
    if p2[page.p2_index()].is_huge() {
        return p2[page.p2_index()].addr() + virt_addr & 0x001f_ffff;
    }
    let p1 = p2.next_table(page.p2_index()).unwrap();
    let addr = p1[page.p1_index()].addr();
    return addr + (virt_addr & 0x0fff);
}

pub fn map_user<A>(page: Page, frame: Frame, allocator: &mut A) -> &mut PageTableEntry
where
    A: FrameAllocator,
{
    assert!(
        page.size == PageSize::Small,
        "UNIMPLEMENTED: only 4kb mapping is support currently"
    );
    let p4 = unsafe { &mut *P4 };
    let p3 = p4.next_table_create(page.p4_index(), true, allocator);
    let p2 = p3.next_table_create(page.p3_index(), true, allocator);
    let p1 = p2.next_table_create(page.p2_index(), true, allocator);

    assert!(p1[page.p1_index()].is_unused());
    let entry = p1[page.p1_index()]
        .set_addr(frame.addr)
        .set_present(true)
        .set_user(true);

    entry
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
    let p3 = p4.next_table_create(page.p4_index(), false, allocator);
    let p2 = p3.next_table_create(page.p3_index(), false, allocator);
    let p1 = p2.next_table_create(page.p2_index(), false, allocator);

    assert!(p1[page.p1_index()].is_unused());
    let entry = p1[page.p1_index()].set_addr(frame.addr).set_present(true);

    entry
}

#[allow(dead_code)]
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

#[allow(unused)]
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

#[allow(unused)]
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
