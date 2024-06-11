use core::ptr::addr_of;

const STACK_SIZE: usize = 0x2000;
pub static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
pub static mut PRIV_TSS_STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

static mut TSS: TaskStateSegment = TaskStateSegment::new();
static mut GDT: GlobalDescriptorTable = GlobalDescriptorTable::new();

pub const CS_SEL_KERNEL: u16 = 1 << 3 | 0;
pub const DS_SEL_KERNEL: u16 = 2 << 3 | 0;
pub const TSS_SEL_USER: u16 = 3 << 3 | 0;
pub const DS_SEL_USER: u16 = 5 << 3 | 3;
pub const CS_SEL_USER: u16 = 6 << 3 | 3;

#[repr(C, packed(4))]
struct TaskStateSegment {
    reserved_1: u32,
    // use virtual address
    pub privilege_stack_table: [u64; 3],
    reserved_2: u64,
    pub interrupt_stack_table: [u64; 7],
    reserved_3: u64,
    reserved_4: u16,
    pub iomap_base: u16,
}

#[repr(transparent)]
struct GDTEntry(u64);

#[repr(transparent)]
struct GlobalDescriptorTable([GDTEntry; 7]);

impl TaskStateSegment {
    pub const fn new() -> Self {
        Self {
            privilege_stack_table: [0; 3],
            interrupt_stack_table: [0; 7],
            iomap_base: core::mem::size_of::<Self>() as u16,
            reserved_1: 0,
            reserved_2: 0,
            reserved_3: 0,
            reserved_4: 0,
        }
    }
}

impl GlobalDescriptorTable {
    pub const fn new() -> Self {
        Self([
            GDTEntry(0),
            GDTEntry(0x00209B00_00000000), // Kernel Code
            GDTEntry(0x00009300_00000000), // Kernel Data
            GDTEntry(0),                   // TSS low
            GDTEntry(0),                   // TSS high
            GDTEntry(0x0000F300_00000000), // User Data
            GDTEntry(0x0020FB00_00000000), // User Code
        ])
    }
    pub fn load_tss(&mut self, tss: *const TaskStateSegment) {
        let ptr = tss as u64;
        // refer to: https://wiki.osdev.org/Global_Descriptor_Table
        let mut low = 1 << 47; // present
        low |= (ptr & 0xff_ffff) << 16;
        low |= (ptr & 0xff00_0000) >> 24 << 56;
        low |= (core::mem::size_of::<TaskStateSegment>() - 1) as u64 & 0xffff;
        low |= 0b1001 << 40;
        let high = ptr >> 32;
        self.0[3] = GDTEntry(low);
        self.0[4] = GDTEntry(high);
    }
    pub fn load(&self) {
        #[repr(C, packed(2))]
        struct DescriptorTablePointer {
            limit: u16,
            base: u64,
        }
        let ptr = &DescriptorTablePointer {
            limit: (core::mem::size_of::<u64>() * 7 - 1) as u16,
            base: self as *const _ as u64,
        };
        unsafe {
            core::arch::asm!("lgdt [{}]", in(reg) ptr, options(readonly, nostack, preserves_flags));
        }
    }
}

pub fn init() {
    unsafe {
        TSS.interrupt_stack_table[0] = {
            let stack_end = STACK.as_ptr() as *const _ as usize + STACK_SIZE;
            stack_end as u64
        };
        TSS.privilege_stack_table[0] = {
            let stack_end = PRIV_TSS_STACK.as_ptr() as *const _ as usize + STACK_SIZE;
            stack_end as u64
        };
        GDT.load_tss(addr_of!(TSS));
        GDT.load();

        core::arch::asm!(
            "push {sel}",
            "lea {tmp}, [1f + rip]",
            "push {tmp}",
            "retfq",
            "1:",
            sel = in(reg) u64::from(CS_SEL_KERNEL),
            tmp = lateout(reg) _,
            options(preserves_flags)
        );
        core::arch::asm!(
            "mov ds, {0:x}", in(reg) DS_SEL_KERNEL, options(nostack, preserves_flags)
        );
        core::arch::asm!(
            "ltr {0:x}", in(reg) TSS_SEL_USER, options(nostack, preserves_flags)
        );
    }
}
