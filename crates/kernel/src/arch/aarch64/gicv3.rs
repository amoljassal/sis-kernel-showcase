//! ARM GICv3 (Generic Interrupt Controller version 3) driver
//!
//! This module provides support for the ARM GICv3 interrupt controller, which is used on:
//! - Raspberry Pi 5 (BCM2712)
//! - QEMU aarch64 virt machine
//! - Other modern ARM platforms
//!
//! # Architecture
//!
//! GICv3 consists of three main components:
//! 1. **Distributor (GICD)**: Global interrupt routing and configuration
//! 2. **Redistributor (GICR)**: Per-CPU interrupt configuration
//! 3. **CPU Interface**: System register-based interface for interrupt handling
//!
//! # Initialization Sequence
//!
//! 1. Initialize Distributor (GICD) - called once on boot CPU
//! 2. Initialize Redistributor (GICR) - called on each CPU
//! 3. Initialize CPU Interface - called on each CPU
//! 4. Enable individual interrupts as needed
//!
//! # Interrupt Types
//!
//! - **SGI (0-15)**: Software Generated Interrupts (inter-processor)
//! - **PPI (16-31)**: Private Peripheral Interrupts (per-CPU, e.g., timer)
//! - **SPI (32-1019)**: Shared Peripheral Interrupts (shared across CPUs)

use core::ptr::{read_volatile, write_volatile};

/// GICv3 Distributor (GICD) register offsets
const GICD_CTLR: usize = 0x0000;        // Distributor Control Register
const GICD_TYPER: usize = 0x0004;       // Interrupt Controller Type Register
const GICD_IIDR: usize = 0x0008;        // Distributor Implementer Identification Register
const GICD_IGROUPR: usize = 0x0080;     // Interrupt Group Registers
const GICD_ISENABLER: usize = 0x0100;   // Interrupt Set-Enable Registers
const GICD_ICENABLER: usize = 0x0180;   // Interrupt Clear-Enable Registers
const GICD_ISPENDR: usize = 0x0200;     // Interrupt Set-Pending Registers
const GICD_ICPENDR: usize = 0x0280;     // Interrupt Clear-Pending Registers
const GICD_ISACTIVER: usize = 0x0300;   // Interrupt Set-Active Registers
const GICD_ICACTIVER: usize = 0x0380;   // Interrupt Clear-Active Registers
const GICD_IPRIORITYR: usize = 0x0400;  // Interrupt Priority Registers
const GICD_ITARGETSR: usize = 0x0800;   // Interrupt Processor Targets Registers (GICv2 compat)
const GICD_ICFGR: usize = 0x0C00;       // Interrupt Configuration Registers
const GICD_IROUTER: usize = 0x6000;     // Interrupt Routing Registers (GICv3)

/// GICv3 Redistributor (GICR) register offsets
const GICR_CTLR: usize = 0x0000;        // Redistributor Control Register
const GICR_IIDR: usize = 0x0004;        // Redistributor Implementer Identification Register
const GICR_TYPER: usize = 0x0008;       // Redistributor Type Register
const GICR_WAKER: usize = 0x0014;       // Redistributor Wake Register

/// GICR SGI/PPI registers (offset from GICR base + 0x10000)
const GICR_IGROUPR0: usize = 0x0080;    // Interrupt Group Register 0
const GICR_ISENABLER0: usize = 0x0100;  // Interrupt Set-Enable Register 0
const GICR_ICENABLER0: usize = 0x0180;  // Interrupt Clear-Enable Register 0
const GICR_ISPENDR0: usize = 0x0200;    // Interrupt Set-Pending Register 0
const GICR_ICPENDR0: usize = 0x0280;    // Interrupt Clear-Pending Register 0
const GICR_IPRIORITYR: usize = 0x0400;  // Interrupt Priority Registers

/// GICD_CTLR register bits
const GICD_CTLR_ENABLE_G0: u32 = 1 << 0;  // Enable Group 0 interrupts
const GICD_CTLR_ENABLE_G1NS: u32 = 1 << 1; // Enable Group 1 non-secure interrupts
const GICD_CTLR_ENABLE_G1S: u32 = 1 << 2;  // Enable Group 1 secure interrupts
const GICD_CTLR_ARE_NS: u32 = 1 << 4;      // Affinity Routing Enable (Non-secure)
const GICD_CTLR_ARE_S: u32 = 1 << 5;       // Affinity Routing Enable (Secure)

/// GICR_WAKER register bits
const GICR_WAKER_PROCESSOR_SLEEP: u32 = 1 << 1;
const GICR_WAKER_CHILDREN_ASLEEP: u32 = 1 << 2;

