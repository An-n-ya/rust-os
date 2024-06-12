use core::fmt::Write;

use crate::{sync::spin::SpinMutex, KERNEL_BASE};

pub static TERMINAL_WRITER: SpinMutex<TerminalWriter> = SpinMutex::new(TerminalWriter::new());

/* Hardware text mode color constants. */
#[allow(dead_code)]
enum VgaColor {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGrey = 7,
    DarkGrey = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    LightBrown = 14,
    White = 15,
}

const fn vga_entry_color(fg: VgaColor, bg: VgaColor) -> u8 {
    fg as u8 | (bg as u8) << 4
}

const fn vga_entry(uc: u8, color: u8) -> u16 {
    uc as u16 | (color as u16) << 8
}

const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;

pub struct TerminalWriter {
    terminal_pos: usize,
    terminal_color: u8,
    terminal_buffer: u64,
}

impl TerminalWriter {
    const fn new() -> TerminalWriter {
        let terminal_color = vga_entry_color(VgaColor::LightGrey, VgaColor::Black);
        let terminal_buffer = 0xB8000 + KERNEL_BASE;

        TerminalWriter {
            terminal_pos: 0,
            terminal_color,
            terminal_buffer,
        }
    }

    pub fn init() {
        let writer = TERMINAL_WRITER.lock();

        let color = writer.terminal_color;
        for y in 0..VGA_HEIGHT {
            for x in 0..VGA_WIDTH {
                let index = y * VGA_WIDTH + x;
                unsafe {
                    *(writer.terminal_buffer as *mut u16).add(index) = vga_entry(b' ', color);
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn set_color(&mut self, color: u8) {
        self.terminal_color = color;
    }

    fn putchar(&mut self, c: u8) {
        if c == b'\n' {
            let mut pos = self.terminal_pos;
            pos += VGA_WIDTH - (pos % VGA_WIDTH);
            self.terminal_pos = pos;
            return;
        }

        let color = self.terminal_color;
        let pos = self.terminal_pos;
        let new_pos = (pos + 1) % (VGA_HEIGHT * VGA_WIDTH);
        self.terminal_pos = new_pos;
        // self.terminal_pos.store(pos, Ordering::Relaxed);
        unsafe {
            *(self.terminal_buffer as *mut u16).add(pos) = vga_entry(c, color);
        }
    }

    fn write(&mut self, data: &[u8]) {
        for c in data {
            self.putchar(*c);
        }
    }
}

impl Write for TerminalWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

unsafe impl Sync for TerminalWriter {}
