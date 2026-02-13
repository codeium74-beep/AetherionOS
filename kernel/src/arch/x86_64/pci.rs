/// Aetherion OS - HAL Layer - PCI Bus Enumeration
/// Phase 2: PCI device detection and initialization
/// 
/// This module provides HAL-level PCI bus scanning with structured logging
/// and ACHA integration for device discovery events.

use alloc::vec::Vec;
use crate::drivers::pci::{self, PciDevice};

/// Initialize PCI bus and enumerate devices
/// 
/// Scans all PCI buses (0-255), devices (0-31), and functions (0-7).
/// Logs discovered devices with detailed information for ACHA analysis.
pub fn init() {
    log::info!("Scanning PCI bus...");
    
    let devices = pci::scan_bus();
    
    log::info!("═══════════════════════════════════════════════");
    log::info!("  PCI Device Enumeration");
    log::info!("  Total Devices: {}", devices.len());
    log::info!("═══════════════════════════════════════════════");
    
    // Categorize devices
    let usb_count = devices.iter().filter(|d| d.is_usb_controller()).count();
    let network_count = devices.iter().filter(|d| d.is_network_controller()).count();
    let storage_count = devices.iter().filter(|d| d.is_storage_controller()).count();
    let display_count = devices.iter().filter(|d| d.is_display_controller()).count();
    
    log::info!("Device Categories:");
    log::info!("  USB Controllers:     {}", usb_count);
    log::info!("  Network Controllers: {}", network_count);
    log::info!("  Storage Controllers: {}", storage_count);
    log::info!("  Display Controllers: {}", display_count);
    log::info!("  Other Devices:       {}", 
        devices.len() - usb_count - network_count - storage_count - display_count);
    
    // Log detailed device information
    for device in &devices {
        log_device_info(device);
    }
    
    log::info!("═══════════════════════════════════════════════");
    log::info!("PCI bus scan complete");
}

/// Log detailed information about a PCI device
fn log_device_info(device: &PciDevice) {
    log::debug!(
        "PCI [{:02X}:{:02X}.{:01X}] {:#06X}:{:#06X} - {} (Class: {:#04X}, Subclass: {:#04X})",
        device.bus,
        device.device,
        device.function,
        device.vendor_id,
        device.device_id,
        device.device_name(),
        device.class_code,
        device.subclass
    );
    
    // Log BAR information if present
    for bar_idx in 0..6 {
        if let Some(bar_type) = device.get_bar_type(bar_idx) {
            log::trace!("  BAR{}: {:?}", bar_idx, bar_type);
        }
    }
    
    // Log USB controller type if applicable
    if let Some(usb_type) = device.usb_controller_type() {
        log::info!("  USB Controller Type: {:?}", usb_type);
    }
}

/// Get list of all PCI devices
pub fn get_devices() -> Vec<PciDevice> {
    pci::scan_bus()
}

/// Find USB controllers
pub fn find_usb_controllers() -> Vec<PciDevice> {
    pci::scan_usb_controllers()
}

/// Find network controllers
pub fn find_network_controllers() -> Vec<PciDevice> {
    pci::find_devices_by_class(0x02)
}

/// Find storage controllers
pub fn find_storage_controllers() -> Vec<PciDevice> {
    pci::find_devices_by_class(0x01)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pci_scan() {
        // Test that PCI scan doesn't panic
        let devices = get_devices();
        // In QEMU, should find at least a few devices
        log::debug!("Found {} PCI devices in test", devices.len());
    }
}
