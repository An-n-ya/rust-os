pub fn wrmsr(address: u64, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    unsafe {
        core::arch::asm!("wrmsr", in("ecx") address, in("eax") low, in("edx") high, options(nostack, preserves_flags))
    }
}
pub fn rdmsr(address: u64) -> u64 {
    let (high, low): (u32, u32);
    unsafe {
        core::arch::asm!("wrmsr", in("ecx") address, out("eax") low, out("edx") high, options(nostack, preserves_flags, nomem))
    }
    (high as u64) << 32 | low as u64
}
