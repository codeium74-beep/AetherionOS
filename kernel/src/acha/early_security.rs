/// Aetherion OS - ACHA Early Security Module
/// Phase 4: TPM validation and early security checks
/// 
/// This module implements Couche 1 security requirements:
/// - TPM 2.0 detection via ACPI
/// - Boot refusal if TPM absent (production mode)
/// - Debug mode bypass for development
/// - Security event logging to ACHA
///
/// References:
/// - TPM 2.0 ACPI Specification: https://trustedcomputinggroup.org/wp-content/uploads/TPM-2.0-ACPI-Specification.pdf
/// - ACPI 6.5 Specification: https://uefi.org/sites/default/files/resources/ACPI_Spec_6_5_Aug29.pdf

use log::{info, warn, error};
use core::ptr::read_unaligned;

/// TPM detection status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TpmStatus {
    /// TPM 2.0 detected and validated
    Present,
    /// TPM not detected
    Absent,
    /// TPM detection not performed (debug mode)
    Bypassed,
}

/// ACPI RSDP (Root System Description Pointer) structure
/// Located in BIOS memory area (0xE0000-0xFFFFF)
#[repr(C, packed)]
struct AcpiRsdp {
    signature: [u8; 8],      // "RSD PTR "
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,            // 0 = ACPI 1.0, 2 = ACPI 2.0+
    rsdt_address: u32,       // For ACPI 1.0
    // ACPI 2.0+ fields
    length: u32,             // Total table length
    xsdt_address: u64,       // 64-bit physical address of XSDT
    extended_checksum: u8,
    reserved: [u8; 3],
}

/// ACPI SDT Header (common to all ACPI tables)
#[repr(C, packed)]
struct AcpiSdtHeader {
    signature: [u8; 4],      // Table signature
    length: u32,             // Total table length
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32,
}

/// TPM2 ACPI Table structure
/// Defined in TCG ACPI Specification
#[repr(C, packed)]
struct Tpm2Table {
    header: AcpiSdtHeader,   // Signature = "TPM2"
    platform_class: u16,     // 0 = Client, 1 = Server
    reserved: u16,
    acpi_control_area_address: u64,  // May be 0 if not used
    // Vendor-specific data follows
}

/// ACPI memory regions to search for RSDP
const ACPI_BIOS_START: usize = 0xE0000;
const ACPI_BIOS_END: usize = 0xFFFFF;
const ACPI_RSDP_SIGNATURE: &[u8; 8] = b"RSD PTR ";
const ACPI_TPM2_SIGNATURE: &[u8; 4] = b"TPM2";

/// Check for TPM 2.0 presence
/// 
/// Scans ACPI tables for TPM 2.0 device (ACPI signature "TPM2").
/// In production mode, absence of TPM causes kernel panic.
/// In debug mode (#[cfg(debug_assertions)]), TPM check is bypassed.
/// 
/// # Returns
/// - `true` if TPM is present or debug mode
/// - `false` if TPM is absent (will cause panic in production)
pub fn check_tpm() -> bool {
    #[cfg(debug_assertions)]
    {
        warn!("═══════════════════════════════════════════════");
        warn!("  DEBUG MODE: TPM check BYPASSED");
        warn!("  Production mode will REQUIRE TPM 2.0");
        warn!("═══════════════════════════════════════════════");
        
        // Log to ACHA that we're in insecure mode
        crate::acha::events::log_event(
            crate::acha::events::CognitiveEvent::SecurityViolation
        );
        
        return true; // Allow boot in debug mode
    }
    
    #[cfg(not(debug_assertions))]
    {
        info!("Performing TPM 2.0 validation...");
        
        match detect_tpm_acpi() {
            TpmStatus::Present => {
                info!("✓ TPM 2.0 detected and validated");
                info!("  Security: ENHANCED");
                true
            }
            TpmStatus::Absent => {
                error!("═══════════════════════════════════════════════");
                error!("  SECURITY VIOLATION: TPM 2.0 NOT DETECTED");
                error!("═══════════════════════════════════════════════");
                error!("  AetherionOS requires TPM 2.0 for secure boot.");
                error!("  Boot refused to prevent security compromise.");
                error!("═══════════════════════════════════════════════");
                
                // Log security violation to ACHA
                crate::acha::events::log_event(
                    crate::acha::events::CognitiveEvent::SecurityViolation
                );
                
                false // Will cause panic in caller
            }
            TpmStatus::Bypassed => {
                // Should not happen in production mode
                false
            }
        }
    }
}

