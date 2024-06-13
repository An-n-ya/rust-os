use alloc::vec;
use alloc::vec::Vec;

use crate::hlt;
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

mod task;

const MSR_STAR: u64 = 0xC000_0081;
const MSR_LSTAR: u64 = 0xC000_0082;
const MSR_FMASK: u64 = 0xC000_0084;
const MSR_IA32_EFER: u64 = 0xC000_0081;
const MSR_STAR_VALUE: u64 = 0x23_0010_0000_0000;

#[repr(C)]
struct SyscallFrame {
    rax: u64,
    rdi: u64,
    rsi: u64,
    rdx: u64,
    r10: u64,
}

#[naked]
#[allow(undefined_naked_function_abi)]
pub unsafe extern "C" fn user_space_prog_1() {
    core::arch::asm!(
        "
        mov rbx, 0xf0000000
        ",
        "3:", // we cannot use 1 or 0 as label, refer to: https://bugs.llvm.org/show_bug.cgi?id=36144
        "
        push 0x595ca11a
        mov rbp, 0x1
        mov rax, 0x2
        mov rcx, 0x3
        mov rdx, 0x4
        mov rdi, 0x5
        mov r8, 0x6
        mov r9, 0x7
        mov r10, 0x8
        mov r11, 0x9
        mov r12, 0xa
        mov r13, 0xb
        mov r14, 0xc
        mov r15, 0xd

        xor rax, rax
        2:
        inc rax
        cmp rax, 0x8000000
        jnz 2b


        pop rax
        inc rbx
        mov rdi, rsp
        mov rsi, rbx
        syscall",
        "jmp 3b",
        options(noreturn)
    )
}
#[naked]
#[allow(undefined_naked_function_abi)]
pub unsafe extern "C" fn user_space_prog_2() {
    core::arch::asm!(
        "
        mov rbx, 0x0
        ",
        "3:", // we cannot use 1 or 0 as label, refer to: https://bugs.llvm.org/show_bug.cgi?id=36144
        "
        push 0x595ca11b
        mov rbp, 0x11
        mov rax, 0x21
        mov rcx, 0x31
        mov rdx, 0x41
        mov rdi, 0x51
        mov r8, 0x61
        mov r9, 0x71
        mov r10, 0x81
        mov r11, 0x91
        mov r12, 0xa1
        mov r13, 0xb1
        mov r14, 0xc1
        mov r15, 0xd1

        xor rax, rax
        2:
        inc rax
        cmp rax, 0x8000000
        jnz 2b


        pop rax
        inc rbx
        mov rdi, rsp
        mov rsi, rbx
        syscall",
        "jmp 3b",
        options(noreturn)
    )
}

// #[allow(undefined_naked_function_abi)]
#[naked]
extern "C" fn handle_syscall() {
    unsafe {
        core::arch::asm!(
            "
            push rcx
            push r11
            push rbp
            push rbx
            push r12
            push r13
            push r14
            push r15
            mov rbp, rsp
            sub rsp, 0x400

            push r10
            push rdx
            push rsi
            push rdi
            push rax

            mov rdi, rsp

            cld
            call x64_handle_syscall
            cli

            pop rax
            pop rdi
            pop rsi
            pop rdx
            pop r10

        mov rax, rbx
        mov rsp, rbp
        pop r15
        pop r14
        pop r13
        pop r12
        pop rbx
        pop rbp
        pop r11
        pop rcx
        sysretq
        ",
            options(noreturn)
        )
    }
}

#[no_mangle]
fn x64_handle_syscall(frame: *mut SyscallFrame) -> u64 {
    let rax = unsafe { (*frame).rax };
    let rdi = unsafe { (*frame).rdi };
    let rsi = unsafe { (*frame).rsi };
    let rdx = unsafe { (*frame).rdx };
    let r10 = unsafe { (*frame).r10 };
    log!("syscall {:x} {:x} {:x} {:x} {:x}", rax, rdi, rsi, rdx, r10);
    0
}

pub fn exec<A>(user_space_fn_in_kernel: u64, allocator: &mut A)
where
    A: FrameAllocator,
{
    // disable();
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
    let stack_space: Vec<u8> = vec![0; 1000];
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

    for i in 0..5 {
        map_user(
            stack_page.offset(0x1000 * i),
            stack_phys_frame.offset(0x1000 * i),
            allocator,
        )
        .set_present(true)
        .set_writable(true);
    }

    read_page();

    jump_to_user_mode(user_space_fn_virt, 0x80_5000);
    // prevent rust from freeing stack early
    drop(stack_space);
    hlt();
}

pub fn init_syscalls() {
    log!("init syscallls");
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
    // log!("code addr {:#X}", code);
    unsafe {
        core::arch::asm!(
            "mov ds, {0:x}", in(reg) DS_SEL_USER, options(nostack, preserves_flags)
        );
    }
    flush_page_table();
    // unsafe { *(0x80_0fff as *mut u8) = 1 };
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
}
