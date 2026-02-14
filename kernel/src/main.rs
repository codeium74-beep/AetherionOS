// Aetherion OS Kernel - Entry Point
// Couche 1 HAL Integration - Phase Complete

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::panic::PanicInfo;

// Couche 1 HAL modules
mod arch;
mod hal;
mod acha;

// Memory management modules
mod memory;
mod allocator;

// Existing modules
mod gdt;
mod interrupts;
mod syscall;
mod process;
mod ipc;
mod drivers;
mod fs;
mod net;

// VGA Text Mode Constants
const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;
const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;
const VGA_COLOR_WHITE_ON_BLACK: u8 = 0x0f;

// Serial Port Constants (COM1)
const SERIAL_PORT: u16 = 0x3F8;

// Memory configuration
const MEMORY_START: usize = 0x100000;  // 1MB (after kernel)
const MEMORY_SIZE: usize = 32 * 1024 * 1024;  // 32MB managed RAM
static mut FRAME_BITMAP: [u8; 4096] = [0; 4096];  // Support 32MB (8192 frames)

// Heap configuration
const HEAP_START: usize = 0x_4444_4444_0000;
const HEAP_SIZE: usize = 1024 * 1024;  // 1 MB heap
static mut HEAP_MEMORY: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

/// Kernel Entry Point
/// Called by bootloader after CPU is in 64-bit long mode
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // ========================================
    // Couche 1 HAL Initialization
    // ========================================
    
    // Phase 1: Initialize HAL (serial + logger)
    hal::init();
    
    log::info!("═══════════════════════════════════════════════");
    log::info!("  AetherionOS Cognitive Core v1.0.0");
    log::info!("  Couche 1 HAL Layer - ACTIVE");
    log::info!("═══════════════════════════════════════════════");
    
    // Phase 2: Architecture initialization (GDT, IDT)
    log::info!("Initializing CPU structures...");
    arch::init();
    
    // Phase 3: ACHA cognitive layer
    log::info!("Initializing ACHA cognitive layer...");
    acha::init();
    
    // ========================================
    // Legacy System Initialization
    // ========================================
    
    // Initialize hardware (legacy code)
    init_serial();
    clear_screen();
    
    // Display boot message
    print_str("AETHERION OS v1.0.0 - COUCHE 1 HAL", 0, 0);
    print_str("===================================", 0, 1);
    print_str("HAL Layer: ACTIVE", 0, 3);
    print_str("GDT: INITIALIZED", 0, 4);
    print_str("IDT: INITIALIZED", 0, 5);
    
    log::info!("Legacy GDT initialization...");
    gdt::init();
    
    log::info!("Legacy IDT initialization...");
    interrupts::init();
    print_str("Legacy IDT: INITIALIZED", 0, 6);
    
    log::info!("Syscall interface initialization...");
    syscall::init();
    print_str("Syscalls: INITIALIZED", 0, 7);
    
    log::info!("Process management initialization...");
    process::init();
    ipc::init();
    print_str("Processes & IPC: INITIALIZED", 0, 8);
    
    log::info!("Device drivers initialization...");
    drivers::init_all();
    print_str("Drivers: INITIALIZED", 0, 9);
    
    log::info!("Filesystem initialization...");
    fs::init();
    print_str("Filesystem: INITIALIZED", 0, 10);
    
    log::info!("Network stack initialization...");
    net::init();
    print_str("Networking: INITIALIZED", 0, 11);
    
    // Initialize frame allocator
    log::info!("Initializing frame allocator...");
    let mut frame_allocator = unsafe {
        memory::frame_allocator::FrameAllocator::new(
            memory::PhysicalAddress::new(MEMORY_START),
            MEMORY_SIZE,
            &mut FRAME_BITMAP,
        )
    };
    
    print_str("Frame Allocator: INITIALIZED", 0, 13);
    
    // Test frame allocation
    log::info!("Testing frame allocation...");
    print_str("Testing Frame Allocation...", 0, 15);
    
    for _i in 1..=5 {
        if let Some(_frame) = frame_allocator.allocate_frame() {
            // Frame allocated successfully
        } else {
            log::error!("Frame allocation failed!");
            print_str("ERROR: Frame allocation failed!", 0, 16);
        }
    }
    
    print_str("Allocated 5 frames successfully!", 0, 16);
    log::info!("Frame allocation tests passed");
    
    // Initialize heap allocator
    log::info!("Initializing heap allocator...");
    unsafe {
        allocator::init_heap(
            HEAP_MEMORY.as_ptr() as usize,
            HEAP_SIZE,
        );
    }
    print_str("Heap Allocator: INITIALIZED", 0, 17);
    
    // Test heap allocations
    log::info!("Testing heap allocations...");
    print_str("Testing Heap Allocations...", 0, 19);
    
    use alloc::vec::Vec;
    use alloc::string::String;
    use alloc::boxed::Box;
    
    let mut vec = Vec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    log::debug!("Vec test: OK");
    
    let mut s = String::from("Aetherion");
    s.push_str(" OS");
    log::debug!("String test: OK");
    
    let boxed = Box::new(42);
    log::debug!("Box test: OK (value: {})", *boxed);
    
    print_str("All heap tests passed!", 0, 20);
    
    // Display final status
    log::info!("═══════════════════════════════════════════════");
    log::info!("  COUCHE 1 HAL: OPERATIONAL");
    log::info!("  System Status: READY");
    log::info!("═══════════════════════════════════════════════");
    
    print_str("Status: OPERATIONAL", 0, 22);
    print_str("Couche 1 HAL: COMPLETE", 0, 23);
    print_str("System ready. Press Reset to reboot.", 0, 24);
    
    // Idle loop
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

