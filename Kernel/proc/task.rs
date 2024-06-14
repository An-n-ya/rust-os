use core::{alloc::Layout, ptr::NonNull};

use alloc::boxed::Box;

use crate::{
    memory::{
        gdt::{CS_SEL_KERNEL, DS_SEL_KERNEL, TSS},
        page_table::{kernel_page_table, Level4, PageTable},
    },
    stack::Stack,
};

const KERNEL_STACK_SIZE: usize = 0x1000;

#[repr(C)]
struct Task {
    context: Context,
}

#[derive(Default)]
#[repr(C)]
struct Context {
    r15: usize,
    r14: usize,
    r13: usize,
    r12: usize,

    rbx: usize,
    rbp: usize,

    rflags: usize,
    rip: usize,
}

#[derive(Default)]
#[repr(C, packed)]
pub struct InterruptFrame {
    pub r15: usize,
    pub r14: usize,
    pub r13: usize,
    pub r12: usize,
    pub rbp: usize,
    pub rbx: usize,

    pub r11: usize,
    pub r10: usize,
    pub r9: usize,
    pub r8: usize,
    pub rsi: usize,
    pub rdi: usize,
    pub rdx: usize,
    pub rcx: usize,
    pub rax: usize,

    pub rip: usize,
    pub cs: usize,
    pub rflags: usize,
    pub rsp: usize,
    pub ss: usize,
}
#[naked]
extern "C" fn iretq_init() -> ! {
    unsafe {
        core::arch::asm!(
            "
            cli
            add rsp, 8
            pop r15
            pop r14
            pop r13
            pop r12
            pop rbp
            pop rbx

            pop r11
            pop r10 
            pop r9 
            pop r8 
            pop rsi 
            pop rdi 
            pop rdx 
            pop rcx
            pop rax

            iretq
            ",
            options(noreturn)
        )
    }
}

#[naked]
extern "C" fn context_switch(_prev: &mut Context, next: &Context) {
    unsafe {
        core::arch::asm!(
            "
        pushfq
        push rbp
        push rbx
        push r12
        push r13
        push r14
        push r15

        mov [rdi], rsp
        mov rsp, rsi

        pop r15
        pop r14
        pop r13
        pop r12
        pop rbx
        pop rbp
        popfq

        ret
        ",
            options(noreturn)
        )
    }
}

#[repr(C)]
pub struct X86Task {
    context: NonNull<Context>,
    kernel_stack: Box<[u8]>,
    page_table: NonNull<PageTable<Level4>>,
}
unsafe impl Sync for X86Task {}

pub fn x86_context_switch(prev: &mut X86Task, next: &X86Task) {
    unsafe {
        TSS.privilege_stack_table[0] = {
            let stack_end = next.kernel_stack.as_ptr() as *const _ as usize + KERNEL_STACK_SIZE;
            stack_end as u64
        };
        next.page_table.as_ref().enable();
        context_switch(prev.context.as_mut(), next.context.as_ref());
    }
}

impl X86Task {
    pub fn get_mut(&self) -> &mut X86Task {
        unsafe { &mut *(self as *const _ as *mut _) }
    }
    pub fn new_kernel(entry_point: u64) -> X86Task {
        let task_stack = unsafe {
            alloc::alloc::alloc_zeroed(Layout::from_size_align_unchecked(KERNEL_STACK_SIZE, 0x1000))
                .add(KERNEL_STACK_SIZE)
        };
        let switch_stack =
            unsafe { alloc::alloc::alloc_zeroed(Layout::from_size_align_unchecked(0x100, 0x1000)) };
        let mut stack_ptr = switch_stack as usize;
        let mut stack = Stack::new(&mut stack_ptr);

        let kframe = unsafe { stack.offset::<InterruptFrame>() };
        *kframe = InterruptFrame::default();
        kframe.ss = DS_SEL_KERNEL as usize;
        kframe.cs = CS_SEL_KERNEL as usize;
        kframe.rip = entry_point as usize;
        kframe.rsp = task_stack as usize;
        kframe.rflags = 0;

        let context = unsafe { stack.offset::<Context>() };
        *context = Context::default();
        context.rip = iretq_init as usize;

        Self {
            context: unsafe { NonNull::new_unchecked(context) },
            kernel_stack: alloc::vec![0u8; KERNEL_STACK_SIZE].into_boxed_slice(),
            page_table: unsafe { NonNull::new_unchecked(kernel_page_table()) },
        }
    }
}
