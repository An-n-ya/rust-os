use alloc::vec::Vec;

use crate::{
    arch::instruction::{rdmsr, wrmsr},
    memory::{
        frame::{Frame, FrameAllocator},
        map,
        page_table::{Level4, Page, PageTable},
        virt_to_physical,
    },
};

const MSR_STAR: u64 = 0xC000_0081;
const MSR_IA32_EFER: u64 = 0xC000_0081;
const MSR_STAR_VALUE: u64 = 0x23_0008_0000_0000;

pub unsafe fn user_space_prog_1() {
    core::arch::asm!(
        "
        nop
        nop
        nop
    ",
        options(nostack, nomem, preserves_flags)
    );
}

#[no_mangle]
pub fn test_user_space<A>(allocator: &mut A)
where
    A: FrameAllocator,
{
    let user_space_fn_in_kernel = user_space_prog_1 as *const () as u64;
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
    let table: PageTable<Level4> = PageTable::kernel_page_table();
    table.enable();
    let user_space_page = Page::new_small_page(user_space_fn_virt_base);
    let phys_frame = Frame::new_small_page(page_phys_start);

    map(user_space_page, phys_frame, allocator)
        .set_present(true)
        .set_user(true);

    let stack_space: Vec<u8> = Vec::with_capacity(0x1000);
    let stack_space_phys = virt_to_physical(stack_space.as_ptr() as *const u8 as u64);
    let stack_page = Page::new_small_page(0x80_0000);
    let stack_phys_frame = Frame::new_small_page(stack_space_phys);

    map(stack_page, stack_phys_frame, allocator)
        .set_present(true)
        .set_user(true);

    jump_to_user_mode(user_space_fn_virt, 0x80_1000);
}

fn init_syscalls() {
    wrmsr(MSR_STAR, MSR_STAR_VALUE);
    // enable system call extensions
    let mut val = rdmsr(MSR_IA32_EFER);
    val |= 1;
    wrmsr(MSR_IA32_EFER, val);
}

#[no_mangle]
pub fn jump_to_user_mode(code: u64, stack_end: u64) {
    let cs_sel = (5 << 3) | 3;
    let ds_sel = (6 << 3) | 3;
    unsafe {
        core::arch::asm!(
            "
            push {stack_seg:r}
            push {stack}
            push 0x200
            push {code_seg:r}
            push {ret_addr}
            iretq
        ",
            ret_addr = in(reg) code,
            stack = in(reg) stack_end,
            stack_seg = in(reg) ds_sel,
            code_seg = in(reg) cs_sel
        )
    }
}