/// Default interrupt priority
const DEFAULT_PRIORITY: u8 = 0xA0;

/// GICv3 controller state
pub struct GicV3 {
    gicd_base: usize,
    gicr_base: usize,
    initialized: bool,
}

impl GicV3 {
    /// Create a new GICv3 instance
    pub const fn new(gicd_base: usize, gicr_base: usize) -> Self {
        Self {
            gicd_base,
            gicr_base,
            initialized: false,
        }
    }

    /// Initialize the GIC Distributor (GICD)
    ///
    /// This should be called once on the boot CPU before any other GIC operations.
    pub unsafe fn init_distributor(&mut self) {
        crate::info!("GICv3: Initializing Distributor at {:#x}", self.gicd_base);

        // Disable distributor during configuration
        write_volatile((self.gicd_base + GICD_CTLR) as *mut u32, 0);

        // Read the number of implemented interrupt lines
        let typer = read_volatile((self.gicd_base + GICD_TYPER) as *const u32);
        let it_lines_number = (typer & 0x1F) as usize;
        let max_spi = (it_lines_number + 1) * 32;

        crate::info!("GICv3: Supports {} SPIs (INTID 32-{})", max_spi - 32, max_spi - 1);

        // Configure all SPIs as Group 1 (non-secure)
        // Each register covers 32 interrupts
        for i in 1..=it_lines_number {
            write_volatile(
                (self.gicd_base + GICD_IGROUPR + i * 4) as *mut u32,
                0xFFFFFFFF
            );
        }

        // Disable all SPI interrupts by default
        for i in 1..=it_lines_number {
            write_volatile(
                (self.gicd_base + GICD_ICENABLER + i * 4) as *mut u32,
                0xFFFFFFFF
            );
        }

        // Clear all pending SPIs
        for i in 1..=it_lines_number {
            write_volatile(
                (self.gicd_base + GICD_ICPENDR + i * 4) as *mut u32,
                0xFFFFFFFF
            );
        }

        // Set default priority for all SPIs
        for i in 32..max_spi {
            write_volatile(
                (self.gicd_base + GICD_IPRIORITYR + i) as *mut u8,
                DEFAULT_PRIORITY
            );
        }

        // Configure all SPIs as level-sensitive by default
        for i in 2..=(it_lines_number * 2) {
            write_volatile(
                (self.gicd_base + GICD_ICFGR + i * 4) as *mut u32,
                0x00000000  // Level-sensitive
            );
        }

        // Enable Affinity Routing and Group 1 non-secure interrupts
        write_volatile(
            (self.gicd_base + GICD_CTLR) as *mut u32,
            GICD_CTLR_ARE_NS | GICD_CTLR_ENABLE_G1NS
        );

        crate::info!("GICv3: Distributor initialized");
    }

    /// Initialize the GIC Redistributor for a specific CPU
    ///
    /// This should be called on each CPU during bring-up.
    ///
    /// # Arguments
    /// * `cpu_id` - The CPU ID (0-based)
    pub unsafe fn init_redistributor(&self, cpu_id: usize) {
        // Each redistributor has two 64KB frames:
        // Frame 0: RD_base (control registers)
        // Frame 1: SGI_base (SGI/PPI configuration)
        let rd_base = self.gicr_base + (cpu_id * 0x20000);
        let sgi_base = rd_base + 0x10000;

        crate::info!("GICv3: Initializing Redistributor for CPU {} at {:#x}", cpu_id, rd_base);

        // Check if this is the last redistributor
        let typer = read_volatile((rd_base + GICR_TYPER) as *const u64);
        let is_last = (typer & (1 << 4)) != 0;
        if is_last {
            crate::info!("GICv3: CPU {} is the last redistributor", cpu_id);
        }

        // Wake up the redistributor
        let mut waker = read_volatile((rd_base + GICR_WAKER) as *const u32);
        waker &= !GICR_WAKER_PROCESSOR_SLEEP;
        write_volatile((rd_base + GICR_WAKER) as *mut u32, waker);

        // Wait for ChildrenAsleep to clear
        let mut timeout = 100000;
        while timeout > 0 {
            let waker = read_volatile((rd_base + GICR_WAKER) as *const u32);
            if (waker & GICR_WAKER_CHILDREN_ASLEEP) == 0 {
                break;
            }
            timeout -= 1;
            core::hint::spin_loop();
        }

        if timeout == 0 {
            crate::warn!("GICv3: Redistributor wake timeout for CPU {}", cpu_id);
        }

        // Configure SGIs and PPIs as Group 1 non-secure
        write_volatile(
            (sgi_base + GICR_IGROUPR0) as *mut u32,
            0xFFFFFFFF
        );

        // Disable all SGIs and PPIs by default
        write_volatile(
            (sgi_base + GICR_ICENABLER0) as *mut u32,
            0xFFFFFFFF
        );

        // Clear all pending SGIs and PPIs
        write_volatile(
            (sgi_base + GICR_ICPENDR0) as *mut u32,
            0xFFFFFFFF
        );

        // Set default priority for SGIs and PPIs
        for i in 0..32 {
            write_volatile(
                (sgi_base + GICR_IPRIORITYR + i) as *mut u8,
                DEFAULT_PRIORITY
            );
        }

        crate::info!("GICv3: Redistributor initialized for CPU {}", cpu_id);
    }

