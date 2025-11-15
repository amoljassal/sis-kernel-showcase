// Signal handling infrastructure for Phase A1
// Implements basic POSIX signal delivery and handling

use crate::lib::error::{Errno, Result};
use alloc::collections::VecDeque;
use core::sync::atomic::{AtomicU64, Ordering};

pub type Pid = u32;

/// Signal numbers (POSIX standard)
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    SIGHUP = 1,    // Hangup
    SIGINT = 2,    // Interrupt (Ctrl-C)
    SIGQUIT = 3,   // Quit
    SIGILL = 4,    // Illegal instruction
    SIGTRAP = 5,   // Trace/breakpoint trap
    SIGABRT = 6,   // Abort
    SIGBUS = 7,    // Bus error
    SIGFPE = 8,    // Floating point exception
    SIGKILL = 9,   // Kill (uncatchable)
    SIGUSR1 = 10,  // User-defined signal 1
    SIGSEGV = 11,  // Segmentation fault
    SIGUSR2 = 12,  // User-defined signal 2
    SIGPIPE = 13,  // Broken pipe
    SIGALRM = 14,  // Alarm clock
    SIGTERM = 15,  // Termination
    SIGCHLD = 17,  // Child status changed
    SIGCONT = 18,  // Continue if stopped
    SIGSTOP = 19,  // Stop (uncatchable)
    SIGTSTP = 20,  // Stop (Ctrl-Z)
    SIGTTIN = 21,  // Background read from TTY
    SIGTTOU = 22,  // Background write to TTY
}

impl Signal {
    pub fn from_u32(signo: u32) -> Option<Self> {
        match signo {
            1 => Some(Signal::SIGHUP),
            2 => Some(Signal::SIGINT),
            3 => Some(Signal::SIGQUIT),
            4 => Some(Signal::SIGILL),
            5 => Some(Signal::SIGTRAP),
            6 => Some(Signal::SIGABRT),
            7 => Some(Signal::SIGBUS),
            8 => Some(Signal::SIGFPE),
            9 => Some(Signal::SIGKILL),
            10 => Some(Signal::SIGUSR1),
            11 => Some(Signal::SIGSEGV),
            12 => Some(Signal::SIGUSR2),
            13 => Some(Signal::SIGPIPE),
            14 => Some(Signal::SIGALRM),
            15 => Some(Signal::SIGTERM),
            17 => Some(Signal::SIGCHLD),
            18 => Some(Signal::SIGCONT),
            19 => Some(Signal::SIGSTOP),
            20 => Some(Signal::SIGTSTP),
            21 => Some(Signal::SIGTTIN),
            22 => Some(Signal::SIGTTOU),
            _ => None,
        }
    }

    pub fn to_u32(self) -> u32 {
        self as u32
    }

    /// Check if signal can be caught/blocked/ignored
    pub fn is_catchable(self) -> bool {
        !matches!(self, Signal::SIGKILL | Signal::SIGSTOP)
    }

    /// Get default action for signal
    pub fn default_action(self) -> SignalAction {
        match self {
            Signal::SIGCHLD | Signal::SIGCONT => SignalAction::Ignore,
            Signal::SIGSTOP | Signal::SIGTSTP | Signal::SIGTTIN | Signal::SIGTTOU => {
                SignalAction::Stop
            }
            Signal::SIGCONT => SignalAction::Continue,
            _ => SignalAction::Terminate,
        }
    }
}

/// Signal action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalAction {
    /// Ignore signal
    Ignore,
    /// Terminate process
    Terminate,
    /// Stop process
    Stop,
    /// Continue process (if stopped)
    Continue,
    /// Call user handler at this address
    Handler(u64),
}

/// Signal handler registration (for sigaction)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SigAction {
    /// Handler function pointer or SIG_DFL/SIG_IGN
    pub sa_handler: u64,
    /// Signal mask to block during handler
    pub sa_mask: u64,
    /// Flags (SA_RESTART, SA_SIGINFO, etc.)
    pub sa_flags: i32,
    /// Restorer function (for sigreturn)
    pub sa_restorer: u64,
}

impl Default for SigAction {
    fn default() -> Self {
        Self {
            sa_handler: 0, // SIG_DFL
            sa_mask: 0,
            sa_flags: 0,
            sa_restorer: 0,
        }
    }
}

/// Signal queue for pending signals
pub struct SignalQueue {
    /// Pending signals (bitset of signal numbers)
    pending: AtomicU64,
    /// Blocked signals (bitset of signal numbers)
    blocked: AtomicU64,
    /// Signal handlers (indexed by signal number)
    handlers: [SignalAction; 32],
}

impl SignalQueue {
    pub fn new() -> Self {
        Self {
            pending: AtomicU64::new(0),
            blocked: AtomicU64::new(0),
            handlers: [SignalAction::Terminate; 32],
        }
    }

    /// Add a signal to the pending set
    pub fn add_signal(&self, sig: Signal) {
        let bit = 1u64 << (sig.to_u32() - 1);
        self.pending.fetch_or(bit, Ordering::SeqCst);
    }

