/*
 * Rust BareBones OS
 * - By John Hodge (Mutabah/thePowersGang)
 *
 * macros.rs
 * - Macros used by the kernel
 *
 * This code has been put into the public domain, there are no restrictions on
 * its use, and the author takes no liability.
 */

/// A very primitive logging macro
///
/// Obtaines a logger instance (locking the log channel) with the current module name passed
/// then passes the standard format! arguments to it
macro_rules! log{
	( $($arg:tt)* ) => ({
		// Import the Writer trait (required by write!)
		use core::fmt::Write;
        let mut writer = ::logging::WRITER.lock();
		let _ = write!(&mut *(writer), "[");
		let _ = write!(&mut *(writer), module_path!());
		let _ = write!(&mut *(writer), "] ");
		let _ = write!(&mut *(writer), $($arg)*);
		let _ = write!(&mut *(writer), "\n");
	})
}

macro_rules! print {
    ($($arg:tt)*) => {
        #[allow(unused_unsafe)]
        unsafe {
            use $crate::vga::TerminalWriter;
            use $crate::interrupts;
            use core::fmt::Write as FmtWrite;
            interrupts::run_without_interrupt(|| {
                let writer = &$crate::vga::TERMINAL_WRITER as *const TerminalWriter;
                // write_fmt needs writer as &mut, but we only access it as *const. Cast to fulfil the
                // API requirements
                let writer = writer as *mut TerminalWriter;
                #[allow(invalid_reference_casting)]
                write!(&mut *(writer), $($arg)*).expect("Failed to print");
            });
        }
    }
}

macro_rules! println {
    ($($arg:tt)*) => {
        print!($($arg)*);
        print!("\n");
    }
}
