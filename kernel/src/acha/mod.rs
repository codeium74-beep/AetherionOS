/// Aetherion OS - ACHA Integration
/// Couche 1: Early security and cognitive event tracking
/// 
/// ACHA (Adaptive Cognitive Hybrid Architecture) integration for HAL layer.
/// This module provides basic event tracking and metrics collection.

pub mod events;
pub mod metrics;
pub mod early_security;

/// Initialize ACHA subsystem
pub fn init() {
    log::info!("ACHA cognitive layer initializing...");
    metrics::init();
    events::init();
    early_security::init(); // TPM check happens here
    log::info!("ACHA cognitive layer ready");
}
