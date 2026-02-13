/// Aetherion OS - HAL Layer - x86_64 Architecture Module
/// Phase 1-2: Hardware Abstraction Layer for x86_64 architecture

pub mod gdt;
pub mod idt;
pub mod pci;

/// Initialize x86_64 architecture-specific components
/// 
/// This function must be called early in kernel initialization.
/// Order of initialization is critical:
/// 1. GDT (required for proper segmentation)
/// 2. IDT (requires GDT for TSS)
/// 3. PCI (hardware enumeration)
pub fn init() {
    // Initialize GDT first (required for TSS)
    gdt::init();
    
    // Initialize IDT (requires GDT for double-fault stack)
    idt::init();
    
    // Initialize PCI bus (hardware detection)
    pci::init();
}
