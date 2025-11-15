//! # Legacy PIC (8259A Programmable Interrupt Controller)
//!
//! This module provides support for the legacy 8259A PIC (Programmable Interrupt Controller),
//! also known as the PIC or "Intel 8259". The 8259A is a cascaded pair of interrupt controllers
//! that handle hardware interrupts (IRQs) on x86 systems.
//!
//! ## Historical Context
//!
//! The 8259A PIC was introduced in 1976 and became the standard interrupt controller for
//! IBM PC-compatible systems. Modern systems use the APIC (Advanced Programmable Interrupt
//! Controller), but the PIC is still present for backward compatibility and is often used
//! during early boot before the APIC is initialized.
//!
//! ## Architecture
//!
//! The PC uses two 8259A PICs in a cascaded configuration:
//!
//! ```text
//! Master PIC (0x20-0x21)          Slave PIC (0xA0-0xA1)
//! ┌─────────────────┐             ┌─────────────────┐
//! │ IRQ 0 - Timer   │             │ IRQ  8 - RTC    │
//! │ IRQ 1 - Keyboard│             │ IRQ  9 - Free   │
//! │ IRQ 2 - Cascade │────────────►│ IRQ 10 - Free   │
//! │ IRQ 3 - COM2    │             │ IRQ 11 - Free   │
//! │ IRQ 4 - COM1    │             │ IRQ 12 - Mouse  │
//! │ IRQ 5 - LPT2    │             │ IRQ 13 - FPU    │
//! │ IRQ 6 - Floppy  │             │ IRQ 14 - IDE 1  │
//! │ IRQ 7 - LPT1    │             │ IRQ 15 - IDE 2  │
//! └─────────────────┘             └─────────────────┘
//! ```
//!
//! IRQ 2 on the master PIC is connected to the slave PIC, effectively providing
//! 15 usable IRQs (IRQ 0-1, 3-7, 8-15).
//!
//! ## I/O Ports
//!
//! ### Master PIC
//! - **0x20**: Command port
//! - **0x21**: Data port (IMR - Interrupt Mask Register)
//!
//! ### Slave PIC
//! - **0xA0**: Command port
//! - **0xA1**: Data port (IMR - Interrupt Mask Register)
//!
//! ## Interrupt Vector Remapping
//!
//! By default, the PIC maps IRQs to vectors 0-15, which conflicts with CPU exceptions
//! (also vectors 0-31). We remap the PIC to use vectors 32-47:
//! - Master PIC: IRQ 0-7 → Vectors 32-39
//! - Slave PIC: IRQ 8-15 → Vectors 40-47
//!
//! ## Initialization Sequence (ICW 1-4)
//!
//! The PIC is initialized using Initialization Command Words (ICW 1-4):
//!
//! 1. **ICW1**: Start initialization sequence, set mode
//! 2. **ICW2**: Set interrupt vector offset
//! 3. **ICW3**: Set cascade configuration
//! 4. **ICW4**: Set operating mode (8086 mode, auto-EOI, etc.)
//!
//! ## End of Interrupt (EOI)
//!
//! After handling an interrupt, the PIC must be sent an EOI (End of Interrupt) command:
//! - **Non-specific EOI** (0x20): Use for master PIC or when cascading is clear
//! - **Specific EOI** (0x60 + IRQ): Specify which IRQ to acknowledge
//!
//! For slave PIC interrupts (IRQ 8-15), both the slave and master must receive EOI.
//!
//! ## Migration to APIC
//!
//! Modern kernels disable the PIC after initializing the APIC:
//! 1. Initialize PIC (this module)
//! 2. Use PIC for early boot (timer, keyboard)
//! 3. Initialize APIC (M2)
//! 4. Disable PIC (mask all interrupts)
//! 5. Use APIC for all future interrupts
//!
//! ## Safety Considerations
//!
//! - PIC programming uses I/O ports, requiring `unsafe` code
//! - Incorrect initialization can cause spurious interrupts
//! - Forgetting EOI causes interrupt controller to hang
//! - Disabling PIC while still using it causes lost interrupts

use spin::Mutex;
use x86_64::instructions::port::Port;

/// Master PIC command port (0x20)
const PIC1_COMMAND: u16 = 0x20;

