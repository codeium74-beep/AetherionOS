/// Aetherion OS - HAL Layer - UART Serial Driver
/// Phase 3: Serial port communication for debugging and logging
/// 
/// This module provides serial output functionality using the uart_16550 crate.
/// It implements the standard print!/println! macros for kernel debugging.

use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    /// UART Serial port (COM1 at 0x3F8)
    /// Wrapped in Mutex for safe concurrent access
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

/// Print formatted arguments to serial port
/// 
/// # Safety
/// Disables interrupts during printing to prevent deadlocks when printing
/// from interrupt handlers.
#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    // Disable interrupts to prevent deadlock if print is called from interrupt handler
    interrupts::without_interrupts(|| {
        SERIAL1
            .lock()
            .write_fmt(args)
            .expect("Printing to serial failed");
    });
}

/// Print to serial port (COM1) without newline
/// 
/// # Examples
/// ```
/// serial_print!("Boot stage: {}", stage_num);
/// ```
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::drivers::serial::_print(format_args!($($arg)*))
    };
}

/// Print to serial port (COM1) with newline
/// 
/// # Examples
/// ```
/// serial_println!("Kernel initialized");
/// serial_println!("Memory: {} MB", mem_size);
/// ```
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*
    ));
}

/// Initialize serial port
/// 
/// Forces initialization of SERIAL1 lazy_static, ensuring the port is
/// ready before any print operations.
pub fn init() {
    lazy_static::initialize(&SERIAL1);
    log::info!("UART serial port (COM1) initialized at 0x3F8");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serial_print() {
        serial_println!("UART test message");
        // If we reach here without panic, the test passed
    }
}
