/// Aetherion OS - HAL Layer - GDT Implementation
/// Phase 1.1: Global Descriptor Table with TSS for double-fault handling
/// 
/// This module implements the Global Descriptor Table (GDT) using the x86_64 crate.
/// It provides proper segmentation setup including:
/// - Kernel code/data segments
/// - Task State Segment (TSS) for interrupt stack switching
/// - Double-fault stack to prevent recursive stack overflow

use x86_64::VirtAddr;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use lazy_static::lazy_static;

/// Index of the double-fault IST stack in the TSS
/// IST (Interrupt Stack Table) allows switching to a separate stack for specific interrupts
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    /// Task State Segment
    /// Contains information about task state including separate interrupt stacks
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        
        // Allocate dedicated stack for double-fault handler
        // This prevents stack overflow from recursively triggering double-faults
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5; // 20 KiB stack
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end // Stack grows downward, so we return the end address
        };
        
        tss
    };
}

lazy_static! {
    /// Global Descriptor Table
    /// Contains segment descriptors for code, data, and TSS segments
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        
        // Add kernel code segment (ring 0, executable)
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        
        // Add kernel data segment (ring 0, writable)
        let data_selector = gdt.add_entry(Descriptor::kernel_data_segment());
        
        // Add TSS segment for interrupt handling
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        
        (gdt, Selectors {
            code_selector,
            data_selector,
            tss_selector,
        })
    };
}

/// Segment selectors returned by GDT initialization
struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

/// Initialize the GDT and load it into the CPU
/// 
/// This function must be called early in kernel initialization, before setting up
/// interrupts, as the IDT relies on the GDT for proper segment configuration.
/// 
/// # Safety
/// This function uses inline assembly to load GDT and update segment registers.
/// It must only be called once during kernel initialization.
pub fn init() {
    use x86_64::instructions::segmentation::{CS, DS, Segment};
    use x86_64::instructions::tables::load_tss;

    log::debug!("Loading GDT...");
    
    // Load the GDT into GDTR register
    GDT.0.load();
    
    unsafe {
        // Update code segment register
        CS::set_reg(GDT.1.code_selector);
        
        // Update data segment register
        DS::set_reg(GDT.1.data_selector);
        
        // Load TSS (Task State Segment)
        load_tss(GDT.1.tss_selector);
    }
    
    log::info!("GDT loaded successfully (code: {:?}, data: {:?}, tss: {:?})",
        GDT.1.code_selector, GDT.1.data_selector, GDT.1.tss_selector);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tss_double_fault_stack() {
        let tss = &*TSS;
        let stack_addr = tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize];
        assert_ne!(stack_addr.as_u64(), 0, "Double-fault stack should be allocated");
    }
}
