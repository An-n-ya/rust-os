#![allow(unused)]
pub struct Port {
    port: u16,
}

impl Port {
    pub const fn new(port: u16) -> Self {
        Self { port }
    }
    pub fn write_u8(&self, value: u8) {
        unsafe {
            core::arch::asm!("out dx, al", in("dx") self.port, in("al") value, options(nomem, preserves_flags, nostack) )
        }
    }
    pub fn write_u16(&self, value: u16) {
        unsafe {
            core::arch::asm!("out dx, ax", in("dx") self.port, in("ax") value, options(nomem, preserves_flags, nostack) )
        }
    }
    pub fn write_u32(&self, value: u32) {
        unsafe {
            core::arch::asm!("out dx, eax", in("dx") self.port, in("eax") value, options(nomem, preserves_flags, nostack) )
        }
    }
    pub fn write_u64(&self, value: u64) {
        unsafe {
            core::arch::asm!("out dx, rax", in("dx") self.port, in("rax") value, options(nomem, preserves_flags, nostack) )
        }
    }
    pub fn read_u8(&self) -> u8 {
        let res: u8;
        unsafe {
            core::arch::asm!("in al, dx", in("dx") self.port, out("al") res, options(nomem, preserves_flags, nostack))
        }
        res
    }
    pub fn read_u16(&self) -> u16 {
        let res: u16;
        unsafe {
            core::arch::asm!("in ax, dx", in("dx") self.port, out("ax") res, options(nomem, preserves_flags, nostack))
        }
        res
    }
    pub fn read_u32(&self) -> u32 {
        let res: u32;
        unsafe {
            core::arch::asm!("in eax, dx", in("dx") self.port, out("eax") res, options(nomem, preserves_flags, nostack))
        }
        res
    }
    pub fn read_u32_to(&self, addr: *const u32, cnt: usize) {
        unsafe {
            core::arch::asm!(
                "cld
                rep insl",
                in("dx") self.port,
                inout("edi") addr => _,
                inout("ecx") cnt => _,
                options(nostack, preserves_flags, att_syntax), // we have to use att because of the rep inst
            )
        }
    }
}
