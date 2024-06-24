use core::fmt;

use super::handler::{
    bound_range_exceeded_interrupt, break_point_interrupt, disk_interrupt_handler,
    divide_zero_handler, double_fault_handler, general_protection_fault_handler,
    invalid_opcode_interrupt, invalid_tss_interrupt, non_maskable_interrupt, overflow_interrupt,
    page_fault_handler, segment_not_present_interrupt, stack_segment_fault_interrupt,
    timer_interrupt_handler,
};

/*
|15         3|2  1|   0|
| 	   INDEX|  TI| RPL|
 */
#[allow(unused)]
const KERNEL_CODE_SELECTOR: u16 = (1 << 3) + (0 << 2) + 0;
#[allow(unused)]
const KERNEL_DATA_SELECTOR: u16 = (2 << 3) + (0 << 2) + 0;
#[allow(unused)]
const USER_CODE_SELECTOR_64: u16 = (5 << 3) + (0 << 2) + 0;
#[allow(unused)]
const USER_DATA_SELECTOR_64: u16 = (6 << 3) + (0 << 2) + 0;

static mut IDT: Idt = Idt::new();

pub struct Idt([Entry; 256]);
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Entry {
    pointer_low: u16,
    gdt_selector: u16,
    options: EntryOptions,
    pointer_middle: u16,
    pointer_high: u32,
    reserved: u32,
}

#[derive(Clone, Copy)]
struct EntryOptions(u16);

#[repr(C)]
pub struct ExceptionFrame {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}

impl fmt::Debug for ExceptionFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{
    instruction_pointer: {:#X},
    code_segment: {:#X},
    cpu_flags: {:#X},
    stack_pointer: {:#X},
    stack_segment: {:#X},
}}",
            self.instruction_pointer,
            self.code_segment,
            self.cpu_flags,
            self.stack_pointer,
            self.stack_segment
        )
    }
}

pub type HandlerFunc = extern "x86-interrupt" fn(_: ExceptionFrame);
pub type HandlerFuncWithErr = extern "x86-interrupt" fn(_: ExceptionFrame, _: u64);

impl EntryOptions {
    fn new() -> Self {
        let mut options = Self::default();
        options.set_present(true).disable_interrupts(true);
        options
    }

    const fn default() -> Self {
        let options = 0 | 0b111 << 9;
        Self(options)
    }

    pub fn set_present(&mut self, present: bool) -> &mut Self {
        if present {
            self.0 |= 1 << 15;
        } else {
            self.0 &= !(1 << 15);
        }
        self
    }
    #[allow(unused)]
    pub fn disable_interrupts(&mut self, disable: bool) -> &mut Self {
        if !disable {
            self.0 |= 1 << 8;
        } else {
            self.0 &= !(1 << 8);
        }

        self
    }
    #[allow(unused)]
    pub fn set_privilege_level(&mut self, level: u16) -> &mut Self {
        assert!(level < 8);
        self.0 &= !(0b111 << 13);
        self.0 |= level << 13;
        self
    }
    #[allow(unused)]
    pub fn set_stack_index(&mut self, index: u16) -> &mut Self {
        assert!(index < 8);
        self.0 &= !(0b111);
        self.0 |= index;
        self
    }
}

impl Entry {
    pub fn new(handler_address: u64) -> Self {
        Entry {
            pointer_low: handler_address as u16,
            gdt_selector: KERNEL_CODE_SELECTOR,
            options: EntryOptions::new(),
            pointer_middle: (handler_address >> 16) as u16,
            pointer_high: (handler_address >> 32) as u32,
            reserved: 0,
        }
    }

    const fn default() -> Self {
        Entry {
            pointer_low: 0,
            gdt_selector: 0,
            options: EntryOptions::default(),
            pointer_middle: 0,
            pointer_high: 0,
            reserved: 0,
        }
    }
}

impl Idt {
    const fn new() -> Self {
        Self([Entry::default(); 256])
    }
    pub fn set_handler_with_errorcode(&mut self, entry: usize, handler: HandlerFuncWithErr) {
        self.0[entry] = Entry::new(handler as u64);
    }
    pub fn set_handler(&mut self, entry: usize, handler: HandlerFunc) {
        self.0[entry] = Entry::new(handler as u64);
    }
    pub fn load(&self) {
        #[derive(Debug)]
        #[repr(C, packed(2))]
        struct DescriptorTablePointer {
            limit: u16,
            base: u64,
        }
        let ptr = &DescriptorTablePointer {
            limit: (core::mem::size_of::<Self>() - 1) as u16,
            base: self as *const _ as u64,
        };
        unsafe {
            core::arch::asm!("lidt [{}]", in(reg) ptr, options(readonly, nostack, preserves_flags));
        }
    }
}

pub fn init_idt() {
    unsafe {
        IDT.set_handler(0x0, divide_zero_handler);
        IDT.set_handler(0x2, non_maskable_interrupt);
        IDT.set_handler(0x3, break_point_interrupt);
        IDT.set_handler(0x4, overflow_interrupt);
        IDT.set_handler(0x5, bound_range_exceeded_interrupt);
        IDT.set_handler(0x6, invalid_opcode_interrupt);
        IDT.set_handler_with_errorcode(0x08, double_fault_handler);
        IDT.set_handler_with_errorcode(0x09, invalid_tss_interrupt);
        IDT.set_handler_with_errorcode(0x0A, segment_not_present_interrupt);
        IDT.set_handler_with_errorcode(0x0B, stack_segment_fault_interrupt);
        IDT.set_handler_with_errorcode(0x0E, page_fault_handler);
        IDT.set_handler_with_errorcode(0x0D, general_protection_fault_handler);
        IDT.set_handler(0x20, timer_interrupt_handler);
        IDT.set_handler(0x2e, disk_interrupt_handler);
        IDT.load();
    }
}
