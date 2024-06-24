use core::fmt;

use crate::sync::spin::SpinMutex;

/// A formatter object
pub struct Writer;

pub static WRITER: SpinMutex<Writer> = SpinMutex::new(Writer);

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // If the lock is owned by this instance, then we can safely write to the output
        unsafe {
            ::arch::debug::puts(s);
        }
        Ok(())
    }
}
