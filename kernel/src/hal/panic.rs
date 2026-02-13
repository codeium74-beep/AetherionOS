/// Aetherion OS - Panic Handler
/// Sophisticated panic handling with ACHA integration and debugging info

use core::panic::PanicInfo;
use x86_64::instructions::hlt;

/// Kernel panic handler
/// 
/// Called when the kernel encounters an unrecoverable error.
/// Displays detailed information and halts the system.
pub fn panic(info: &PanicInfo) -> ! {
    // Log to ACHA if available
    crate::acha::events::log_event(crate::acha::events::CognitiveEvent::KernelPanic);
    
    crate::serial_println!("\n\n╔════════════════════════════════════════════════╗");
    crate::serial_println!("║          KERNEL PANIC DETECTED                 ║");
    crate::serial_println!("╚════════════════════════════════════════════════╝\n");

    // Message
    if let Some(message) = info.message() {
        crate::serial_println!("Message: {}", message);
    }

    // Location
    if let Some(location) = info.location() {
        crate::serial_println!("Location: {}:{}:{}", 
            location.file(),
            location.line(),
            location.column()
        );
    }

    // Display metrics if available
    if let Some((interrupts, page_faults, exceptions, uptime)) = crate::acha::metrics::get_metrics() {
        crate::serial_println!("\nSystem Metrics:");
        crate::serial_println!("  Interrupts: {}", interrupts);
        crate::serial_println!("  Page Faults: {}", page_faults);
        crate::serial_println!("  Exceptions: {}", exceptions);
        crate::serial_println!("  Uptime Ticks: {}", uptime);
    }

    // Stack trace (simplified - full unwinding requires more infrastructure)
    crate::serial_println!("\nStack trace:");
    crate::serial_println!("  <full stack unwinding not yet implemented>");

    crate::serial_println!("\n╔════════════════════════════════════════════════╗");
    crate::serial_println!("║  System Halted - Please reboot                 ║");
    crate::serial_println!("╚════════════════════════════════════════════════╝");

    // Halt forever
    loop {
        hlt();
    }
}
