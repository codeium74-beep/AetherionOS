/// Aetherion OS - Architecture Abstraction Module
/// Provides platform-specific implementations

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "x86_64")]
pub use self::x86_64 as current;

/// Initialize the current platform's HAL
pub fn init() {
    current::init();
}
