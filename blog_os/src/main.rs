// Blog OS - A minimal operating system in Rust
// ============================================
// This is the entry point of our kernel. It demonstrates VGA text mode
// output using a proper abstraction layer instead of raw memory writes.

#![no_std]  // Don't link the Rust standard library
#![no_main] // Disable all Rust-level entry points

// Module declarations
mod vga_buffer;

use core::panic::PanicInfo;

/// The kernel entry point
///
/// STUDY NOTE: This function is called by the bootloader after setting up
/// a 64-bit environment. The `_start` name is the default entry point name
/// that linkers look for. We use `no_mangle` to prevent Rust from changing
/// the function name during compilation (name mangling).
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // STUDY NOTE: Instead of writing directly to the VGA buffer memory,
    // we now use our abstraction layer which provides:
    // - Type safety
    // - Automatic scrolling
    // - Proper volatile writes
    // - Color support

    println!("Hello World{}", "!");
    println!("Welcome to Blog OS");
    println!();

    // Demonstrate different types of output
    println!("Numbers: {} hex: 0x{:X}", 42, 0xDEADBEEFu32);
    println!("Booleans: {} {}", true, false);

    // Show some system information
    println!();
    println!("=== System Information ===");
    println!("VGA Buffer Address: 0x{:X}", 0xb8000);
    println!("Screen Size: {}x{} characters", 80, 25);

    // Test scrolling by printing many lines
    println!();
    println!("Testing scrolling:");
    for i in 0..5 {
        println!("  Line {}", i);
    }

    // Demonstrate panic message (commented out to keep system running)
    // panic!("Some panic message");

    // Kernel main loop - the kernel should never return
    println!();
    println!("Kernel initialized successfully!");
    println!("System is now running...");

    loop {
        // In a real OS, this is where we would:
        // - Handle interrupts
        // - Schedule processes
        // - Manage resources
        // For now, we just loop forever
    }
}

/// This function is called on panic
///
/// STUDY NOTE: When something goes wrong in the kernel (array out of bounds,
/// assertion failure, explicit panic, etc.), this handler is called.
/// We print the panic information to help with debugging.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("\n=== KERNEL PANIC ===");
    println!("{}", info);
    println!("===================");

    // Halt the CPU by looping forever
    // In a real OS, we might try to save state or reboot
    loop {}
}
