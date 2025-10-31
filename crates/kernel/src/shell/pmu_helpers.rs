// Helpers for PMU demo

impl super::Shell {
    pub(crate) fn pmu_demo_cmd(&self) {
        #[cfg(feature = "perf-verbose")]
        {
            unsafe { crate::uart_print(b"[PMU] Demo: setup events and run busy loop\n"); }
            unsafe { crate::pmu::aarch64::setup_events(); }
            let s0 = unsafe { crate::pmu::aarch64::read_snapshot() };
            let mut acc: u64 = 0;
            let mut buf: [u64; 128] = [0; 128];
            for i in 0..8192 {
                acc = acc.wrapping_mul(6364136223846793005).wrapping_add(1);
                let idx = (i & 127) as usize;
                buf[idx] = buf[idx].wrapping_add(acc ^ (i as u64));
            }
            unsafe { core::ptr::read_volatile(&acc); }
            let s1 = unsafe { crate::pmu::aarch64::read_snapshot() };
            let d_cycles = s1.cycles.saturating_sub(s0.cycles);
            let d_inst = s1.inst.saturating_sub(s0.inst);
            let d_l1d = s1.l1d_refill.saturating_sub(s0.l1d_refill);
            unsafe {
                crate::uart_print(b"METRIC pmu_cycles="); self.print_number_simple(d_cycles);
                crate::uart_print(b"\nMETRIC pmu_inst="); self.print_number_simple(d_inst);
                crate::uart_print(b"\nMETRIC pmu_l1d_refill="); self.print_number_simple(d_l1d);
                crate::uart_print(b"\n");
            }
            if d_inst == 0 {
                unsafe { crate::uart_print(b"[PMU] Note: instructions counter may be unsupported in this QEMU build\n"); }
            }
        }
        #[cfg(not(feature = "perf-verbose"))]
        unsafe {
            crate::uart_print(b"[PMU] perf-verbose feature not enabled\n");
        }
    }
}

