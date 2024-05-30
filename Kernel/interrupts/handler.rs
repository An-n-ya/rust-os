use core::panic;

use super::idt::ExceptionFrame;

pub extern "x86-interrupt" fn page_fault_handler(frame: ExceptionFrame) -> ! {
    log!("page fault handler missing");
    log!("EXCEPTION MESSAGE: {frame:#?}");
    loop {}
}

pub extern "x86-interrupt" fn divide_zero_handler(frame: ExceptionFrame) -> ! {
    log!("EXCEPTION: DIVIDE BY ZERO");
    log!("EXCEPTION MESSAGE: {frame:#?}");
    loop {}
}
