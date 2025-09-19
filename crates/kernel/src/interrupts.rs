use spin::Mutex;
use core::sync::atomic::{AtomicU32, Ordering};

pub const MAX_IRQ_HANDLERS: usize = 256;

pub trait InterruptHandler: Send + Sync {
    fn handle_interrupt(&self, irq_number: u32);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterruptError {
    InvalidIrq,
    HandlerNotFound,
    RegistrationFailed,
}

pub struct InterruptRegistry {
    handlers: [Option<&'static dyn InterruptHandler>; MAX_IRQ_HANDLERS],
    registered_count: AtomicU32,
}

impl InterruptRegistry {
    pub const fn new() -> Self {
        Self {
            handlers: [None; MAX_IRQ_HANDLERS],
            registered_count: AtomicU32::new(0),
        }
    }

    pub fn register_handler(
        &mut self,
        irq_number: u32,
        handler: &'static dyn InterruptHandler,
    ) -> Result<(), InterruptError> {
        if irq_number >= MAX_IRQ_HANDLERS as u32 {
            return Err(InterruptError::InvalidIrq);
        }

        let idx = irq_number as usize;
        if self.handlers[idx].is_some() {
            return Err(InterruptError::RegistrationFailed);
        }

        self.handlers[idx] = Some(handler);
        self.registered_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    pub fn unregister_handler(&mut self, irq_number: u32) -> Result<(), InterruptError> {
        if irq_number >= MAX_IRQ_HANDLERS as u32 {
            return Err(InterruptError::InvalidIrq);
        }

        let idx = irq_number as usize;
        if self.handlers[idx].is_none() {
            return Err(InterruptError::HandlerNotFound);
        }

        self.handlers[idx] = None;
        self.registered_count.fetch_sub(1, Ordering::SeqCst);
        Ok(())
    }

    pub fn handle_interrupt(&self, irq_number: u32) -> Result<(), InterruptError> {
        if irq_number >= MAX_IRQ_HANDLERS as u32 {
            return Err(InterruptError::InvalidIrq);
        }

        let idx = irq_number as usize;
        if let Some(handler) = self.handlers[idx] {
            handler.handle_interrupt(irq_number);
            Ok(())
        } else {
            Err(InterruptError::HandlerNotFound)
        }
    }

    pub fn get_registered_count(&self) -> u32 {
        self.registered_count.load(Ordering::SeqCst)
    }
}

unsafe impl Send for InterruptRegistry {}
unsafe impl Sync for InterruptRegistry {}

static INTERRUPT_REGISTRY: Mutex<InterruptRegistry> = Mutex::new(InterruptRegistry::new());

pub fn register_interrupt_handler(
    irq_number: u32,
    handler: &'static dyn InterruptHandler,
) -> Result<(), InterruptError> {
    let mut registry = INTERRUPT_REGISTRY.lock();
    registry.register_handler(irq_number, handler)
}

pub fn unregister_interrupt_handler(irq_number: u32) -> Result<(), InterruptError> {
    let mut registry = INTERRUPT_REGISTRY.lock();
    registry.unregister_handler(irq_number)
}

pub fn dispatch_interrupt(irq_number: u32) -> Result<(), InterruptError> {
    let registry = INTERRUPT_REGISTRY.lock();
    registry.handle_interrupt(irq_number)
}

pub fn get_interrupt_handler_count() -> u32 {
    let registry = INTERRUPT_REGISTRY.lock();
    registry.get_registered_count()
}