/// Master PIC data port (0x21)
const PIC1_DATA: u16 = 0x21;

/// Slave PIC command port (0xA0)
const PIC2_COMMAND: u16 = 0xA0;

/// Slave PIC data port (0xA1)
const PIC2_DATA: u16 = 0xA1;

/// End of Interrupt command
const CMD_EOI: u8 = 0x20;

/// Initialization Command Word 1 (ICW1)
const ICW1_INIT: u8 = 0x10;  // Initialization
const ICW1_ICW4: u8 = 0x01;  // ICW4 needed

/// Initialization Command Word 4 (ICW4)
const ICW4_8086: u8 = 0x01;  // 8086/8088 mode

/// Default vector offset for master PIC (IRQ 0-7 → vectors 32-39)
pub const PIC1_OFFSET: u8 = 32;

/// Default vector offset for slave PIC (IRQ 8-15 → vectors 40-47)
pub const PIC2_OFFSET: u8 = PIC1_OFFSET + 8;

/// Hardware IRQ numbers (0-15)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Irq {
    Timer = 0,          // IRQ 0 - PIT timer
    Keyboard = 1,       // IRQ 1 - PS/2 keyboard
    Cascade = 2,        // IRQ 2 - Cascade to slave PIC (never raised)
    COM2 = 3,           // IRQ 3 - Serial port COM2
    COM1 = 4,           // IRQ 4 - Serial port COM1
    LPT2 = 5,           // IRQ 5 - Parallel port LPT2
    Floppy = 6,         // IRQ 6 - Floppy disk controller
    LPT1 = 7,           // IRQ 7 - Parallel port LPT1 (spurious)
    RTC = 8,            // IRQ 8 - Real-time clock
    ACPI = 9,           // IRQ 9 - ACPI (or free)
    Free1 = 10,         // IRQ 10 - Free/available
    Free2 = 11,         // IRQ 11 - Free/available
    Mouse = 12,         // IRQ 12 - PS/2 mouse
    FPU = 13,           // IRQ 13 - Floating-point unit
    PrimaryATA = 14,    // IRQ 14 - Primary ATA (IDE)
    SecondaryATA = 15,  // IRQ 15 - Secondary ATA (IDE)
}

impl Irq {
    /// Convert IRQ number to interrupt vector
    pub fn to_vector(self) -> u8 {
        let irq = self as u8;
        if irq < 8 {
            PIC1_OFFSET + irq
        } else {
            PIC2_OFFSET + (irq - 8)
        }
    }

    /// Convert interrupt vector to IRQ number
    pub fn from_vector(vector: u8) -> Option<Self> {
        let irq = if vector >= PIC1_OFFSET && vector < PIC1_OFFSET + 8 {
            vector - PIC1_OFFSET
        } else if vector >= PIC2_OFFSET && vector < PIC2_OFFSET + 8 {
            (vector - PIC2_OFFSET) + 8
        } else {
            return None;
        };

        match irq {
            0 => Some(Irq::Timer),
            1 => Some(Irq::Keyboard),
            2 => Some(Irq::Cascade),
            3 => Some(Irq::COM2),
            4 => Some(Irq::COM1),
            5 => Some(Irq::LPT2),
            6 => Some(Irq::Floppy),
            7 => Some(Irq::LPT1),
            8 => Some(Irq::RTC),
            9 => Some(Irq::ACPI),
            10 => Some(Irq::Free1),
            11 => Some(Irq::Free2),
            12 => Some(Irq::Mouse),
            13 => Some(Irq::FPU),
            14 => Some(Irq::PrimaryATA),
            15 => Some(Irq::SecondaryATA),
            _ => None,
        }
    }
}

/// Chained PIC (8259A) controller
///
/// Manages both master and slave PICs as a single unit.
pub struct ChainedPics {
    master_command: Port<u8>,
    master_data: Port<u8>,
    slave_command: Port<u8>,
    slave_data: Port<u8>,
    master_offset: u8,
    slave_offset: u8,
}

