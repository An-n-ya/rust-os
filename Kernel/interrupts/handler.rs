pub extern "C" fn page_fault_handler() -> ! {
    log!("page fault handler missing");
    loop {}
}

pub extern "C" fn divide_zero_handler() -> ! {
    log!("EXCEPTION: DIVIDE BY ZERO");
    loop {}
}
