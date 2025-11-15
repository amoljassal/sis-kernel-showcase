//! RISC-V Interrupt Management
//!
//! Advanced Interrupt Architecture (AIA) implementation with legacy PLIC fallback.
//! Follows the research-backed approach prioritizing AIA for scalable interrupt handling.
//!
//! Research Basis:
//! - RISC-V AIA Specification v1.0
//! - QEMU AIA implementation studies  
//! - Scalable interrupt handling research

use core::arch::asm;
use crate::arch::riscv64::mmu::{PhysAddr, VirtAddr};

/// Interrupt controller interface
pub trait InterruptController {
    fn init(&mut self) -> Result<(), InterruptError>;
    fn enable_interrupt(&mut self, irq: u32) -> Result<(), InterruptError>;
    fn disable_interrupt(&mut self, irq: u32) -> Result<(), InterruptError>;
    fn set_priority(&mut self, irq: u32, priority: u8) -> Result<(), InterruptError>;
    fn handle_interrupt(&mut self) -> Option<u32>;
    fn end_interrupt(&mut self, irq: u32) -> Result<(), InterruptError>;
}

/// Interrupt management errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterruptError {
    InvalidIrq,
    NotSupported,
    HardwareError,
    AlreadyEnabled,
    NotEnabled,
}

/// Advanced Interrupt Architecture (AIA) Controller
/// Implements the RISC-V AIA specification for scalable interrupt handling
pub struct AIAController {
    aplic_base: PhysAddr,
    imsic_base: PhysAddr,
    num_sources: u32,
    hart_count: u32,
}

impl AIAController {
    /// Create new AIA controller instance
    pub fn new(aplic_base: PhysAddr, imsic_base: PhysAddr) -> Self {
        Self {
            aplic_base,
            imsic_base,
            num_sources: 256, // Default, will be read from hardware
            hart_count: 4,    // Default, will be detected
        }
    }

    /// Read from APLIC register
    fn read_aplic_reg(&self, offset: u32) -> u32 {
        unsafe {
            let addr = (self.aplic_base.0 + offset as u64) as *const u32;
            core::ptr::read_volatile(addr)
        }
    }

    /// Write to APLIC register
    fn write_aplic_reg(&self, offset: u32, value: u32) {
        unsafe {
            let addr = (self.aplic_base.0 + offset as u64) as *mut u32;
            core::ptr::write_volatile(addr, value);
        }
    }

    /// Read from IMSIC register for specific hart
    fn read_imsic_reg(&self, hart: u32, offset: u32) -> u64 {
        unsafe {
            let addr = (self.imsic_base.0 + (hart as u64 * 0x1000) + offset as u64) as *const u64;
            core::ptr::read_volatile(addr)
        }
    }

    /// Write to IMSIC register for specific hart
    fn write_imsic_reg(&self, hart: u32, offset: u32, value: u64) {
        unsafe {
            let addr = (self.imsic_base.0 + (hart as u64 * 0x1000) + offset as u64) as *mut u64;
            core::ptr::write_volatile(addr, value);
        }
    }

    /// Initialize APLIC (Advanced Platform-Level Interrupt Controller)
    fn init_aplic(&mut self) -> Result<(), InterruptError> {
        // Check APLIC configuration
        let config = self.read_aplic_reg(0x0000); // Configuration register
        self.num_sources = (config >> 16) & 0xFFFF;

        // Set delivery mode to MSI (Message Signaled Interrupts)
        self.write_aplic_reg(0x0004, 1); // domaincfg register

        // Configure source priorities (default to lowest)
        for irq in 1..=self.num_sources {
            self.write_aplic_reg(0x0004 + irq * 4, 1); // sourcecfg[irq]
        }

        // Configure target addresses for MSI delivery
        for hart in 0..self.hart_count {
            let target_addr = self.imsic_base.0 + (hart as u64 * 0x1000);
            let target_reg = 0x3004 + hart * 8;
            self.write_aplic_reg(target_reg, target_addr as u32);
            self.write_aplic_reg(target_reg + 4, (target_addr >> 32) as u32);
        }

        Ok(())
    }

