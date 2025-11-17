// VGA Text Mode Driver
// ====================
// This module implements a VGA text mode driver for displaying text on screen.
// The VGA text buffer is a special memory region at address 0xb8000 that
// directly maps to what's displayed on screen.
//
// Study Notes:
// - VGA (Video Graphics Array) is a display hardware standard
// - Text mode allows displaying ASCII characters in a grid (typically 80x25)
// - Each character on screen requires 2 bytes: character byte + attribute byte
// - The attribute byte contains color information (foreground and background)

use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

// STUDY NOTE: We use lazy_static! to create a global writer instance.
// This is necessary because Rust statics require compile-time initialization,
// but we need runtime initialization for complex types like Mutex.
lazy_static! {
    /// Global writer instance protected by a spinlock mutex
    /// A spinlock doesn't put the thread to sleep - it keeps checking in a loop
    /// This is important in kernel code where we don't have thread scheduling yet
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

// =============================================================================
// COLOR HANDLING
// =============================================================================

/// VGA color palette
/// STUDY NOTE: We use #[repr(u8)] to ensure each enum variant is stored as u8
/// This is crucial for memory layout compatibility with VGA hardware
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// Represents a full color code (foreground + background)
/// STUDY NOTE: The color byte format is:
/// - Bits 0-3: Foreground color
/// - Bits 4-6: Background color
/// - Bit 7: Blink bit (we don't use this)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]  // Ensures ColorCode has same memory layout as u8
struct ColorCode(u8);

impl ColorCode {
    /// Creates a new ColorCode from foreground and background colors
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

// =============================================================================
// BUFFER STRUCTURE
// =============================================================================

/// Represents a single character on the screen
/// STUDY NOTE: #[repr(C)] ensures the struct has the same memory layout
/// as it would in C, which is important for hardware compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

/// The height of the VGA text buffer (number of rows)
const BUFFER_HEIGHT: usize = 25;
/// The width of the VGA text buffer (number of columns)
const BUFFER_WIDTH: usize = 80;

/// Represents the VGA text buffer
/// STUDY NOTE: We use Volatile to prevent compiler optimizations.
/// Without volatile, the compiler might optimize away repeated writes
/// thinking they're redundant, but we need every write to reach the hardware.
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// =============================================================================
// WRITER IMPLEMENTATION
// =============================================================================

/// A writer that can write ASCII bytes and strings to the VGA text buffer
pub struct Writer {
    column_position: usize,  // Current column position (0-79)
    color_code: ColorCode,   // Current color configuration
    buffer: &'static mut Buffer,  // Reference to the VGA buffer
}

impl Writer {
    /// Writes a single ASCII byte to the buffer
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),  // Handle newline character
            byte => {
                // Check if we need to wrap to the next line
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;  // Always write to the last row
                let col = self.column_position;

                let color_code = self.color_code;

                // STUDY NOTE: We write using Volatile to ensure the write
                // actually happens and isn't optimized away
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    /// Writes a string to the buffer
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // Printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // Not part of printable ASCII range
                // STUDY NOTE: VGA text mode only supports ASCII, not full UTF-8
                // We display a ■ character for unsupported bytes
                _ => self.write_byte(0xfe),
            }
        }
    }

    /// Shifts all lines up by one and clears the last line
    fn new_line(&mut self) {
        // STUDY NOTE: When we reach the bottom of the screen, we need to scroll
        // This is done by copying each row to the row above it
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    /// Clears a row by filling it with blank characters
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

// =============================================================================
// FORMATTING SUPPORT
// =============================================================================

// STUDY NOTE: Implementing fmt::Write allows us to use write! macro
// This is how we enable formatted output like numbers, hex values, etc.
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// =============================================================================
// PRINT MACROS
// =============================================================================

/// Like the standard `print!` macro, but prints to the VGA text buffer
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

/// Like the standard `println!` macro, but prints to the VGA text buffer
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Prints the given formatted string to the VGA text buffer
/// through the global `WRITER` instance.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    // STUDY NOTE: We use interrupt::without_interrupts to prevent deadlocks
    // If an interrupt occurs while holding the lock and tries to print,
    // it would deadlock. We'll add this protection later when we handle interrupts.
    WRITER.lock().write_fmt(args).unwrap();
}

// =============================================================================
// TESTING
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_println_simple() {
        println!("test_println_simple output");
    }

    #[test_case]
    fn test_println_many() {
        for _ in 0..200 {
            println!("test_println_many output");
        }
    }

    #[test_case]
    fn test_println_output() {
        let s = "Some test string that fits on a single line";
        println!("{}", s);
        for (i, c) in s.chars().enumerate() {
            let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    }
}