/// Detect TPM 2.0 via ACPI tables
/// 
/// Searches for ACPI "TPM2" table which indicates TPM 2.0 presence.
/// Implementation follows TCG ACPI Specification v1.2.
/// 
/// Algorithm:
/// 1. Find RSDP in BIOS memory area (0xE0000-0xFFFFF)
/// 2. Parse RSDT/XSDT to enumerate all ACPI tables
/// 3. Search for table with "TPM2" signature
/// 4. Validate table checksum
/// 5. Verify TPM2 table contents
fn detect_tpm_acpi() -> TpmStatus {
    info!("Scanning ACPI tables for TPM2...");
    
    // Step 1: Find RSDP
    let rsdp = match find_rsdp() {
        Some(rsdp) => rsdp,
        None => {
            warn!("ACPI RSDP not found - cannot detect TPM");
            return TpmStatus::Absent;
        }
    };
    
    info!("ACPI RSDP found at revision {}", rsdp.revision);
    
    // Step 2: Parse RSDT or XSDT based on ACPI revision
    let tpm2_found = if rsdp.revision >= 2 && rsdp.xsdt_address != 0 {
        // ACPI 2.0+: Use XSDT (64-bit)
        parse_xsdt_for_tpm2(rsdp.xsdt_address)
    } else {
        // ACPI 1.0: Use RSDT (32-bit)
        parse_rsdt_for_tpm2(rsdp.rsdt_address as u64)
    };
    
    if tpm2_found {
        info!("TPM2 ACPI table found - TPM 2.0 device detected");
        TpmStatus::Present
    } else {
        warn!("TPM2 ACPI table not found - TPM 2.0 not detected");
        TpmStatus::Absent
    }
}

/// Find RSDP in BIOS memory area
/// 
/// Searches the BIOS ROM memory (0xE0000-0xFFFFF) on 16-byte boundaries
/// for the RSDP signature "RSD PTR ".
fn find_rsdp() -> Option<AcpiRsdp> {
    // Note: In a real kernel, we'd need to map this physical memory
    // For now, we use a direct physical address access approach
    // This works in QEMU and early boot environments with identity mapping
    
    let start = ACPI_BIOS_START as *const u8;
    let length = ACPI_BIOS_END - ACPI_BIOS_START;
    
    // Search on 16-byte boundaries
    for offset in (0..length).step_by(16) {
        let addr = unsafe { start.add(offset) };
        
        // Check for "RSD PTR " signature
        let signature = unsafe { core::slice::from_raw_parts(addr, 8) };
        if signature == ACPI_RSDP_SIGNATURE {
            // Read the full RSDP structure
            let rsdp = unsafe { read_unaligned(addr as *const AcpiRsdp) };
            
            // Validate checksum (ACPI 1.0 portion)
            if validate_rsdp_checksum(&rsdp) {
                info!("Valid RSDP found at {:#x}", ACPI_BIOS_START + offset);
                return Some(rsdp);
            } else {
                warn!("RSDP found at {:#x} but checksum invalid", ACPI_BIOS_START + offset);
            }
        }
    }
    
    None
}

/// Validate RSDP checksum
/// 
/// The checksum is calculated as the sum of all bytes in the ACPI 1.0
/// portion of the RSDP (first 20 bytes) must equal 0 (mod 256).
fn validate_rsdp_checksum(rsdp: &AcpiRsdp) -> bool {
    let bytes = unsafe {
        core::slice::from_raw_parts(
            rsdp as *const _ as *const u8,
            20 // ACPI 1.0 portion size
        )
    };
    
    let sum: u8 = bytes.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
    sum == 0
}

/// Parse RSDT (Root System Description Table) for TPM2
/// 
/// RSDT contains an array of 32-bit physical addresses pointing to
/// other ACPI tables.
fn parse_rsdt_for_tpm2(rsdt_address: u64) -> bool {
    info!("Parsing RSDT at {:#x}", rsdt_address);
    
    // Safety: We're assuming identity mapping or proper page mapping
    let header = unsafe { &*(rsdt_address as *const AcpiSdtHeader) };
    
    // Verify RSDT signature
    if &header.signature != b"RSDT" {
        warn!("RSDT signature mismatch: {:?}", header.signature);
        return false;
    }
    
    // Calculate number of entries
    let entries = (header.length as usize - core::mem::size_of::<AcpiSdtHeader>()) / 4;
    info!("RSDT has {} table entries", entries);
    
    // Get pointer to array of 32-bit addresses
    let tables = unsafe {
        (rsdt_address as *const u8)
            .add(core::mem::size_of::<AcpiSdtHeader>())
            as *const u32
    };
    
    // Search each table for TPM2
    for i in 0..entries {
        let table_addr = unsafe { *tables.add(i) } as u64;
        if check_table_for_tpm2(table_addr) {
            return true;
        }
    }
    
    false
}