    /// Initialize the CPU Interface using system registers
    ///
    /// This should be called on each CPU after redistributor initialization.
    pub unsafe fn init_cpu_interface(&self) {
        crate::info!("GICv3: Initializing CPU Interface");

        // Enable system register access to GIC
        // ICC_SRE_EL1: System Register Enable
        // - SRE (bit 0): Enable system register interface
        // - DIB (bit 1): Disable IRQ bypass
        // - DFB (bit 2): Disable FIQ bypass
        core::arch::asm!(
            "msr ICC_SRE_EL1, {sre}",
            "isb",
            sre = in(reg) 0x07u64,  // SRE | DIB | DFB
            options(nomem, nostack)
        );

        // Set priority mask to allow all priorities
        // ICC_PMR_EL1: Priority Mask Register
        // Set to 0xFF to allow all interrupt priorities
        core::arch::asm!(
            "msr ICC_PMR_EL1, {pmr}",
            pmr = in(reg) 0xFFu64,
            options(nomem, nostack)
        );

        // Set binary point register (no priority grouping)
        // ICC_BPR1_EL1: Binary Point Register for Group 1
        core::arch::asm!(
            "msr ICC_BPR1_EL1, {bpr}",
            bpr = in(reg) 0u64,
            options(nomem, nostack)
        );

        // Set EOI mode to drop priority and deactivate interrupt
        // ICC_CTLR_EL1: Control Register
        // EOImode (bit 1): 0 = priority drop and deactivate
        core::arch::asm!(
            "msr ICC_CTLR_EL1, {ctlr}",
            ctlr = in(reg) 0u64,
            options(nomem, nostack)
        );

        // Enable Group 1 interrupts
        // ICC_IGRPEN1_EL1: Interrupt Group 1 Enable Register
        core::arch::asm!(
            "msr ICC_IGRPEN1_EL1, {en}",
            "isb",
            en = in(reg) 1u64,
            options(nomem, nostack)
        );

        crate::info!("GICv3: CPU Interface initialized");
    }

    /// Enable an interrupt
    ///
    /// # Arguments
    /// * `irq` - Interrupt number (0-1019)
    pub unsafe fn enable_irq(&self, irq: u32) {
        if irq < 32 {
            // SGI/PPI: enable in redistributor
            let sgi_base = self.gicr_base + 0x10000;
            write_volatile(
                (sgi_base + GICR_ISENABLER0) as *mut u32,
                1 << irq
            );
        } else {
            // SPI: enable in distributor
            let reg = (irq / 32) as usize;
            let bit = irq % 32;
            write_volatile(
                (self.gicd_base + GICD_ISENABLER + reg * 4) as *mut u32,
                1 << bit
            );
        }
    }

    /// Disable an interrupt
    ///
    /// # Arguments
    /// * `irq` - Interrupt number (0-1019)
    pub unsafe fn disable_irq(&self, irq: u32) {
        if irq < 32 {
            // SGI/PPI: disable in redistributor
            let sgi_base = self.gicr_base + 0x10000;
            write_volatile(
                (sgi_base + GICR_ICENABLER0) as *mut u32,
                1 << irq
            );
        } else {
            // SPI: disable in distributor
            let reg = (irq / 32) as usize;
            let bit = irq % 32;
            write_volatile(
                (self.gicd_base + GICD_ICENABLER + reg * 4) as *mut u32,
                1 << bit
            );
        }
    }

    /// Acknowledge an interrupt
    ///
    /// Returns the interrupt ID. Special values:
    /// - 1020-1023: Reserved/spurious
    ///
    /// # Safety
    /// Must be called from an IRQ exception handler
    pub unsafe fn ack_irq(&self) -> u32 {
        let intid: u64;
        core::arch::asm!(
            "mrs {}, ICC_IAR1_EL1",
            out(reg) intid,
            options(nomem, nostack)
        );
        intid as u32
    }