    /// Initialize IMSIC (Incoming Message Signaled Interrupt Controller)
    fn init_imsic(&mut self) -> Result<(), InterruptError> {
        // Initialize IMSIC for each hart
        for hart in 0..self.hart_count {
            // Enable external interrupt file
            self.write_imsic_reg(hart, 0x0070, 1); // eidelivery register
            
            // Clear all pending interrupts
            self.write_imsic_reg(hart, 0x1C00, 0xFFFFFFFFFFFFFFFF); // eip0 register
            
            // Set interrupt thresholds
            self.write_imsic_reg(hart, 0x0080, 0); // eithreshold register
        }

        Ok(())
    }

    /// Get current hart ID
    fn current_hart_id(&self) -> u32 {
        let hart_id: u64;
        unsafe {
            asm!("mv {}, tp", out(reg) hart_id);
        }
        hart_id as u32
    }
}

impl InterruptController for AIAController {
    fn init(&mut self) -> Result<(), InterruptError> {
        self.init_aplic()?;
        self.init_imsic()?;

        // Enable supervisor external interrupts
        unsafe {
            asm!("csrs sie, {}", in(reg) 1 << 9); // SEIE bit
        }

        Ok(())
    }

    fn enable_interrupt(&mut self, irq: u32) -> Result<(), InterruptError> {
        if irq == 0 || irq > self.num_sources {
            return Err(InterruptError::InvalidIrq);
        }

        // Enable interrupt source in APLIC
        let enable_reg = 0x2000 + (irq / 32) * 4;
        let enable_bit = 1u32 << (irq % 32);
        let current_value = self.read_aplic_reg(enable_reg);
        self.write_aplic_reg(enable_reg, current_value | enable_bit);

        // Enable in IMSIC for current hart
        let hart = self.current_hart_id();
        let imsic_enable_reg = 0x1800 + (irq / 64) * 8;
        let imsic_enable_bit = 1u64 << (irq % 64);
        let current_imsic = self.read_imsic_reg(hart, imsic_enable_reg as u32);
        self.write_imsic_reg(hart, imsic_enable_reg as u32, current_imsic | imsic_enable_bit);

        Ok(())
    }

    fn disable_interrupt(&mut self, irq: u32) -> Result<(), InterruptError> {
        if irq == 0 || irq > self.num_sources {
            return Err(InterruptError::InvalidIrq);
        }

        // Disable interrupt source in APLIC
        let enable_reg = 0x2000 + (irq / 32) * 4;
        let enable_bit = 1u32 << (irq % 32);
        let current_value = self.read_aplic_reg(enable_reg);
        self.write_aplic_reg(enable_reg, current_value & !enable_bit);

        Ok(())
    }

    fn set_priority(&mut self, irq: u32, priority: u8) -> Result<(), InterruptError> {
        if irq == 0 || irq > self.num_sources {
            return Err(InterruptError::InvalidIrq);
        }

        // Set priority in APLIC source configuration
        let priority_reg = 0x0004 + irq * 4;
        let current_config = self.read_aplic_reg(priority_reg);
        let new_config = (current_config & !0xFF) | (priority as u32);
        self.write_aplic_reg(priority_reg, new_config);

        Ok(())
    }

    fn handle_interrupt(&mut self) -> Option<u32> {
        let hart = self.current_hart_id();

        // Check IMSIC for pending interrupts
        let pending = self.read_imsic_reg(hart, 0x1C00); // eip0 register
        
        if pending != 0 {
            // Find highest priority pending interrupt
            let irq = pending.trailing_zeros();
            Some(irq)
        } else {
            None
        }
    }

    fn end_interrupt(&mut self, irq: u32) -> Result<(), InterruptError> {
        if irq > self.num_sources {
            return Err(InterruptError::InvalidIrq);
        }

        let hart = self.current_hart_id();

        // Clear pending bit in IMSIC
        let clear_reg = 0x1C00 + (irq / 64) * 8;
        let clear_bit = 1u64 << (irq % 64);
        self.write_imsic_reg(hart, clear_reg as u32, clear_bit);

        Ok(())
    }
}

