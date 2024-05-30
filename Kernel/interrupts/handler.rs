use crate::interrupts::pic::PIC;

use super::idt::ExceptionFrame;

pub extern "x86-interrupt" fn page_fault_handler(frame: ExceptionFrame) {
    log!("page fault handler missing");
    log!("EXCEPTION MESSAGE: {frame:#?}");
    loop {}
}

pub extern "x86-interrupt" fn divide_zero_handler(frame: ExceptionFrame) {
    log!("EXCEPTION: DIVIDE BY ZERO");
    log!("EXCEPTION MESSAGE: {frame:#?}");
    loop {}
}

pub extern "x86-interrupt" fn double_fault_handler(frame: ExceptionFrame, error_code: u64) {
    // the error_code is related to segment fault, which is quite useless
    log!("EXCEPTION: DOUBLE FAULT, errorcode: {error_code}");
    log!("EXCEPTION MESSAGE: {frame:#?}");
    loop {}
}

pub extern "x86-interrupt" fn timer_interrupt_handler(_frame: ExceptionFrame) {
    print!(".");
    PIC.eof(0x20);
}
