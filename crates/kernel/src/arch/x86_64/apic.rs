//! # Local APIC (Advanced Programmable Interrupt Controller)
//!
//! This module provides support for the Local APIC, the modern interrupt controller
//! found on all contemporary x86_64 CPUs. The Local APIC replaces the legacy PIC
//! (8259A) and provides enhanced interrupt handling capabilities.
//!
//! ## APIC vs PIC
//!
//! The APIC offers several advantages over the legacy PIC:
//! - **Per-CPU Interrupts**: Each CPU has its own Local APIC
//! - **More IRQ Lines**: 256 vectors vs 15 usable IRQs
//! - **Inter-Processor Interrupts (IPIs)**: For SMP synchronization
//! - **Better Performance**: Lower latency, hardware prioritization
//! - **Timer Integration**: High-resolution local timer
//!
//! ## APIC Modes
//!
//! The Local APIC can operate in two modes:
//!
//! ### xAPIC (Legacy Memory-Mapped)
//! - Physical address: 0xFEE00000 (fixed)
//! - Access via MMIO (memory-mapped I/O)
//! - 4-byte aligned 32-bit registers
//! - Maximum 255 CPUs
//!
//! ### x2APIC (Modern MSR-Based)
//! - Access via Model-Specific Registers (MSRs)
//! - Faster than MMIO (no bus transactions)
//! - Supports more than 255 CPUs
//! - Available on newer CPUs (Nehalem+)
//! - Preferred mode when available
//!
//! ## Local APIC Registers
//!
//! Key registers (offsets for xAPIC mode):
//!
//! ```text
//! Offset  Name                  Description
//! ------  --------------------  ------------------------------------
//! 0x020   Local APIC ID         APIC identifier for this CPU
//! 0x030   Local APIC Version    APIC version information
//! 0x080   Task Priority (TPR)   Current task priority
//! 0x0B0   EOI                   End of Interrupt
//! 0x0D0   Logical Destination   Logical APIC ID
//! 0x0E0   Destination Format    Flat or cluster mode
//! 0x0F0   Spurious Int Vector   Spurious interrupt configuration
//! 0x300   ICR Low               Interrupt Command (IPI)
//! 0x310   ICR High              IPI destination
//! 0x320   LVT Timer             Local vector table - Timer
//! 0x330   LVT Thermal           Local vector table - Thermal sensor
//! 0x340   LVT Perf Counter      Local vector table - Performance counter
//! 0x350   LVT LINT0             Local vector table - LINT0 pin
//! 0x360   LVT LINT1             Local vector table - LINT1 pin
//! 0x370   LVT Error             Local vector table - Error
//! 0x380   Timer Initial Count   Timer reload value
//! 0x390   Timer Current Count   Current timer value (read-only)
//! 0x3E0   Timer Divide Config   Timer frequency divisor
//! ```
//!
//! ## Local APIC Timer
//!
//! Each Local APIC contains a 32-bit programmable timer:
//! - **One-shot mode**: Counts down once, then stops
//! - **Periodic mode**: Reloads automatically
//! - **TSC-Deadline mode**: Fire at specific TSC value
//!
//! Timer frequency = Bus frequency / Divisor
//! Common divisors: 1, 2, 4, 8, 16, 32, 64, 128
//!
//! ## Inter-Processor Interrupts (IPIs)
//!
//! IPIs are used for SMP communication:
//! - **TLB Shootdown**: Invalidate TLB entries on other CPUs
//! - **Reschedule**: Trigger scheduler on another CPU
//! - **Function Call**: Execute function on specific CPU
//! - **Halt**: Stop another CPU
//!
//! ## Initialization Sequence
//!
//! 1. Detect APIC mode (xAPIC vs x2APIC)
//! 2. Map APIC registers (xAPIC only)
//! 3. Enable APIC via APIC_BASE MSR
//! 4. Set Spurious Interrupt Vector (0xFF)
//! 5. Configure LVT entries (mask/unmask)
//! 6. Configure timer if needed
//! 7. Send EOI to clear any pending interrupts
//!
//! ## Safety Considerations
//!
//! - APIC registers must be accessed with correct alignment
//! - EOI must be sent for all handled interrupts
//! - Timer must be configured before enabling timer interrupt
//! - IPIs must specify valid destination CPU
//! - x2APIC mode cannot be disabled once enabled