/// Legacy PLIC (Platform-Level Interrupt Controller)
/// Fallback implementation for systems without AIA support
pub struct PLICController {
    base_addr: PhysAddr,
    num_sources: u32,
    hart_count: u32,
}

impl PLICController {
    /// Create new PLIC controller instance
    pub fn new(base_addr: PhysAddr) -> Self {
        Self {
            base_addr,
            num_sources: 256,
            hart_count: 4,
        }
    }

    /// Read from PLIC register
    fn read_reg(&self, offset: u32) -> u32 {
        unsafe {
            let addr = (self.base_addr.0 + offset as u64) as *const u32;
            core::ptr::read_volatile(addr)
        }
    }

    /// Write to PLIC register
    fn write_reg(&self, offset: u32, value: u32) {
        unsafe {
            let addr = (self.base_addr.0 + offset as u64) as *mut u32;
            core::ptr::write_volatile(addr, value);
        }
    }

    /// Get context offset for hart and privilege mode
    fn context_offset(&self, hart: u32, privilege: u32) -> u32 {
        0x200000 + (hart * 2 + privilege) * 0x1000
    }
}

impl InterruptController for PLICController {
    fn init(&mut self) -> Result<(), InterruptError> {
        // Set all interrupt priorities to 1 (minimum non-zero)
        for irq in 1..=self.num_sources {
            self.write_reg(irq * 4, 1);
        }

        // Set interrupt thresholds to 0 for all contexts
        for hart in 0..self.hart_count {
            let s_context = self.context_offset(hart, 1); // Supervisor mode
            self.write_reg(s_context, 0); // threshold register
        }

        // Enable supervisor external interrupts
        unsafe {
            asm!("csrs sie, {}", in(reg) 1 << 9); // SEIE bit
        }

        Ok(())
    }

    fn enable_interrupt(&mut self, irq: u32) -> Result<(), InterruptError> {
        if irq == 0 || irq > self.num_sources {
            return Err(InterruptError::InvalidIrq);
        }

        // Enable interrupt for supervisor mode on hart 0
        let hart = 0; // For simplicity, only support hart 0 in PLIC mode
        let s_context = self.context_offset(hart, 1);
        let enable_reg = s_context + 0x80 + (irq / 32) * 4;
        let enable_bit = 1u32 << (irq % 32);
        let current_value = self.read_reg(enable_reg);
        self.write_reg(enable_reg, current_value | enable_bit);

        Ok(())
    }

    fn disable_interrupt(&mut self, irq: u32) -> Result<(), InterruptError> {
        if irq == 0 || irq > self.num_sources {
            return Err(InterruptError::InvalidIrq);
        }

        let hart = 0;
        let s_context = self.context_offset(hart, 1);
        let enable_reg = s_context + 0x80 + (irq / 32) * 4;
        let enable_bit = 1u32 << (irq % 32);
        let current_value = self.read_reg(enable_reg);
        self.write_reg(enable_reg, current_value & !enable_bit);

        Ok(())
    }

    fn set_priority(&mut self, irq: u32, priority: u8) -> Result<(), InterruptError> {
        if irq == 0 || irq > self.num_sources {
            return Err(InterruptError::InvalidIrq);
        }

        self.write_reg(irq * 4, priority as u32);
        Ok(())
    }

    fn handle_interrupt(&mut self) -> Option<u32> {
        let hart = 0;
        let s_context = self.context_offset(hart, 1);
        let claim_reg = s_context + 4;
        let irq = self.read_reg(claim_reg);
        
        if irq != 0 {
            Some(irq)
        } else {
            None
        }
    }

    fn end_interrupt(&mut self, irq: u32) -> Result<(), InterruptError> {
        let hart = 0;
        let s_context = self.context_offset(hart, 1);
        let complete_reg = s_context + 4;
        self.write_reg(complete_reg, irq);
        Ok(())
    }
}

/// Global interrupt controller instance
static mut INTERRUPT_CONTROLLER: Option<InterruptControllerType> = None;

/// Enum for different interrupt controller types
enum InterruptControllerType {
    AIA(AIAController),
    PLIC(PLICController),
}

