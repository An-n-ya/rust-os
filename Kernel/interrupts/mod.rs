use self::idt::init_idt;
use self::pic::PIC;

mod handler;
pub mod idt;
mod pic;

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
    init_idt();
    PIC.init();
    enable();
}