impl ChainedPics {
    /// Create a new chained PIC controller
    ///
    /// # Arguments
    ///
    /// * `master_offset` - Interrupt vector offset for master PIC (typically 32)
    /// * `slave_offset` - Interrupt vector offset for slave PIC (typically 40)
    pub const fn new(master_offset: u8, slave_offset: u8) -> Self {
        Self {
            master_command: Port::new(PIC1_COMMAND),
            master_data: Port::new(PIC1_DATA),
            slave_command: Port::new(PIC2_COMMAND),
            slave_data: Port::new(PIC2_DATA),
            master_offset,
            slave_offset,
        }
    }

    /// Initialize both PICs with remapped vectors
    ///
    /// This function performs the complete initialization sequence:
    /// 1. Save current interrupt masks
    /// 2. Send ICW1 (start initialization)
    /// 3. Send ICW2 (vector offsets)
    /// 4. Send ICW3 (cascade configuration)
    /// 5. Send ICW4 (operation mode)
    /// 6. Restore interrupt masks (all disabled initially)
    ///
    /// # Safety
    ///
    /// Must be called with interrupts disabled. After initialization, interrupts
    /// must be explicitly enabled using `enable_irq()`.
    pub unsafe fn initialize(&mut self) {
        // Save current masks
        let mask1 = self.master_data.read();
        let mask2 = self.slave_data.read();

        // Start initialization sequence (ICW1)
        self.master_command.write(ICW1_INIT | ICW1_ICW4);
        io_wait();
        self.slave_command.write(ICW1_INIT | ICW1_ICW4);
        io_wait();

        // Set vector offsets (ICW2)
        self.master_data.write(self.master_offset);
        io_wait();
        self.slave_data.write(self.slave_offset);
        io_wait();

        // Configure cascade (ICW3)
        // Master: IRQ 2 has slave
        // Slave: Cascade identity (IRQ 2)
        self.master_data.write(0x04); // 0000_0100 = IRQ 2
        io_wait();
        self.slave_data.write(0x02);  // 0000_0010 = cascade identity 2
        io_wait();

        // Set mode (ICW4)
        self.master_data.write(ICW4_8086);
        io_wait();
        self.slave_data.write(ICW4_8086);
        io_wait();

        // Restore masks (disable all interrupts initially)
        self.master_data.write(0xFF); // Mask all
        self.slave_data.write(0xFF);  // Mask all
    }

    /// Enable a specific IRQ
    ///
    /// Unmasks the specified IRQ in the appropriate PIC.
    ///
    /// # Safety
    ///
    /// The caller must ensure a handler is registered for this IRQ before enabling it.
    pub unsafe fn enable_irq(&mut self, irq: u8) {
        if irq < 8 {
            // Master PIC
            let mask = self.master_data.read();
            self.master_data.write(mask & !(1 << irq));
        } else {
            // Slave PIC
            let irq = irq - 8;
            let mask = self.slave_data.read();
            self.slave_data.write(mask & !(1 << irq));

            // Also enable cascade on master
            let master_mask = self.master_data.read();
            self.master_data.write(master_mask & !(1 << 2));
        }
    }

    /// Disable a specific IRQ
    ///
    /// Masks the specified IRQ in the appropriate PIC.
    pub unsafe fn disable_irq(&mut self, irq: u8) {
        if irq < 8 {
            // Master PIC
            let mask = self.master_data.read();
            self.master_data.write(mask | (1 << irq));
        } else {
            // Slave PIC
            let irq = irq - 8;
            let mask = self.slave_data.read();
            self.slave_data.write(mask | (1 << irq));
        }
    }

    /// Send End of Interrupt (EOI) signal
    ///
    /// This must be called at the end of every interrupt handler to signal
    /// that interrupt processing is complete.
    ///
    /// For slave PIC interrupts (IRQ 8-15), EOI must be sent to both PICs.
    ///
    /// # Safety
    ///
    /// Must be called exactly once per interrupt. Calling it too many times
    /// can cause spurious interrupts.
    pub unsafe fn notify_end_of_interrupt(&mut self, vector: u8) {
        // Determine if this is a slave PIC interrupt
        let is_slave = vector >= self.slave_offset && vector < self.slave_offset + 8;

        if is_slave {
            // Send EOI to slave
            self.slave_command.write(CMD_EOI);
        }

        // Always send EOI to master (for master IRQs and cascade)
        self.master_command.write(CMD_EOI);
    }

