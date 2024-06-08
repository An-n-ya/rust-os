use alloc::vec;
use alloc::vec::Vec;

use crate::hlt;
use crate::interrupts::disable;
use crate::memory::gdt::{CS_SEL_USER, DS_SEL_USER};

use crate::memory::map_user;
// use crate::memory::gdt::set_usermode_segs;
use crate::{
    arch::instruction::{rdmsr, wrmsr},
    memory::{
        frame::{Frame, FrameAllocator},
        page_table::{flush_page_table, kernel_page_table, Page},
        read_page, virt_to_physical,
    },
};

const MSR_STAR: u64 = 0xC000_0081;
const MSR_LSTAR: u64 = 0xC000_0082;
const MSR_FMASK: u64 = 0xC000_0084;
const MSR_IA32_EFER: u64 = 0xC000_0081;
const MSR_STAR_VALUE: u64 = 0x23_0008_0000_0000;

#[naked]
#[allow(undefined_naked_function_abi)]
pub unsafe fn user_space_prog_1() {
    core::arch::asm!(
        "
        syscall
        nop
        nop
        nop
        ret
    ",
        options(noreturn)
    )
}
#[naked]
#[allow(undefined_naked_function_abi)]
fn handle_syscall() {
    unsafe {
        core::arch::asm!(
            "
        nop
        nop
        nop
        sysretq
        ",
            options(noreturn),
        )
    }
}

pub fn test_user_space<A>(allocator: &mut A)
where
    A: FrameAllocator,
{
    disable();
    let user_space_fn_in_kernel = user_space_prog_1 as *const () as u64;
    log!("fn addr {:#X}", user_space_fn_in_kernel);
    let user_space_fn_phys = virt_to_physical(user_space_fn_in_kernel);
    let page_phys_start = (user_space_fn_phys >> 12) << 12;
    let fn_page_offset = user_space_fn_phys - page_phys_start;
    let user_space_fn_virt_base = 0x40_0000;
    let user_space_fn_virt = user_space_fn_virt_base + fn_page_offset;
    log!(
        "mapping {:#X} to {:#X}, size: {:#X}",
        page_phys_start,
        user_space_fn_virt_base,
        fn_page_offset
    );
    let mut stack_space: Vec<u8> = vec![0; 1000];
    let ptr = stack_space.as_ptr() as *const u8 as u64;
    let stack_space_phys = virt_to_physical(ptr);
    let stack_page = Page::new_small_page(0x80_0000);
    let stack_phys_frame = Frame::new_small_page(stack_space_phys);

    let table = unsafe { &*kernel_page_table() };
    table.enable();
    let user_space_page = Page::new_small_page(user_space_fn_virt_base);
    let phys_frame = Frame::new_small_page(page_phys_start);
    for i in 0..5 {
        map_user(
            user_space_page.offset(0x1000 * i),
            phys_frame.offset(0x1000 * i),
            allocator,
        )
        .set_present(true);
    }

    map_user(stack_page, stack_phys_frame, allocator)
        .set_present(true)
        .set_writable(true);

    read_page();

    init_syscalls();
    jump_to_user_mode(user_space_fn_virt, 0x80_1000);
    // prevent rust from freeing stack early
    drop(stack_space);
    hlt();
}

fn init_syscalls() {
    wrmsr(MSR_STAR, MSR_STAR_VALUE);
    // enable system call extensions
    let mut val = rdmsr(MSR_IA32_EFER);
    val |= 1;
    wrmsr(MSR_IA32_EFER, val);
    wrmsr(MSR_FMASK, 0x200);
    let handler_addr = handle_syscall as *const () as u64;
    wrmsr(MSR_LSTAR, handler_addr);
}

#[no_mangle]
pub fn jump_to_user_mode(code: u64, stack_end: u64) {
    log!("code addr {:#X}", code);
    // let a = unsafe { *(code as *const u32) };
    // log!("a {}", a);
    // let f = code as *const ();
    // let f: extern "C" fn() = unsafe { core::mem::transmute(f) };
    // f();
    // loop {}
    unsafe {
        core::arch::asm!(
            "mov ds, {0:x}", in(reg) DS_SEL_USER, options(nostack, preserves_flags)
        );
    }
    // let (cs_idx, ds_idx) = unsafe { set_usermode_segs() };
    flush_page_table();
    let a = unsafe { *(0x80_0fff as *const u8) };
    log!("a {}", a);
    unsafe { *(0x80_0fff as *mut u8) = 1 };
    // loop {}
    unsafe {
        core::arch::asm!(
            "
            push rax
            push rsi
            push 0x200
            push rdx
            push rdi
        ",
        in("rax") DS_SEL_USER,
        in("rsi") stack_end,
        in("rdx") CS_SEL_USER,
        in("rdi") code,
        );
        core::arch::asm!("iretq")
    }
    // unsafe {
    //     core::arch::asm!(
    //         "
    //         push rax
    //         push rsi
    //         push 0x200
    //         push rdx
    //         push rdi
    //     ",
    //     in("rax") ds_idx,
    //     in("rsi") stack_end,
    //     in("rdx") cs_idx,
    //     in("rdi") code,
    //     );
    //     core::arch::asm!("iretq")
    // }
}