use core::ptr::{read_volatile, write_volatile};
use raw_cpuid::CpuId;
use spin::Mutex;
use x86_64::VirtAddr;

/// xAPIC base physical address (fixed by Intel architecture)
const APIC_BASE_ADDR: u64 = 0xFEE00000;

/// MSR for APIC base address configuration
const IA32_APIC_BASE: u32 = 0x1B;

/// APIC_BASE MSR bit: Global APIC enable
const APIC_BASE_ENABLE: u64 = 1 << 11;

/// APIC_BASE MSR bit: x2APIC mode enable
const APIC_BASE_X2APIC: u64 = 1 << 10;

/// APIC_BASE MSR bit: BSP (Bootstrap Processor) flag
const APIC_BASE_BSP: u64 = 1 << 8;

// xAPIC Register offsets
const APIC_REG_ID: u32 = 0x020;              // Local APIC ID
const APIC_REG_VERSION: u32 = 0x030;         // APIC Version
const APIC_REG_TPR: u32 = 0x080;             // Task Priority
const APIC_REG_EOI: u32 = 0x0B0;             // End of Interrupt
const APIC_REG_SPURIOUS: u32 = 0x0F0;        // Spurious Interrupt Vector
const APIC_REG_ICR_LOW: u32 = 0x300;         // Interrupt Command (low)
const APIC_REG_ICR_HIGH: u32 = 0x310;        // Interrupt Command (high)
const APIC_REG_LVT_TIMER: u32 = 0x320;       // LVT Timer
const APIC_REG_LVT_LINT0: u32 = 0x350;       // LVT LINT0
const APIC_REG_LVT_LINT1: u32 = 0x360;       // LVT LINT1
const APIC_REG_LVT_ERROR: u32 = 0x370;       // LVT Error
const APIC_REG_TIMER_INIT: u32 = 0x380;      // Timer Initial Count
const APIC_REG_TIMER_CURRENT: u32 = 0x390;   // Timer Current Count
const APIC_REG_TIMER_DIV: u32 = 0x3E0;       // Timer Divide Configuration

// x2APIC MSR base
const X2APIC_MSR_BASE: u32 = 0x800;

// Spurious Interrupt Vector Register bits
const APIC_SPURIOUS_ENABLE: u32 = 1 << 8;    // APIC Software Enable
const APIC_SPURIOUS_VECTOR: u32 = 0xFF;      // Spurious vector number

// LVT bits
const APIC_LVT_MASKED: u32 = 1 << 16;        // Interrupt masked
const APIC_LVT_TIMER_PERIODIC: u32 = 1 << 17; // Periodic mode
const APIC_LVT_TIMER_DEADLINE: u32 = 2 << 17; // TSC-Deadline mode

// ICR (Interrupt Command Register) bits - for IPIs
const ICR_DELIVERY_MODE_FIXED: u32 = 0 << 8;      // Fixed delivery
const ICR_DELIVERY_MODE_LOWEST: u32 = 1 << 8;     // Lowest priority
const ICR_DELIVERY_MODE_SMI: u32 = 2 << 8;        // SMI
const ICR_DELIVERY_MODE_NMI: u32 = 4 << 8;        // NMI
const ICR_DELIVERY_MODE_INIT: u32 = 5 << 8;       // INIT IPI
const ICR_DELIVERY_MODE_SIPI: u32 = 6 << 8;       // SIPI (Startup IPI)

const ICR_DEST_MODE_PHYSICAL: u32 = 0 << 11;      // Physical destination
const ICR_DEST_MODE_LOGICAL: u32 = 1 << 11;       // Logical destination

const ICR_DELIVERY_PENDING: u32 = 1 << 12;        // Delivery status (read-only)

const ICR_LEVEL_DEASSERT: u32 = 0 << 14;          // De-assert level
const ICR_LEVEL_ASSERT: u32 = 1 << 14;            // Assert level

const ICR_TRIGGER_EDGE: u32 = 0 << 15;            // Edge triggered
const ICR_TRIGGER_LEVEL: u32 = 1 << 15;           // Level triggered

