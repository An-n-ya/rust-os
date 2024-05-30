use self::idt::init_idt;
use self::pic::PIC;

mod handler;
pub mod idt;
mod pic;

pub fn run_without_interrupt<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    if is_enable() {
        disable();
    }
    let res = f();

    if !is_enable() {
        enable();
    }

    res
}

pub fn is_enable() -> bool {
    let rflags: u64;

    unsafe {
        core::arch::asm!("pushfq; pop {}", out(reg) rflags, options(nomem, preserves_flags));
    }

    if (rflags & 1 << 9) != 0 {
        true
    } else {
        false
    }
}

pub fn disable() {
    unsafe {
        core::arch::asm!("cli", options(nostack, preserves_flags));
    }
}
pub fn enable() {
    unsafe {
        core::arch::asm!("sti", options(nostack, preserves_flags));
    }
}

#[allow(dead_code)]
pub fn divide_by_zero() {
    unsafe { core::arch::asm!("mov edx, 0; div edx") }
}

pub fn init() {
    disable();
    init_idt();
    PIC.init();
    enable();
}
