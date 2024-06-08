use crate::{hlt, interrupts::pic::PIC};

use super::idt::ExceptionFrame;
pub extern "x86-interrupt" fn divide_zero_handler(frame: ExceptionFrame) {
    log!("EXCEPTION: DIVIDE BY ZERO");
    log!("EXCEPTION MESSAGE: {frame:#?}");
    hlt();
}

pub extern "x86-interrupt" fn non_maskable_interrupt(frame: ExceptionFrame) {
    log!("EXCEPTION: NMI FAULT");
    log!("EXCEPTION MESSAGE: {frame:#?}");
    hlt()
}
pub extern "x86-interrupt" fn break_point_interrupt(frame: ExceptionFrame) {
    log!("EXCEPTION: BREAKPOINT FAULT");
    log!("EXCEPTION MESSAGE: {frame:#?}");
    hlt()
}
pub extern "x86-interrupt" fn overflow_interrupt(frame: ExceptionFrame) {
    log!("EXCEPTION: OVERFLOW FAULT");
    log!("EXCEPTION MESSAGE: {frame:#?}");
    hlt()
}
pub extern "x86-interrupt" fn bound_range_exceeded_interrupt(frame: ExceptionFrame) {
    log!("EXCEPTION: BOUND_RANGE_EXCEEDED FAULT");
    log!("EXCEPTION MESSAGE: {frame:#?}");
    hlt()
}
pub extern "x86-interrupt" fn invalid_opcode_interrupt(frame: ExceptionFrame) {
    log!("EXCEPTION: INVALID OPCODE FAULT");
    log!("EXCEPTION MESSAGE: {frame:#?}");
    hlt()
}
pub extern "x86-interrupt" fn invalid_tss_interrupt(frame: ExceptionFrame, error_code: u64) {
    log!("errorcode: {}", error_code);
    log!("EXCEPTION: INVALID TSS FAULT");
    log!("EXCEPTION MESSAGE: {frame:#?}");
    hlt()
}
pub extern "x86-interrupt" fn segment_not_present_interrupt(
    frame: ExceptionFrame,
    error_code: u64,
) {
    log!("errorcode: {}", error_code);
    log!("EXCEPTION: SEGMENT_NOT_PRESENT FAULT");
    log!("EXCEPTION MESSAGE: {frame:#?}");
    hlt()
}
pub extern "x86-interrupt" fn stack_segment_fault_interrupt(
    frame: ExceptionFrame,
    error_code: u64,
) {
    log!("errorcode: {}", error_code);
    log!("EXCEPTION: STACK_SEGMENT_FAULT FAULT");
    log!("EXCEPTION MESSAGE: {frame:#?}");
    hlt()
}

pub extern "x86-interrupt" fn page_fault_handler(frame: ExceptionFrame, error_code: u64) {
    log!("errorcode: {}", error_code);
    let cr2: u64;
    unsafe {
        core::arch::asm!("mov {}, cr2", out(reg) cr2, options(nomem, nostack, preserves_flags));
    }
    log!("page fault handler missing");
    log!("trying to access addr {:#X}", cr2);
    log!("EXCEPTION MESSAGE: {frame:#?}");
    hlt();
}

pub extern "x86-interrupt" fn double_fault_handler(frame: ExceptionFrame, error_code: u64) {
    // the error_code is related to segment fault, which is quite useless
    hlt();
    log!("EXCEPTION: DOUBLE FAULT, errorcode: {error_code}");
    log!("EXCEPTION MESSAGE: {frame:#?}");
    hlt();
}

pub extern "x86-interrupt" fn timer_interrupt_handler(_frame: ExceptionFrame) {
    print!(".");
    PIC.eof(0x20);
}
pub extern "x86-interrupt" fn general_protection_fault_handler(
    frame: ExceptionFrame,
    error_code: u64,
) {
    log!("error code {}", error_code);
    log!("protection fault occur!");
    log!("EXCEPTION MESSAGE: {frame:#?}");
    hlt();
}