const ICR_DEST_SHORTHAND_NONE: u32 = 0 << 18;     // No shorthand
const ICR_DEST_SHORTHAND_SELF: u32 = 1 << 18;     // Send to self
const ICR_DEST_SHORTHAND_ALL: u32 = 2 << 18;      // All including self
const ICR_DEST_SHORTHAND_OTHERS: u32 = 3 << 18;   // All excluding self

/// APIC operating mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApicMode {
    /// APIC is disabled
    Disabled,
    /// xAPIC mode (memory-mapped)
    XApic,
    /// x2APIC mode (MSR-based)
    X2Apic,
}

/// IPI (Inter-Processor Interrupt) types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpiType {
    /// Fixed interrupt with specified vector
    Fixed(u8),
    /// INIT IPI - resets target CPU to initial state
    Init,
    /// SIPI (Startup IPI) - starts CPU at specified 4K page
    /// Page number is bits 19:12 of startup address (i.e., address >> 12)
    Startup(u8),
    /// NMI (Non-Maskable Interrupt)
    Nmi,
}

/// IPI destination
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpiDestination {
    /// Specific APIC ID (physical mode)
    Physical(u32),
    /// Send to self
    SelfOnly,
    /// Send to all CPUs (including self)
    AllIncludingSelf,
    /// Send to all CPUs (excluding self)
    AllExcludingSelf,
}

/// Local APIC controller
pub struct LocalApic {
    /// Current APIC mode
    mode: ApicMode,
    /// Virtual address for xAPIC MMIO (None for x2APIC)
    base_addr: Option<VirtAddr>,
    /// Local APIC ID
    apic_id: u32,
}

impl LocalApic {
    /// Create and initialize a new Local APIC
    ///
    /// Detects the APIC mode (xAPIC or x2APIC), maps registers if needed,
    /// and performs basic initialization.
    ///
    /// # Safety
    ///
    /// Must be called exactly once per CPU during boot.
    pub unsafe fn new() -> Result<Self, &'static str> {
        let mode = detect_apic_mode()?;

        let base_addr = match mode {
            ApicMode::XApic => {
                // Map APIC MMIO region
                // TODO: Proper memory mapping via MM subsystem (M3)
                // For now, use identity mapping
                Some(VirtAddr::new(APIC_BASE_ADDR))
            }
            ApicMode::X2Apic => None, // MSR access, no mapping needed
            ApicMode::Disabled => return Err("APIC not available"),
        };

        let mut apic = Self {
            mode,
            base_addr,
            apic_id: 0,
        };