    /// Remove a signal from the pending set
    pub fn remove_signal(&self, sig: Signal) {
        let bit = 1u64 << (sig.to_u32() - 1);
        self.pending.fetch_and(!bit, Ordering::SeqCst);
    }

    /// Check if a signal is pending
    pub fn is_pending(&self, sig: Signal) -> bool {
        let bit = 1u64 << (sig.to_u32() - 1);
        (self.pending.load(Ordering::SeqCst) & bit) != 0
    }

    /// Get the next pending, unblocked signal
    pub fn next_pending(&self) -> Option<Signal> {
        let pending = self.pending.load(Ordering::SeqCst);
        let blocked = self.blocked.load(Ordering::SeqCst);
        let deliverable = pending & !blocked;

        if deliverable == 0 {
            return None;
        }

        // Find lowest bit set (lowest signal number)
        let signo = deliverable.trailing_zeros() + 1;
        Signal::from_u32(signo)
    }

    /// Block a signal
    pub fn block(&self, sig: Signal) {
        let bit = 1u64 << (sig.to_u32() - 1);
        self.blocked.fetch_or(bit, Ordering::SeqCst);
    }

    /// Unblock a signal
    pub fn unblock(&self, sig: Signal) {
        let bit = 1u64 << (sig.to_u32() - 1);
        self.blocked.fetch_and(!bit, Ordering::SeqCst);
    }

    /// Set signal handler
    pub fn set_handler(&mut self, sig: Signal, action: SignalAction) {
        let idx = (sig.to_u32() - 1) as usize;
        if idx < 32 {
            self.handlers[idx] = action;
        }
    }

    /// Get signal handler
    pub fn get_handler(&self, sig: Signal) -> SignalAction {
        let idx = (sig.to_u32() - 1) as usize;
        if idx < 32 {
            self.handlers[idx]
        } else {
            SignalAction::Terminate
        }
    }

    /// Clear all pending signals
    pub fn clear_all(&self) {
        self.pending.store(0, Ordering::SeqCst);
    }
}

impl Clone for SignalQueue {
    fn clone(&self) -> Self {
        Self {
            pending: AtomicU64::new(self.pending.load(Ordering::SeqCst)),
            blocked: AtomicU64::new(self.blocked.load(Ordering::SeqCst)),
            handlers: self.handlers,
        }
    }
}

/// Send a signal to a process
pub fn send_signal(pid: Pid, sig: Signal) -> Result<()> {
    let mut table = crate::process::get_process_table();
    let table = table.as_mut().ok_or(Errno::ESRCH)?;
    let task = table.get_mut(pid).ok_or(Errno::ESRCH)?;

    // Add signal to pending set
    task.signals.add_signal(sig);

    // If process is sleeping, wake it up
    if task.state == crate::process::ProcessState::Sleeping {
        task.state = crate::process::ProcessState::Ready;
        // TODO: Add to run queue
    }

    Ok(())
}

/// Deliver pending signals to current process
/// Called before returning to userspace from syscall or interrupt
pub fn deliver_signals() {
    let pid = crate::process::current_pid();
    let mut table = crate::process::get_process_table();
    let Some(ref mut table) = *table else {
        return;
    };
    let Some(task) = table.get_mut(pid) else {
        return;
    };

    // Get next deliverable signal
    let Some(sig) = task.signals.next_pending() else {
        return;
    };

    // Remove from pending set
    task.signals.remove_signal(sig);

    // Get action
    let action = task.signals.get_handler(sig);

    match action {
        SignalAction::Ignore => {
            // Do nothing
        }
        SignalAction::Terminate => {
            // Terminate process
            crate::info!("Process {} terminated by signal {:?}", pid, sig);
            task.exit_code = 128 + sig.to_u32() as i32;
            task.state = crate::process::ProcessState::Zombie;
            // Notify parent
            if task.ppid != 0 {
                let _ = send_signal(task.ppid, Signal::SIGCHLD);
            }
        }
        SignalAction::Stop => {
            // Stop process (Phase A1: minimal support)
            task.state = crate::process::ProcessState::Stopped;
        }
        SignalAction::Continue => {
            // Continue process if stopped
            if task.state == crate::process::ProcessState::Stopped {
                task.state = crate::process::ProcessState::Ready;
            }
        }
        SignalAction::Handler(handler_addr) => {
            // Set up signal frame to call user handler
            // Phase A1: minimal implementation
            // Save current trap frame
            let saved_pc = task.trap_frame.pc;
            let saved_sp = task.trap_frame.sp;

            // Set PC to handler
            task.trap_frame.pc = handler_addr;

            // Set x0 to signal number (first arg)
            task.trap_frame.x0 = sig.to_u32() as u64;

            // Push return address (for sigreturn)
            // Simplified for A1: store in task for later
            // TODO: Create proper signal frame on user stack
        }
    }
}

/// Constants for sigaction
pub const SIG_DFL: u64 = 0; // Default action
pub const SIG_IGN: u64 = 1; // Ignore signal

/// Flags for sigaction
pub const SA_RESTART: i32 = 0x10000000;
pub const SA_SIGINFO: i32 = 0x00000004;