/// Initialize interrupt controller based on hardware support
pub fn init_interrupt_controller() -> Result<(), InterruptError> {
    unsafe {
        // Try to detect AIA support first
        #[cfg(feature = "aia")]
        {
            // In a real implementation, these addresses would be read from device tree
            let aplic_base = PhysAddr(0x0C000000);
            let imsic_base = PhysAddr(0x28000000);
            
            let mut aia_controller = AIAController::new(aplic_base, imsic_base);
            match aia_controller.init() {
                Ok(()) => {
                    INTERRUPT_CONTROLLER = Some(InterruptControllerType::AIA(aia_controller));
                    return Ok(());
                }
                Err(_) => {
                    // Fall back to PLIC
                }
            }
        }

        // Fall back to PLIC
        let plic_base = PhysAddr(0x0C000000); // QEMU virt PLIC address
        let mut plic_controller = PLICController::new(plic_base);
        plic_controller.init()?;
        INTERRUPT_CONTROLLER = Some(InterruptControllerType::PLIC(plic_controller));
    }

    Ok(())
}

/// Get reference to global interrupt controller
pub fn get_interrupt_controller() -> &'static mut dyn InterruptController {
    unsafe {
        match INTERRUPT_CONTROLLER.as_mut().expect("Interrupt controller not initialized") {
            InterruptControllerType::AIA(controller) => controller,
            InterruptControllerType::PLIC(controller) => controller,
        }
    }
}

/// RISC-V trap handler (called from assembly)
#[no_mangle]
pub extern "C" fn riscv_trap_handler() {
    let scause: u64;
    let sepc: u64;
    let stval: u64;

    unsafe {
        asm!("csrr {}, scause", out(reg) scause);
        asm!("csrr {}, sepc", out(reg) sepc);
        asm!("csrr {}, stval", out(reg) stval);
    }

    // Check if this is an interrupt (MSB set)
    if (scause & (1 << 63)) != 0 {
        let interrupt_code = scause & 0x7FFFFFFFFFFFFFFF;
        
        match interrupt_code {
            9 => {
                // Supervisor external interrupt
                if let Some(irq) = get_interrupt_controller().handle_interrupt() {
                    // Handle the specific interrupt
                    handle_device_interrupt(irq);
                    
                    // End interrupt processing
                    let _ = get_interrupt_controller().end_interrupt(irq);
                }
            }
            5 => {
                // Supervisor timer interrupt
                handle_timer_interrupt();
            }
            _ => {
                // Unknown interrupt
                unsafe {
                    crate::uart_print(b"Unknown interrupt\n");
                }
            }
        }
    } else {
        // Handle exceptions (page faults, illegal instructions, etc.)
        handle_exception(scause, sepc, stval);
    }
}

/// Handle device-specific interrupts
fn handle_device_interrupt(_irq: u32) {
    // Device interrupt handling would go here
    // For now, just print debug info
    unsafe {
        crate::uart_print(b"Device interrupt: ");
    }
    // Note: In a real implementation, we'd have proper number printing
}

/// Handle timer interrupts
fn handle_timer_interrupt() {
    // Clear timer interrupt
    unsafe {
        asm!("csrc sip, {}", in(reg) 1 << 5); // STIP bit
    }
    
    // Timer interrupt handling would go here
    // This is where scheduler context switching would happen
}

/// Handle exceptions (page faults, illegal instructions, etc.)
fn handle_exception(scause: u64, _sepc: u64, _stval: u64) {
    unsafe {
        match scause {
            12 => {
                // Instruction page fault
                crate::uart_print(b"Instruction page fault\n");
            }
            13 => {
                // Load page fault
                crate::uart_print(b"Load page fault\n");
            }
            15 => {
                // Store page fault
                crate::uart_print(b"Store page fault\n");
            }
            _ => {
                crate::uart_print(b"Unknown exception\n");
            }
        }
    }

    // For now, halt on exceptions
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plic_context_offset() {
        let plic = PLICController::new(PhysAddr(0));
        assert_eq!(plic.context_offset(0, 1), 0x201000); // Hart 0, supervisor mode
        assert_eq!(plic.context_offset(1, 1), 0x203000); // Hart 1, supervisor mode
    }
}