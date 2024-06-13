#[repr(C)]
struct Task {
    context: Context,
}

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
