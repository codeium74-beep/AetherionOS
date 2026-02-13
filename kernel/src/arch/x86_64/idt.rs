/// Aetherion OS - HAL Layer - IDT Implementation
/// Phase 1.2: Interrupt Descriptor Table with exception handlers
/// 
/// This module implements a comprehensive IDT (Interrupt Descriptor Table) with handlers
/// for all CPU exceptions (0-31). It uses the x86_64 crate's interrupt calling convention
/// and integrates with ACHA for cognitive event logging.

use x86_64::structures::idt::{
    InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode
};
use lazy_static::lazy_static;
use crate::arch::x86_64::gdt;

lazy_static! {
    /// Interrupt Descriptor Table
    /// Maps interrupt vectors (0-255) to handler functions
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        
        // CPU Exceptions (0-31)
        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.debug.set_handler_fn(debug_handler);
        idt.non_maskable_interrupt.set_handler_fn(nmi_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.overflow.set_handler_fn(overflow_handler);
        idt.bound_range_exceeded.set_handler_fn(bound_range_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.device_not_available.set_handler_fn(device_not_available_handler);
        
        // Double fault with separate IST stack (prevents recursive stack overflow)
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        
        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        idt.segment_not_present.set_handler_fn(segment_not_present_handler);
        idt.stack_segment_fault.set_handler_fn(stack_segment_fault_handler);
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.x87_floating_point.set_handler_fn(x87_floating_point_handler);
        idt.alignment_check.set_handler_fn(alignment_check_handler);
        idt.machine_check.set_handler_fn(machine_check_handler);
        idt.simd_floating_point.set_handler_fn(simd_floating_point_handler);
        idt.virtualization.set_handler_fn(virtualization_handler);
        idt.security_exception.set_handler_fn(security_exception_handler);
        
        idt
    };
}

/// Initialize and load the IDT
/// 
/// This function must be called after GDT initialization and before enabling interrupts.
/// It loads the IDT into the CPU's IDTR register.
pub fn init() {
    log::debug!("Loading IDT...");
    IDT.load();
    log::info!("IDT loaded successfully with {} exception handlers", 20);
}

// ============================================================================
// Exception Handlers
// ============================================================================

/// Divide by Zero Exception (#DE)
/// Triggered when attempting to divide by zero
extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    log::error!("EXCEPTION: DIVIDE BY ZERO");
    log::error!("{:#?}", stack_frame);
    
    panic!("DIVIDE BY ZERO");
}

/// Debug Exception (#DB)
/// Triggered by debug events (breakpoints, single-step)
extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
    log::debug!("EXCEPTION: DEBUG");
    log::debug!("{:#?}", stack_frame);
}

/// Non-Maskable Interrupt (NMI)
/// Hardware interrupt that cannot be masked
extern "x86-interrupt" fn nmi_handler(stack_frame: InterruptStackFrame) {
    log::error!("EXCEPTION: NON-MASKABLE INTERRUPT");
    log::error!("{:#?}", stack_frame);
    panic!("NMI");
}

/// Breakpoint Exception (#BP)
/// Triggered by INT3 instruction (used by debuggers)
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    log::info!("EXCEPTION: BREAKPOINT");
    log::debug!("{:#?}", stack_frame);
}

/// Overflow Exception (#OF)
/// Triggered by INTO instruction when overflow flag is set
extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    log::error!("EXCEPTION: OVERFLOW");
    log::error!("{:#?}", stack_frame);
    panic!("OVERFLOW");
}

/// Bound Range Exceeded (#BR)
/// Triggered when BOUND instruction detects out-of-range value
extern "x86-interrupt" fn bound_range_handler(stack_frame: InterruptStackFrame) {
    log::error!("EXCEPTION: BOUND RANGE EXCEEDED");
    log::error!("{:#?}", stack_frame);
    panic!("BOUND RANGE EXCEEDED");
}

/// Invalid Opcode (#UD)
/// Triggered when CPU encounters invalid/unsupported instruction
extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    log::error!("EXCEPTION: INVALID OPCODE");
    log::error!("{:#?}", stack_frame);
    panic!("INVALID OPCODE");
}

