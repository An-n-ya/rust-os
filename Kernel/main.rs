/*
 * Rust BareBones OS
 * - By John Hodge (Mutabah/thePowersGang)
 *
 * main.rs
 * - Top-level file for kernel
 *
 * This code has been put into the public domain, there are no restrictions on
 * its use, and the author takes no liability.
 */
#![feature(panic_info_message)] //< Panic handling
#![feature(str_from_raw_parts)]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]
#![feature(naked_functions)]
#![no_std] //< Kernels can't use std
#![crate_name = "kernel"]

use core::str::from_raw_parts;

#[allow(unused_imports)]
use interrupts::divide_by_zero;
use memory::{frame::Allocator, gdt, read_page, virt_to_physical};
use proc::{exec, user_space_prog_1};
use vga::TerminalWriter;

/// Macros, need to be loaded before everything else due to how rust parses
#[macro_use]
mod macros;

// Achitecture-specific modules
#[cfg(target_arch = "x86_64")]
#[path = "arch/amd64/mod.rs"]
pub mod arch;
#[cfg(target_arch = "x86")]
#[path = "arch/x86/mod.rs"]
pub mod arch;

/// Exception handling (panic)
pub mod unwind;

/// Logging code
mod logging;

/// vga
mod vga;

mod interrupts;

mod backtrace;

mod port;

mod memory;

mod proc;

pub const KERNEL_BASE: u64 = 0xFFFFFFFF80000000;

extern "C" {
    static kernel_end: u8;
    static kernel_start: u8;
}

extern crate alloc;

#[repr(C, packed)]
pub struct MultibootInfo {
    /* Multiboot info version number */
    flags: u32,

    /* Available memory from BIOS */
    mem_lower: u32,
    mem_upper: u32,

    /* "root" partition */
    boot_device: u32,

    /* Kernel command line */
    cmdline: u32,

    /* Boot-Module list */
    mods_count: u32,
    mods_addr: u32,

    dummy: [u8; 16],

    /* Memory Mapping buffer */
    mmap_length: u32,
    mmap_addr: u32,

    /* Drive Info buffer */
    drives_length: u32,
    drives_addr: u32,

    /* ROM configuration table */
    config_table: u32,

    /* Boot Loader Name */
    boot_loader_name: *const u8,

    /* APM table */
    apm_table: u32,
}

#[repr(C, packed)]
struct MultibootMmapEntry {
    size: u32,
    addr: u64,
    len: u64,
    typ: u32,
}

// Kernel entrypoint (called by arch/<foo>/start.S)
#[no_mangle]
pub unsafe extern "C" fn kmain(_multiboot_magic: u64, _info: *const MultibootInfo) -> ! {
    let _multiboot_magic = _multiboot_magic as u32;
    assert_eq!(_multiboot_magic, 0x2BADB002);
    init();
    println!("hello world!");

    // *(0xDEADBEAF as *mut u64) = 100;
    // divide_by_zero();

    let end_addr = &kernel_end as *const u8 as u64;
    let start_addr = &kernel_start as *const u8 as u64;
    let start_addr = virt_to_physical(start_addr);
    let end_addr = virt_to_physical(end_addr);
    log!("kernel start: {:#X}", start_addr);
    log!("kernel end: {:#X}", end_addr);
    let mut allocator = Allocator::new(_info, (start_addr, end_addr));
    memory::init(&mut allocator);

    read_page();

    exec(user_space_prog_1 as *const () as u64, &mut allocator);

    // test_allocator(_info, (start_addr, end_addr));
    // test_map(&mut allocator);
    // print_boot_info(_info);
    hlt();
}

pub fn hlt() -> ! {
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack));
        }
    }
}

fn init() {
    TerminalWriter::init();
    interrupts::init();
    gdt::init();
    proc::init_syscalls();
}

#[allow(dead_code)]
fn print_boot_info(info: *const MultibootInfo) {
    unsafe {
        let ptr = (*info).boot_loader_name;
        let name = from_raw_parts(ptr.add(KERNEL_BASE as usize), 4);
        let flags = (*info).flags;
        let mem_lower = (*info).mem_lower;
        let mem_upper = (*info).mem_upper;
        let mmap_length = (*info).mmap_length;

        log!("flags {:#b}", flags);
        log!("mem_lower {:#X}", mem_lower);
        log!("mem_upper {:#X}", mem_upper);
        log!("mmap_length {}", mmap_length);
        log!("name {}", name);

        for i in 0..mmap_length {
            let ptr = ((*info).mmap_addr as u64
                + KERNEL_BASE
                + core::mem::size_of::<MultibootMmapEntry>() as u64 * i as u64)
                as *const MultibootMmapEntry;
            let len = (*ptr).len;
            let addr = (*ptr).addr;
            if len != 0 {
                log!("len: {:#X}, addr: {:#X}", len, addr);
            }
        }
    }
}
