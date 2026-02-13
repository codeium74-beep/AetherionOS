/// Aetherion OS - ACHA Events Module
/// Cognitive event tracking for kernel anomalies and security events

use spin::Mutex;

/// Cognitive event types tracked by ACHA
#[derive(Debug, Clone, Copy)]
pub enum CognitiveEvent {
    /// Kernel panic occurred
    KernelPanic,
    /// CPU exception was triggered
    Exception(&'static str),
    /// Security violation detected
    SecurityViolation,
    /// Memory allocation failed
    AllocationFailure,
}

/// Event counter
static EVENT_COUNTER: Mutex<u64> = Mutex::new(0);

/// Log a cognitive event
/// 
/// This function records events for later analysis by ACHA cognitive layers.
/// In Couche 1, we simply log and count events. Higher layers will perform
/// anomaly detection and predictive analysis.
pub fn log_event(event: CognitiveEvent) {
    let mut counter = EVENT_COUNTER.lock();
    *counter += 1;
    
    log::warn!("ACHA Event #{}: {:?}", *counter, event);
}

/// Log an exception event (convenience function)
pub fn log_exception(name: &'static str) {
    log_event(CognitiveEvent::Exception(name));
}

/// Get total event count
pub fn get_event_count() -> u64 {
    *EVENT_COUNTER.lock()
}

/// Initialize events subsystem
pub fn init() {
    log::debug!("ACHA events subsystem initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_logging() {
        let initial_count = get_event_count();
        log_exception("TestException");
        assert_eq!(get_event_count(), initial_count + 1);
    }
}
