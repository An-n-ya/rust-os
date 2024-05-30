mod handler;
pub mod idt;


#[allow(dead_code)]
pub fn divide_by_zero() {
    unsafe { core::arch::asm!("mov edx, 0; div edx") }
}