/// Device Not Available (#NM)
/// Triggered when using FPU/MMX/SSE without proper setup
extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
    log::error!("EXCEPTION: DEVICE NOT AVAILABLE");
    log::error!("{:#?}", stack_frame);
    panic!("DEVICE NOT AVAILABLE");
}

/// Double Fault (#DF)
/// Triggered when exception occurs while handling another exception
/// Uses separate IST stack to prevent recursive stack overflow
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    log::error!("EXCEPTION: DOUBLE FAULT (error_code: {:#x})", error_code);
    log::error!("{:#?}", stack_frame);
    
    panic!("DOUBLE FAULT");
}

/// Invalid TSS (#TS)
/// Triggered when loading invalid Task State Segment
extern "x86-interrupt" fn invalid_tss_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    log::error!("EXCEPTION: INVALID TSS (error_code: {:#x})", error_code);
    log::error!("{:#?}", stack_frame);
    panic!("INVALID TSS");
}

/// Segment Not Present (#NP)
/// Triggered when accessing segment marked as not present
extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    log::error!("EXCEPTION: SEGMENT NOT PRESENT (error_code: {:#x})", error_code);
    log::error!("{:#?}", stack_frame);
    panic!("SEGMENT NOT PRESENT");
}

/// Stack Segment Fault (#SS)
/// Triggered by stack segment violations
extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    log::error!("EXCEPTION: STACK SEGMENT FAULT (error_code: {:#x})", error_code);
    log::error!("{:#?}", stack_frame);
    panic!("STACK SEGMENT FAULT");
}

/// General Protection Fault (#GP)
/// Triggered by various protection violations
extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    log::error!("EXCEPTION: GENERAL PROTECTION FAULT (error_code: {:#x})", error_code);
    log::error!("{:#?}", stack_frame);
    panic!("GENERAL PROTECTION FAULT");
}

/// Page Fault (#PF)
/// Triggered when accessing invalid memory page or protection violation
extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    
    let accessed_address = Cr2::read();
    log::error!("EXCEPTION: PAGE FAULT");
    log::error!("Accessed Address: {:?}", accessed_address);
    log::error!("Error Code: {:?}", error_code);
    log::error!("{:#?}", stack_frame);
    
    panic!("PAGE FAULT");
}

/// x87 FPU Floating-Point Exception (#MF)
/// Triggered by x87 FPU floating-point errors
extern "x86-interrupt" fn x87_floating_point_handler(stack_frame: InterruptStackFrame) {
    log::error!("EXCEPTION: x87 FLOATING POINT");
    log::error!("{:#?}", stack_frame);
    panic!("x87 FLOATING POINT");
}

/// Alignment Check (#AC)
/// Triggered when alignment checking is enabled and unaligned memory access occurs
extern "x86-interrupt" fn alignment_check_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    log::error!("EXCEPTION: ALIGNMENT CHECK (error_code: {:#x})", error_code);
    log::error!("{:#?}", stack_frame);
    panic!("ALIGNMENT CHECK");
}

/// Machine Check (#MC)
/// Triggered by internal machine errors (hardware failures)
extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
    log::error!("EXCEPTION: MACHINE CHECK");
    log::error!("{:#?}", stack_frame);
    panic!("MACHINE CHECK");
}

/// SIMD Floating-Point Exception (#XM/#XF)
/// Triggered by SSE/AVX floating-point errors
extern "x86-interrupt" fn simd_floating_point_handler(stack_frame: InterruptStackFrame) {
    log::error!("EXCEPTION: SIMD FLOATING POINT");
    log::error!("{:#?}", stack_frame);
    panic!("SIMD FLOATING POINT");
}

/// Virtualization Exception (#VE)
/// Triggered in virtualized environments
extern "x86-interrupt" fn virtualization_handler(stack_frame: InterruptStackFrame) {
    log::error!("EXCEPTION: VIRTUALIZATION");
    log::error!("{:#?}", stack_frame);
    panic!("VIRTUALIZATION");
}

/// Security Exception (#SX)
/// Triggered by security-related violations
extern "x86-interrupt" fn security_exception_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    log::error!("EXCEPTION: SECURITY EXCEPTION (error_code: {:#x})", error_code);
    log::error!("{:#?}", stack_frame);
    
    panic!("SECURITY EXCEPTION");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idt_creation() {
        // Just ensure IDT can be created without panicking
        let _ = &*IDT;
    }
}