/// Parse XSDT (Extended System Description Table) for TPM2
/// 
/// XSDT contains an array of 64-bit physical addresses pointing to
/// other ACPI tables. Used in ACPI 2.0+.
fn parse_xsdt_for_tpm2(xsdt_address: u64) -> bool {
    info!("Parsing XSDT at {:#x}", xsdt_address);
    
    // Safety: We're assuming identity mapping or proper page mapping
    let header = unsafe { &*(xsdt_address as *const AcpiSdtHeader) };
    
    // Verify XSDT signature
    if &header.signature != b"XSDT" {
        warn!("XSDT signature mismatch: {:?}", header.signature);
        return false;
    }
    
    // Calculate number of entries
    let entries = (header.length as usize - core::mem::size_of::<AcpiSdtHeader>()) / 8;
    info!("XSDT has {} table entries", entries);
    
    // Get pointer to array of 64-bit addresses
    let tables = unsafe {
        (xsdt_address as *const u8)
            .add(core::mem::size_of::<AcpiSdtHeader>())
            as *const u64
    };
    
    // Search each table for TPM2
    for i in 0..entries {
        let table_addr = unsafe { *tables.add(i) };
        if check_table_for_tpm2(table_addr) {
            return true;
        }
    }
    
    false
}

/// Check if an ACPI table at given address is TPM2
/// 
/// Reads the table header and checks for "TPM2" signature.
/// Also validates the table checksum.
fn check_table_for_tpm2(table_address: u64) -> bool {
    let header = unsafe { &*(table_address as *const AcpiSdtHeader) };
    
    // Check for TPM2 signature
    if &header.signature == ACPI_TPM2_SIGNATURE {
        info!("Found TPM2 table at {:#x}", table_address);
        
        // Validate checksum
        if validate_acpi_table_checksum(table_address, header.length) {
            // Read TPM2-specific data
            let tpm2 = unsafe { &*(table_address as *const Tpm2Table) };
            let platform_class = unsafe { read_unaligned(core::ptr::addr_of!(tpm2.platform_class)) };
            
            info!("TPM2 Platform Class: {} ({})", 
                platform_class,
                if platform_class == 0 { "Client" } else { "Server" }
            );
            
            return true;
        } else {
            warn!("TPM2 table checksum invalid at {:#x}", table_address);
        }
    }
    
    false
}

/// Validate ACPI table checksum
/// 
/// The checksum is calculated as the sum of all bytes in the table
/// must equal 0 (mod 256).
fn validate_acpi_table_checksum(table_address: u64, length: u32) -> bool {
    let bytes = unsafe {
        core::slice::from_raw_parts(table_address as *const u8, length as usize)
    };
    
    let sum: u8 = bytes.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
    sum == 0
}

/// Get TPM information string
pub fn get_tpm_info() -> &'static str {
    #[cfg(debug_assertions)]
    {
        "TPM check bypassed (DEBUG MODE)"
    }
    
    #[cfg(not(debug_assertions))]
    {
        match detect_tpm_acpi() {
            TpmStatus::Present => "TPM 2.0 Present",
            TpmStatus::Absent => "TPM 2.0 Absent - SECURITY RISK",
            TpmStatus::Bypassed => "TPM check disabled",
        }
    }
}

/// Initialize early security subsystem
pub fn init() {
    info!("Initializing ACHA early security...");
    
    // Perform TPM check
    let tpm_ok = check_tpm();
    
    if tpm_ok {
        info!("Early security checks passed");
    } else {
        error!("Early security checks FAILED");
        panic!("TPM 2.0 not detected - boot refused (production mode)");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tpm_check_debug_mode() {
        // In debug mode, TPM check should always pass
        assert!(check_tpm(), "TPM check should pass in debug mode");
    }
    
    #[test]
    fn test_tpm_status_enum() {
        assert_ne!(TpmStatus::Present, TpmStatus::Absent);
        assert_ne!(TpmStatus::Absent, TpmStatus::Bypassed);
    }
}