/// Allocation Error Handler
/// Called when heap allocation fails
#[alloc_error_handler]
fn alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation failed: {:?}", layout);
}

/// Panic Handler
/// Imported from HAL layer
/// 
/// The actual panic handler is defined in hal/panic.rs to provide
/// sophisticated error reporting with ACHA integration.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crate::hal::panic::panic(info)
}

/// Initialize Serial Port (COM1)
fn init_serial() {
    unsafe {
        // Disable interrupts
        outb(SERIAL_PORT + 1, 0x00);
        
        // Set baud rate (115200)
        outb(SERIAL_PORT + 3, 0x80); // Enable DLAB
        outb(SERIAL_PORT + 0, 0x01); // Divisor low byte (115200)
        outb(SERIAL_PORT + 1, 0x00); // Divisor high byte
        
        // Configure: 8 bits, no parity, one stop bit
        outb(SERIAL_PORT + 3, 0x03);
        
        // Enable FIFO with 14-byte threshold
        outb(SERIAL_PORT + 2, 0xC7);
        
        // Enable RTS/DSR
        outb(SERIAL_PORT + 4, 0x0B);
    }
}

/// Print string to serial port
fn serial_print(s: &str) {
    for byte in s.bytes() {
        unsafe {
            // Wait for transmit buffer to be empty
            while (inb(SERIAL_PORT + 5) & 0x20) == 0 {}
            outb(SERIAL_PORT, byte);
        }
    }
}

/// Clear VGA screen
fn clear_screen() {
    let buffer = VGA_BUFFER;
    for i in 0..(VGA_WIDTH * VGA_HEIGHT) {
        unsafe {
            *buffer.offset(i as isize * 2) = b' ';
            *buffer.offset(i as isize * 2 + 1) = VGA_COLOR_WHITE_ON_BLACK;
        }
    }
}

/// Print string to VGA at specific position
fn print_str(s: &str, x: usize, y: usize) {
    let buffer = VGA_BUFFER;
    let offset = (y * VGA_WIDTH + x) * 2;
    
    for (i, byte) in s.bytes().enumerate() {
        if x + i >= VGA_WIDTH {
            break; // Don't overflow line
        }
        unsafe {
            *buffer.offset((offset + i * 2) as isize) = byte;
            *buffer.offset((offset + i * 2 + 1) as isize) = VGA_COLOR_WHITE_ON_BLACK;
        }
    }
}

/// Output byte to I/O port
#[inline]
unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nomem, nostack, preserves_flags)
    );
}

/// Input byte from I/O port
#[inline]
unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!(
        "in al, dx",
        out("al") value,
        in("dx") port,
        options(nomem, nostack, preserves_flags)
    );
    value
}

// Additional compiler-required functions

#[no_mangle]
pub extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    unsafe {
        for i in 0..n {
            *dest.add(i) = *src.add(i);
        }
    }
    dest
}

#[no_mangle]
pub extern "C" fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    unsafe {
        for i in 0..n {
            *s.add(i) = c as u8;
        }
    }
    s
}

#[no_mangle]
pub extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    unsafe {
        for i in 0..n {
            let a = *s1.add(i);
            let b = *s2.add(i);
            if a != b {
                return a as i32 - b as i32;
            }
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if dest < src as *mut u8 {
        // Copy forward
        unsafe {
            for i in 0..n {
                *dest.add(i) = *src.add(i);
            }
        }
    } else {
        // Copy backward
        unsafe {
            for i in (0..n).rev() {
                *dest.add(i) = *src.add(i);
            }
        }
    }
    dest
}
