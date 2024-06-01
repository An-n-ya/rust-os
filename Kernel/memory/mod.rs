use self::frame::PageSize;

use self::frame::Allocator;

use self::page_table::PageTable;

use crate::{MultibootInfo, KERNEL_BASE};

mod frame;
mod page_table;

pub fn read_page() {
    let cr3: u64;
    unsafe {
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack, preserves_flags));
    }
    log!("cr3: {cr3:#X}");
    let pml4 = cr3 + KERNEL_BASE;
    let page_table_4 = pml4 as *const PageTable;
    for (i, entry) in unsafe { (*page_table_4).entries.iter().enumerate() } {
        if !entry.is_unused() {
            log!("pml4 entry {} -> physical addr {:#X}", i, entry.addr());
            if entry.addr() != cr3 {
                let pdpt = entry.addr() + KERNEL_BASE;
                let pdpt = pdpt as *const PageTable;
                for (i, entry) in unsafe { (*pdpt).entries.iter().enumerate() } {
                    if !entry.is_unused() {
                        log!("  pdpt entry {} -> physical addr {:#X}", i, entry.addr());
                    }
                }
            }
        }
    }

    // let virt_addr = KERNEL_BASE + 0xb8000;
    // let physical_addr = virt_to_physical(virt_addr);
    // log!(
    //     "virt_addr {:#X} -> physical_addr {:#x}",
    //     virt_addr,
    //     physical_addr
    // );
}

pub fn virt_to_physical(virt_addr: u64) -> u64 {
    let cr3: u64;
    unsafe {
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack, preserves_flags));
    }
    let pml4 = cr3 + KERNEL_BASE;
    let mut table = pml4 as *const PageTable;
    let page_index = [
        (virt_addr & 0o777 << 39) >> 39,
        (virt_addr & 0o777 << 30) >> 30,
        (virt_addr & 0o777 << 21) >> 21,
        (virt_addr & 0o777 << 12) >> 12,
    ];
    for (i, index) in page_index.iter().enumerate() {
        // log!("table addr {:#X}", table as u64);
        let entry = unsafe { &(*table).entries[*index as usize] };
        if (i == 1 || i == 2) && entry.is_huge() {
            return entry.addr() + virt_addr & if i == 1 { 0x3fff_ffff } else { 0x001f_ffff };
        }

        table = (entry.addr() + KERNEL_BASE) as *const PageTable;
        if i == 3 {
            return entry.addr() + virt_addr & 0x0fff;
        }
    }
    unreachable!()
}

pub fn map_page(virt_addr: u64, phy_addr: u64) {
    let page_index = [
        (virt_addr & 0o777 << 39) >> 39,
        (virt_addr & 0o777 << 30) >> 30,
        (virt_addr & 0o777 << 21) >> 21,
        (virt_addr & 0o777 << 12) >> 12,
    ];
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