        apic.init()?;
        Ok(apic)
    }

    /// Initialize the Local APIC
    unsafe fn init(&mut self) -> Result<(), &'static str> {
        match self.mode {
            ApicMode::XApic => self.init_xapic(),
            ApicMode::X2Apic => self.init_x2apic(),
            ApicMode::Disabled => Err("APIC disabled"),
        }
    }

    /// Initialize xAPIC (memory-mapped) mode
    unsafe fn init_xapic(&mut self) -> Result<(), &'static str> {
        // Enable APIC via APIC_BASE MSR
        let mut apic_base = crate::arch::x86_64::rdmsr(IA32_APIC_BASE);
        apic_base |= APIC_BASE_ENABLE;
        crate::arch::x86_64::wrmsr(IA32_APIC_BASE, apic_base);

        // Read APIC ID
        self.apic_id = self.read_xapic(APIC_REG_ID) >> 24;

        // Set Spurious Interrupt Vector (0xFF) and enable APIC
        self.write_xapic(APIC_REG_SPURIOUS, APIC_SPURIOUS_ENABLE | APIC_SPURIOUS_VECTOR);

        // Mask all LVT entries initially
        self.write_xapic(APIC_REG_LVT_TIMER, APIC_LVT_MASKED);
        self.write_xapic(APIC_REG_LVT_LINT0, APIC_LVT_MASKED);
        self.write_xapic(APIC_REG_LVT_LINT1, APIC_LVT_MASKED);
        self.write_xapic(APIC_REG_LVT_ERROR, APIC_LVT_MASKED);

        // Clear task priority (allow all interrupts)
        self.write_xapic(APIC_REG_TPR, 0);

        // Send EOI to clear any pending interrupts
        self.write_xapic(APIC_REG_EOI, 0);

        Ok(())
    }

    /// Initialize x2APIC (MSR-based) mode
    unsafe fn init_x2apic(&mut self) -> Result<(), &'static str> {
        // Enable x2APIC mode via APIC_BASE MSR
        let mut apic_base = crate::arch::x86_64::rdmsr(IA32_APIC_BASE);
        apic_base |= APIC_BASE_ENABLE | APIC_BASE_X2APIC;
        crate::arch::x86_64::wrmsr(IA32_APIC_BASE, apic_base);

        // Read APIC ID from x2APIC MSR
        self.apic_id = crate::arch::x86_64::rdmsr(X2APIC_MSR_BASE + (APIC_REG_ID >> 4)) as u32;

        // Set Spurious Interrupt Vector and enable APIC
        self.write_x2apic(APIC_REG_SPURIOUS, APIC_SPURIOUS_ENABLE | APIC_SPURIOUS_VECTOR);

        // Mask all LVT entries
        self.write_x2apic(APIC_REG_LVT_TIMER, APIC_LVT_MASKED);
        self.write_x2apic(APIC_REG_LVT_LINT0, APIC_LVT_MASKED);
        self.write_x2apic(APIC_REG_LVT_LINT1, APIC_LVT_MASKED);
        self.write_x2apic(APIC_REG_LVT_ERROR, APIC_LVT_MASKED);

        // Clear task priority
        self.write_x2apic(APIC_REG_TPR, 0);

        // Send EOI
        self.write_x2apic(APIC_REG_EOI, 0);

        Ok(())
    }

    /// Read xAPIC register (memory-mapped)
    unsafe fn read_xapic(&self, offset: u32) -> u32 {
        let addr = self.base_addr.unwrap().as_u64() + offset as u64;
        read_volatile(addr as *const u32)
    }

    /// Write xAPIC register (memory-mapped)
    unsafe fn write_xapic(&self, offset: u32, value: u32) {
        let addr = self.base_addr.unwrap().as_u64() + offset as u64;
        write_volatile(addr as *mut u32, value);
    }

    /// Read x2APIC register (MSR-based)
    unsafe fn read_x2apic(&self, offset: u32) -> u64 {
        let msr = X2APIC_MSR_BASE + (offset >> 4);
        crate::arch::x86_64::rdmsr(msr)
    }

    /// Write x2APIC register (MSR-based)
    unsafe fn write_x2apic(&self, offset: u32, value: u32) {
        let msr = X2APIC_MSR_BASE + (offset >> 4);
        crate::arch::x86_64::wrmsr(msr, value as u64);
    }

    /// Send End of Interrupt (EOI) signal
    ///
    /// Must be called at the end of every interrupt handler to signal
    /// that interrupt processing is complete.
    ///
    /// # Safety
    ///
    /// Must be called exactly once per interrupt.
    pub unsafe fn eoi(&self) {
        match self.mode {
            ApicMode::XApic => self.write_xapic(APIC_REG_EOI, 0),
            ApicMode::X2Apic => self.write_x2apic(APIC_REG_EOI, 0),
            ApicMode::Disabled => {}
        }
    }

    /// Get the Local APIC ID
    pub fn id(&self) -> u32 {
        self.apic_id
    }

    /// Get the current APIC mode
    pub fn mode(&self) -> ApicMode {
        self.mode
    }

    /// Configure the Local APIC timer
    ///
    /// # Arguments
    ///
    /// * `vector` - Interrupt vector number (32-255)
    /// * `initial_count` - Timer reload value
    /// * `divisor` - Frequency divisor (1, 2, 4, 8, 16, 32, 64, 128)
    /// * `periodic` - True for periodic mode, false for one-shot
    ///
    /// # Safety
    ///
    /// Must have a handler registered for the specified vector.
    pub unsafe fn configure_timer(&self, vector: u8, initial_count: u32, divisor: u8, periodic: bool) {
        // Validate divisor
        let div_value = match divisor {
            1 => 0x0B,
            2 => 0x00,
            4 => 0x01,
            8 => 0x02,
            16 => 0x03,
            32 => 0x08,
            64 => 0x09,
            128 => 0x0A,
            _ => return, // Invalid divisor
        };

        // Set timer divisor
        match self.mode {
            ApicMode::XApic => self.write_xapic(APIC_REG_TIMER_DIV, div_value),
            ApicMode::X2Apic => self.write_x2apic(APIC_REG_TIMER_DIV, div_value),
            ApicMode::Disabled => return,
        }

        // Configure timer LVT entry
        let mut lvt = vector as u32;
        if periodic {
            lvt |= APIC_LVT_TIMER_PERIODIC;
        }

        match self.mode {
            ApicMode::XApic => {
                self.write_xapic(APIC_REG_LVT_TIMER, lvt);
                self.write_xapic(APIC_REG_TIMER_INIT, initial_count);
            }
            ApicMode::X2Apic => {
                self.write_x2apic(APIC_REG_LVT_TIMER, lvt);
                self.write_x2apic(APIC_REG_TIMER_INIT, initial_count);
            }
            ApicMode::Disabled => {}
        }
    }

    /// Send Inter-Processor Interrupt (IPI)
    ///
    /// # Arguments
    ///
    /// * `destination` - IPI destination (specific CPU, self, or all)
    /// * `ipi_type` - Type of IPI to send (Fixed, INIT, SIPI, NMI)
    ///
    /// # Safety
    ///
    /// - For Fixed IPIs: destination CPU must have a handler for the vector
    /// - For INIT: destination CPU will be reset to initial state
    /// - For SIPI: startup vector must be a valid 4K-aligned address >> 12
    ///
    /// # Example
    ///
    /// ```rust
    /// // Send INIT IPI to APIC ID 1
    /// apic.send_ipi(IpiDestination::Physical(1), IpiType::Init);
    ///
    /// // Send SIPI to start CPU at 0x8000
    /// apic.send_ipi(IpiDestination::Physical(1), IpiType::Startup(0x08));
    ///
    /// // Send fixed interrupt vector 0x30 to all other CPUs
    /// apic.send_ipi(IpiDestination::AllExcludingSelf, IpiType::Fixed(0x30));
    /// ```
    pub unsafe fn send_ipi(&self, destination: IpiDestination, ipi_type: IpiType) {
        // Build ICR value
        let mut icr_low: u32 = 0;
        let mut dest_apic_id: u32 = 0;

        // Set delivery mode and vector based on IPI type
        match ipi_type {
            IpiType::Fixed(vector) => {
                icr_low |= ICR_DELIVERY_MODE_FIXED;
                icr_low |= vector as u32;
            }
            IpiType::Init => {
                icr_low |= ICR_DELIVERY_MODE_INIT;
                icr_low |= ICR_TRIGGER_LEVEL | ICR_LEVEL_ASSERT;
            }
            IpiType::Startup(page) => {
                icr_low |= ICR_DELIVERY_MODE_SIPI;
                icr_low |= page as u32; // Startup address is page << 12
            }
            IpiType::Nmi => {
                icr_low |= ICR_DELIVERY_MODE_NMI;
            }
        }

        // Set destination mode and shorthand
        match destination {
            IpiDestination::Physical(apic_id) => {
                icr_low |= ICR_DEST_MODE_PHYSICAL | ICR_DEST_SHORTHAND_NONE;
                dest_apic_id = apic_id;
            }
            IpiDestination::SelfOnly => {
                icr_low |= ICR_DEST_SHORTHAND_SELF;
            }
            IpiDestination::AllIncludingSelf => {
                icr_low |= ICR_DEST_SHORTHAND_ALL;
            }
            IpiDestination::AllExcludingSelf => {
                icr_low |= ICR_DEST_SHORTHAND_OTHERS;
            }
        }

        // Send the IPI
        match self.mode {
            ApicMode::XApic => {
                // Write destination to ICR high
                self.write_xapic(APIC_REG_ICR_HIGH, dest_apic_id << 24);
                // Write command to ICR low (triggers IPI)
                self.write_xapic(APIC_REG_ICR_LOW, icr_low);
            }
            ApicMode::X2Apic => {
                // In x2APIC, ICR is a single 64-bit register
                let icr = ((dest_apic_id as u64) << 32) | (icr_low as u64);
                let msr = X2APIC_MSR_BASE + (APIC_REG_ICR_LOW >> 4);
                crate::arch::x86_64::wrmsr(msr, icr);
            }
            ApicMode::Disabled => {}
        }
    }

    /// Wait for IPI delivery to complete
    ///
    /// Polls the ICR delivery status bit until the IPI has been sent.
    /// Should be called after send_ipi() for INIT and SIPI IPIs.
    ///
    /// # Safety
    ///
    /// May block for a short time (~10-20 microseconds).
    pub unsafe fn wait_ipi_delivery(&self) {
        match self.mode {
            ApicMode::XApic => {
                // Poll ICR delivery status bit (bit 12)
                while (self.read_xapic(APIC_REG_ICR_LOW) & ICR_DELIVERY_PENDING) != 0 {
                    core::hint::spin_loop();
                }
            }
            ApicMode::X2Apic => {
                // In x2APIC mode, polling is not required
                // The write to ICR is serializing
            }
            ApicMode::Disabled => {}
        }
    }
}