    /// End of Interrupt - signal that interrupt handling is complete
    ///
    /// # Arguments
    /// * `irq` - The interrupt number returned by `ack_irq()`
    ///
    /// # Safety
    /// Must be called from an IRQ exception handler after handling the interrupt
    pub unsafe fn eoi_irq(&self, irq: u32) {
        core::arch::asm!(
            "msr ICC_EOIR1_EL1, {}",
            in(reg) irq as u64,
            options(nomem, nostack)
        );
    }

    /// Set interrupt priority
    ///
    /// # Arguments
    /// * `irq` - Interrupt number
    /// * `priority` - Priority value (0-255, lower is higher priority)
    pub unsafe fn set_priority(&self, irq: u32, priority: u8) {
        if irq < 32 {
            // SGI/PPI: set in redistributor
            let sgi_base = self.gicr_base + 0x10000;
            write_volatile(
                (sgi_base + GICR_IPRIORITYR + irq as usize) as *mut u8,
                priority
            );
        } else {
            // SPI: set in distributor
            write_volatile(
                (self.gicd_base + GICD_IPRIORITYR + irq as usize) as *mut u8,
                priority
            );
        }
    }
}

/// Global GICv3 instance
static mut GIC: Option<GicV3> = None;

/// Initialize the GICv3 controller
///
/// This should be called once during early boot on the boot CPU.
///
/// # Safety
/// Must be called exactly once during boot before enabling interrupts.
pub unsafe fn init() {
    let desc = crate::platform::active().gic();
    let mut gic = GicV3::new(desc.gicd, desc.gicr);

    // Initialize distributor (global, once)
    gic.init_distributor();

    // Initialize redistributor and CPU interface for CPU 0
    gic.init_redistributor(0);
    gic.init_cpu_interface();

    gic.initialized = true;
    GIC = Some(gic);

    crate::info!("GICv3: Initialization complete");
}

/// Initialize GICv3 for a secondary CPU
///
/// This should be called on each secondary CPU during SMP bring-up.
///
/// # Arguments
/// * `cpu_id` - The CPU ID (0-based)
///
/// # Safety
/// Must be called on each CPU before that CPU enables interrupts.
pub unsafe fn init_cpu(cpu_id: usize) {
    if let Some(ref gic) = GIC {
        gic.init_redistributor(cpu_id);
        gic.init_cpu_interface();
        crate::info!("GICv3: CPU {} initialized", cpu_id);
    } else {
        crate::warn!("GICv3: Cannot initialize CPU {} - GIC not initialized", cpu_id);
    }
}

/// Enable an interrupt
///
/// # Arguments
/// * `irq` - Interrupt number (0-1019)
pub fn enable_irq(irq: u32) {
    unsafe {
        if let Some(ref gic) = GIC {
            gic.enable_irq(irq);
        }
    }
}

/// Enable an interrupt (checked version that returns success status)
///
/// # Arguments
/// * `irq` - Interrupt number (0-1019)
///
/// # Returns
/// Some(()) if GIC is initialized and IRQ was enabled, None otherwise
pub fn enable_irq_checked(irq: u32) -> Option<()> {
    unsafe {
        if let Some(ref gic) = GIC {
            gic.enable_irq(irq);
            Some(())
        } else {
            None
        }
    }
}

/// Disable an interrupt
///
/// # Arguments
/// * `irq` - Interrupt number (0-1019)
pub fn disable_irq(irq: u32) {
    unsafe {
        if let Some(ref gic) = GIC {
            gic.disable_irq(irq);
        }
    }
}

/// Handle an IRQ exception
///
/// Returns the interrupt ID that was handled.
///
/// # Safety
/// Must be called from the IRQ exception vector
pub unsafe fn handle_irq() -> u32 {
    if let Some(ref gic) = GIC {
        gic.ack_irq()
    } else {
        1023 // Spurious interrupt
    }
}

/// Signal end of interrupt handling
///
/// # Arguments
/// * `irq` - The interrupt number returned by `handle_irq()`
///
/// # Safety
/// Must be called after interrupt handling is complete
pub unsafe fn eoi_irq(irq: u32) {
    if let Some(ref gic) = GIC {
        gic.eoi_irq(irq);
    }
}

/// Set interrupt priority
///
/// # Arguments
/// * `irq` - Interrupt number
/// * `priority` - Priority value (0-255, lower is higher priority)
pub fn set_priority(irq: u32, priority: u8) {
    unsafe {
        if let Some(ref gic) = GIC {
            gic.set_priority(irq, priority);
        }
    }
}