    /// Disable both PICs
    ///
    /// Masks all interrupts on both PICs. This should be called when
    /// transitioning to APIC mode (M2).
    pub unsafe fn disable(&mut self) {
        self.master_data.write(0xFF);
        self.slave_data.write(0xFF);
    }

    /// Check if a spurious IRQ occurred
    ///
    /// Spurious interrupts can occur on IRQ 7 (master) or IRQ 15 (slave).
    /// This function checks the In-Service Register (ISR) to determine if
    /// the interrupt was real or spurious.
    ///
    /// Returns `true` if the interrupt was spurious.
    pub unsafe fn is_spurious(&mut self, vector: u8) -> bool {
        if vector == self.master_offset + 7 {
            // Check master PIC ISR
            self.master_command.write(0x0B); // Read ISR
            let isr = self.master_command.read();
            return (isr & 0x80) == 0;
        } else if vector == self.slave_offset + 7 {
            // Check slave PIC ISR
            self.slave_command.write(0x0B); // Read ISR
            let isr = self.slave_command.read();
            return (isr & 0x80) == 0;
        }
        false
    }
}

/// Global PIC instance
pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe {
    ChainedPics::new(PIC1_OFFSET, PIC2_OFFSET)
});

/// Initialize the legacy PIC
///
/// This function initializes both PICs with remapped interrupt vectors
/// (32-47 instead of 0-15 to avoid conflicts with CPU exceptions).
///
/// All interrupts are initially masked. Use `enable_irq()` to enable
/// specific interrupts.
///
/// # Safety
///
/// Must be called exactly once during boot, with interrupts disabled.
pub unsafe fn init() {
    PICS.lock().initialize();

    crate::arch::x86_64::serial::serial_write(b"[PIC] Legacy 8259A PIC initialized\n");
    crate::arch::x86_64::serial::serial_write(b"[PIC] Master: IRQ 0-7 -> Vectors 32-39\n");
    crate::arch::x86_64::serial::serial_write(b"[PIC] Slave:  IRQ 8-15 -> Vectors 40-47\n");
}

/// Enable a specific IRQ
///
/// # Safety
///
/// A handler must be registered in the IDT before enabling the IRQ.
pub unsafe fn enable_irq(irq: Irq) {
    PICS.lock().enable_irq(irq as u8);
}

/// Disable a specific IRQ
pub unsafe fn disable_irq(irq: Irq) {
    PICS.lock().disable_irq(irq as u8);
}

/// Send End of Interrupt signal
///
/// Must be called at the end of every hardware interrupt handler.
///
/// # Safety
///
/// Must be called exactly once per interrupt.
pub unsafe fn end_of_interrupt(vector: u8) {
    PICS.lock().notify_end_of_interrupt(vector);
}

/// Disable both PICs
///
/// Call this when transitioning to APIC mode.
pub unsafe fn disable() {
    PICS.lock().disable();
    crate::arch::x86_64::serial::serial_write(b"[PIC] Legacy PIC disabled\n");
}

/// I/O wait - short delay for PIC programming
///
/// The PIC requires a small delay between commands. We use an I/O port
/// write to port 0x80 (unused diagnostic port) as a short delay.
#[inline]
fn io_wait() {
    unsafe {
        Port::<u8>::new(0x80).write(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_irq_vector_conversion() {
        assert_eq!(Irq::Timer.to_vector(), 32);
        assert_eq!(Irq::Keyboard.to_vector(), 33);
        assert_eq!(Irq::COM1.to_vector(), 36);
        assert_eq!(Irq::RTC.to_vector(), 40);
        assert_eq!(Irq::Mouse.to_vector(), 44);
    }

    #[test]
    fn test_vector_to_irq() {
        assert_eq!(Irq::from_vector(32), Some(Irq::Timer));
        assert_eq!(Irq::from_vector(33), Some(Irq::Keyboard));
        assert_eq!(Irq::from_vector(40), Some(Irq::RTC));
        assert_eq!(Irq::from_vector(44), Some(Irq::Mouse));
        assert_eq!(Irq::from_vector(100), None);
    }
}