/// Global Local APIC instance (BSP only, per-CPU in M8)
pub static LOCAL_APIC: Mutex<Option<LocalApic>> = Mutex::new(None);

/// Detect APIC mode (xAPIC or x2APIC)
fn detect_apic_mode() -> Result<ApicMode, &'static str> {
    let cpuid = CpuId::new();
    let features = cpuid.get_feature_info()
        .ok_or("No CPUID feature info")?;

    // Check if APIC is available
    if !features.has_apic() {
        return Ok(ApicMode::Disabled);
    }

    // Check if x2APIC is available
    let has_x2apic = features.has_x2apic();

    // Read APIC_BASE MSR to check current mode
    let apic_base = unsafe { crate::arch::x86_64::rdmsr(IA32_APIC_BASE) };
    let x2apic_enabled = (apic_base & APIC_BASE_X2APIC) != 0;

    // Prefer x2APIC if available
    if has_x2apic {
        Ok(ApicMode::X2Apic)
    } else {
        Ok(ApicMode::XApic)
    }
}

/// Initialize the Local APIC
///
/// # Safety
///
/// Must be called once during boot, after PIC is disabled.
pub unsafe fn init() -> Result<(), &'static str> {
    let apic = LocalApic::new()?;

    let mode_str = match apic.mode() {
        ApicMode::XApic => "xAPIC (memory-mapped)",
        ApicMode::X2Apic => "x2APIC (MSR-based)",
        ApicMode::Disabled => "Disabled",
    };

    crate::arch::x86_64::serial::serial_write(b"[APIC] Local APIC initialized\n");
    crate::arch::x86_64::serial::serial_write(b"[APIC] Mode: ");
    crate::arch::x86_64::serial::serial_write(mode_str.as_bytes());
    crate::arch::x86_64::serial::serial_write(b"\n[APIC] APIC ID: ");
    print_u32(apic.id());
    crate::arch::x86_64::serial::serial_write(b"\n");

    *LOCAL_APIC.lock() = Some(apic);

    Ok(())
}

