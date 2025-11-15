// Helpers for PMU (Performance Monitoring Unit)
//
// M8 Hardening: Error handling for all PMU operations

use crate::drivers::DriverError;

impl super::Shell {
    /// Display current PMU statistics
    pub(crate) fn pmu_stats_cmd(&self) {
        if !crate::pmu::is_initialized() {
            unsafe { crate::uart_print(b"[PMU] Not initialized\n"); }
            return;
        }

        let snap = match crate::pmu::read_snapshot() {
            Ok(s) => s,
            Err(e) => {
                self.print_pmu_error(e, b"Failed to read PMU snapshot");
                return;
            }
        };

        unsafe {
            crate::uart_print(b"[PMU] Performance Monitoring Unit Statistics\n");
            crate::uart_print(b"============================================\n\n");

            // Cycle counter
            crate::uart_print(b"Cycles:              ");
            self.print_number_simple(snap.cycles);
            crate::uart_print(b"\n");

            // Event counters
            crate::uart_print(b"Instructions:        ");
            self.print_number_simple(snap.inst);
            crate::uart_print(b"\n");

            crate::uart_print(b"L1D Cache Refill:    ");
            self.print_number_simple(snap.l1d_refill);
            crate::uart_print(b"\n");

            crate::uart_print(b"Branch Mispred:      ");
            self.print_number_simple(snap.branch_mispred);
            crate::uart_print(b"\n");

            crate::uart_print(b"L2D Cache Access:    ");
            self.print_number_simple(snap.l2d_cache);
            crate::uart_print(b"\n");

            crate::uart_print(b"L1I Cache Refill:    ");
            self.print_number_simple(snap.l1i_refill);
            crate::uart_print(b"\n");

            crate::uart_print(b"Exceptions Taken:    ");
            self.print_number_simple(snap.exc_taken);
            crate::uart_print(b"\n\n");

            // Derived metrics
            crate::uart_print(b"Derived Metrics:\n");
            crate::uart_print(b"----------------\n");

            // Instructions per cycle (IPC)
            let ipc = snap.ipc();
            crate::uart_print(b"IPC (Inst/Cycle):    ");
            self.print_float(ipc);
            crate::uart_print(b"\n");

            // L1D miss rate
            let l1d_rate = snap.l1d_miss_rate();
            crate::uart_print(b"L1D Miss Rate:       ");
            self.print_float(l1d_rate);
            crate::uart_print(b"%\n");

            // Branch miss rate
            let br_rate = snap.branch_miss_rate();
            crate::uart_print(b"Branch Miss Rate:    ");
            self.print_float(br_rate);
            crate::uart_print(b"%\n");
        }
    }

    /// Run PMU benchmark (busy loop with measurements)
    pub(crate) fn pmu_demo_cmd(&self) {
        if !crate::pmu::is_initialized() {
            unsafe { crate::uart_print(b"[PMU] Not initialized\n"); }
            return;
        }

        unsafe {
            crate::uart_print(b"[PMU] Running benchmark: 8192 iterations of busy loop\n");
        }

        // Take snapshot before
        let s0 = match crate::pmu::read_snapshot() {
            Ok(s) => s,
            Err(e) => {
                self.print_pmu_error(e, b"Failed to read initial snapshot");
                return;
            }
        };

        // Busy loop with some memory access
        let mut acc: u64 = 0;
        let mut buf: [u64; 128] = [0; 128];
        for i in 0..8192 {
            acc = acc.wrapping_mul(6364136223846793005).wrapping_add(1);
            let idx = (i & 127) as usize;
            buf[idx] = buf[idx].wrapping_add(acc ^ (i as u64));
        }
        unsafe { core::ptr::read_volatile(&acc); } // Prevent optimization

        // Take snapshot after
        let s1 = match crate::pmu::read_snapshot() {
            Ok(s) => s,
            Err(e) => {
                self.print_pmu_error(e, b"Failed to read final snapshot");
                return;
            }
        };

        // Calculate deltas
        let d_cycles = s1.cycles.saturating_sub(s0.cycles);
        let d_inst = s1.inst.saturating_sub(s0.inst);
        let d_l1d = s1.l1d_refill.saturating_sub(s0.l1d_refill);
        let d_branch = s1.branch_mispred.saturating_sub(s0.branch_mispred);
        let d_l2d = s1.l2d_cache.saturating_sub(s0.l2d_cache);

        unsafe {
            crate::uart_print(b"\n[PMU] Benchmark Results:\n");
            crate::uart_print(b"========================\n");
            crate::uart_print(b"Cycles:          ");
            self.print_number_simple(d_cycles);
            crate::uart_print(b"\n");

            crate::uart_print(b"Instructions:    ");
            self.print_number_simple(d_inst);
            crate::uart_print(b"\n");

            crate::uart_print(b"L1D Refill:      ");
            self.print_number_simple(d_l1d);
            crate::uart_print(b"\n");

            crate::uart_print(b"Branch Mispred:  ");
            self.print_number_simple(d_branch);
            crate::uart_print(b"\n");

            crate::uart_print(b"L2D Access:      ");
            self.print_number_simple(d_l2d);
            crate::uart_print(b"\n\n");

            if d_cycles > 0 {
                let ipc = d_inst as f64 / d_cycles as f64;
                crate::uart_print(b"IPC:             ");
                self.print_float(ipc);
                crate::uart_print(b"\n");
            }

            if d_inst == 0 {
                crate::uart_print(b"[PMU] Note: instructions counter may be unsupported in this environment\n");
            }
        }
    }

    /// Helper to print floating point number
    fn print_float(&self, value: f64) {
        let integer_part = value as u64;
        let fractional_part = ((value - integer_part as f64) * 100.0) as u64;

        unsafe {
            self.print_number_simple(integer_part);
            crate::uart_print(b".");
            if fractional_part < 10 {
                crate::uart_print(b"0");
            }
            self.print_number_simple(fractional_part);
        }
    }

    /// Print PMU error message
    /// M8 Hardening: Comprehensive error reporting for PMU operations
    fn print_pmu_error(&self, error: DriverError, context: &[u8]) {
        unsafe {
            crate::uart_print(b"[PMU ERROR] ");
            crate::uart_print(context);
            crate::uart_print(b": ");

            match error {
                DriverError::NotInitialized => crate::uart_print(b"PMU not initialized\n"),
                DriverError::InvalidParameter => crate::uart_print(b"Invalid counter index (valid: 0-5)\n"),
                DriverError::HardwareError => crate::uart_print(b"Hardware error\n"),
                _ => {
                    crate::uart_print(b"Error code ");
                    self.print_number_simple(error.code() as u64);
                    crate::uart_print(b"\n");
                }
            }
        }
    }
}

