/// Aetherion OS - ACHA Metrics Module
/// Kernel metrics collection for cognitive monitoring

use spin::Mutex;
use core::sync::atomic::{AtomicU64, Ordering};

/// Kernel metrics structure
pub struct KernelMetrics {
    /// Total interrupts handled
    pub interrupt_count: AtomicU64,
    /// Total page faults
    pub page_fault_count: AtomicU64,
    /// Total exceptions
    pub exception_count: AtomicU64,
    /// System uptime ticks
    pub uptime_ticks: AtomicU64,
}

/// Global metrics instance
static METRICS: Mutex<Option<KernelMetrics>> = Mutex::new(None);

/// Initialize metrics subsystem
pub fn init() {
    let mut metrics = METRICS.lock();
    *metrics = Some(KernelMetrics {
        interrupt_count: AtomicU64::new(0),
        page_fault_count: AtomicU64::new(0),
        exception_count: AtomicU64::new(0),
        uptime_ticks: AtomicU64::new(0),
    });
    
    log::debug!("ACHA metrics subsystem initialized");
}

/// Increment interrupt counter
pub fn increment_interrupt_count() {
    if let Some(ref metrics) = *METRICS.lock() {
        metrics.interrupt_count.fetch_add(1, Ordering::Relaxed);
    }
}

/// Increment page fault counter
pub fn increment_page_fault_count() {
    if let Some(ref metrics) = *METRICS.lock() {
        metrics.page_fault_count.fetch_add(1, Ordering::Relaxed);
    }
}

/// Increment exception counter
pub fn increment_exception_count() {
    if let Some(ref metrics) = *METRICS.lock() {
        metrics.exception_count.fetch_add(1, Ordering::Relaxed);
    }
}

/// Get current metrics snapshot
pub fn get_metrics() -> Option<(u64, u64, u64, u64)> {
    let metrics = METRICS.lock();
    if let Some(ref m) = *metrics {
        Some((
            m.interrupt_count.load(Ordering::Relaxed),
            m.page_fault_count.load(Ordering::Relaxed),
            m.exception_count.load(Ordering::Relaxed),
            m.uptime_ticks.load(Ordering::Relaxed),
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_increment() {
        init();
        increment_interrupt_count();
        increment_page_fault_count();
        
        if let Some((interrupts, page_faults, _, _)) = get_metrics() {
            assert!(interrupts > 0);
            assert!(page_faults > 0);
        }
    }
}
