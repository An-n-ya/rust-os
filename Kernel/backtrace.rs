use crate::KERNEL_BASE;

const BACKTRACE_MAX_DEPTH: usize = 10;
pub fn backtrace(mut frame_pointer: *const u64) {
    for _ in 0..BACKTRACE_MAX_DEPTH {
        unsafe {
            // log!("frame_pointer: {:#X}", *frame_pointer);
            // frame_pointer = *frame_pointer.offset(-1) as *const u64;
            // frame_pointer = frame_pointer.add(KERNEL_BASE as usize);
            // log!("frame_pointer: {:#X}", frame_pointer as u64);
            if frame_pointer.is_null() || *frame_pointer == 0 {
                break;
            }
            // log!("frame_pointer: {:#X}", frame_pointer as u64);
            // log!("value: {:#X}", *frame_pointer);
            let return_address = *frame_pointer.add(1);
            log!("BACKTRACE return_address: {:#X}", return_address);
            frame_pointer = *frame_pointer as *const u64;
        }
    }
}