/// Send End of Interrupt
///
/// # Safety
///
/// Must be called exactly once per interrupt.
pub unsafe fn eoi() {
    if let Some(apic) = LOCAL_APIC.lock().as_ref() {
        apic.eoi();
    }
}

/// Get the Local APIC ID
pub fn local_apic_id() -> u32 {
    LOCAL_APIC.lock().as_ref().map(|apic| apic.id()).unwrap_or(0)
}

/// Get a reference to the Local APIC
///
/// Returns None if APIC is not initialized.
pub fn get() -> Option<spin::MutexGuard<'static, Option<LocalApic>>> {
    Some(LOCAL_APIC.lock())
}

/// Helper to print u32
fn print_u32(mut n: u32) {
    if n == 0 {
        crate::arch::x86_64::serial::serial_write(b"0");
        return;
    }

    let mut buf = [0u8; 10];
    let mut i = 0;

    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }

    while i > 0 {
        i -= 1;
        crate::arch::x86_64::serial::serial_write_byte(buf[i]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apic_mode_detection() {
        // APIC should be available on all modern x86_64 CPUs
        let mode = detect_apic_mode();
        assert!(mode.is_ok());

        let mode = mode.unwrap();
        assert!(mode == ApicMode::XApic || mode == ApicMode::X2Apic);
    }
}
