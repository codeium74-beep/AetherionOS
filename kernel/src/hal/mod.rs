/// Aetherion OS - Hardware Abstraction Layer (HAL)
/// Couche 1: Foundation layer providing hardware abstraction

pub mod logger;
pub mod panic;

/// Initialize the HAL layer
/// 
/// This function initializes all hardware abstraction components:
/// - Serial port (for debug output)
/// - Logger (for structured logging)
/// - Architecture-specific initialization (GDT, IDT)
pub fn init() {
    // Serial must be initialized first for logging
    crate::drivers::serial::init();
    
    // Initialize logger (depends on serial)
    logger::init().expect("Failed to initialize logger");
    
    log::info!("HAL layer initialized");
}
