// Helper for neuralctl status and small utilities

impl super::Shell {
    pub(crate) fn neuralctl_status(&self) {
        crate::neural::print_status();
    }

    pub(crate) fn neuralctl_reset(&self) {
        crate::neural::reset();
        unsafe { crate::uart_print(b"[NN] reset\n"); }
    }

    pub(crate) fn neuralctl_infer(&self, args: &[&str]) {
        if args.len() < 1 {
            unsafe { crate::uart_print(b"Usage: neuralctl infer <m1 m2 ...> (values in milli)\n"); }
            return;
        }
        let mut vals: heapless::Vec<i32, 32> = heapless::Vec::new();
        for a in args {
            if let Ok(v) = a.parse::<i32>() { let _ = vals.push(v); }
        }
        let n = crate::neural::infer_from_milli(&vals);
        crate::neural::print_status();
        unsafe { crate::uart_print(b"[NN] out_len="); }
        self.print_number_simple(n as u64);
        unsafe { crate::uart_print(b"\n"); }
    }

    pub(crate) fn neuralctl_update(&self, args: &[&str]) {
        if args.is_empty() {
            unsafe { crate::uart_print(b"Usage: neuralctl update <weights in milli: w1(h*in),b1(h),w2(out*h),b2(out)>\n"); }
            return;
        }
        let mut vals: heapless::Vec<i32, 1024> = heapless::Vec::new();
        for a in args {
            if let Ok(v) = a.parse::<i32>() { let _ = vals.push(v); }
        }
        if crate::neural::update_from_milli(&vals) {
            unsafe { crate::uart_print(b"[NN] weights updated\n"); }
        } else {
            unsafe { crate::uart_print(b"[NN] update failed (count mismatch)\n"); }
        }
    }

    pub(crate) fn neuralctl_teach(&self, args: &[&str]) {
        if args.is_empty() { unsafe { crate::uart_print(b"Usage: neuralctl teach <i...>|<t...> (milli)\n"); } return; }
        let mut inputs: heapless::Vec<i32, 32> = heapless::Vec::new();
        let mut targets: heapless::Vec<i32, 32> = heapless::Vec::new();
        let mut sep = false;
        for a in args {
            if *a == "|" { sep = true; continue; }
            if let Ok(v) = a.parse::<i32>() {
                if !sep { let _ = inputs.push(v); } else { let _ = targets.push(v); }
            }
        }
        let ok = crate::neural::teach_milli(&inputs, &targets);
        unsafe { crate::uart_print(if ok { b"[NN] teach ok\n" } else { b"[NN] teach failed\n" }); }
    }

    pub(crate) fn neuralctl_selftest(&self) {
        let ok = crate::neural::selftest();
        unsafe { crate::uart_print(if ok { b"[NN] selftest: PASS\n" } else { b"[NN] selftest: FAIL\n" }); }
        crate::neural::print_status();
    }

    pub(crate) fn neuralctl_learn(&self, args: &[&str]) {
        if args.is_empty() { unsafe { crate::uart_print(b"Usage: neuralctl learn on|off [limit N]\n"); } return; }
        match args[0] {
            "on" => {
                let mut limit: Option<usize> = None;
                if args.len() >= 3 && args[1] == "limit" {
                    if let Ok(v) = args[2].parse::<usize>() { limit = Some(v); }
                }
                crate::neural::learn_set(true, limit);
                unsafe { crate::uart_print(b"[NN] learn: ON\n"); }
            }
            "off" => { crate::neural::learn_set(false, None); unsafe { crate::uart_print(b"[NN] learn: OFF\n"); } }
            _ => unsafe { crate::uart_print(b"Usage: neuralctl learn on|off [limit N]\n"); }
        }
    }

    pub(crate) fn neuralctl_tick(&self) {
        let applied = crate::neural::learn_tick();
        unsafe { crate::uart_print(b"[NN] tick applied="); }
        self.print_number_simple(applied as u64);
        unsafe { crate::uart_print(b"\n"); }
    }

    pub(crate) fn neuralctl_dump(&self) {
        crate::neural::dump_milli();
    }

    pub(crate) fn neuralctl_load(&self, args: &[&str]) {
        let mut i = 0usize;
        if args.len() < 4 { unsafe { crate::uart_print(b"Usage: neuralctl load <in> <hid> <out> | <weights...>\n"); } return; }
        let di = match args[i].parse::<usize>() { Ok(v)=>v, Err(_)=>{ unsafe{ crate::uart_print(b"[NN] bad in\n"); } return; } }; i+=1;
        let dh = match args[i].parse::<usize>() { Ok(v)=>v, Err(_)=>{ unsafe{ crate::uart_print(b"[NN] bad hid\n"); } return; } }; i+=1;
        let do_ = match args[i].parse::<usize>() { Ok(v)=>v, Err(_)=>{ unsafe{ crate::uart_print(b"[NN] bad out\n"); } return; } }; i+=1;
        if args[i] != "|" { unsafe { crate::uart_print(b"[NN] expect '|' before weights\n"); } return; }
        i += 1;
        let mut weights: heapless::Vec<i32, 1024> = heapless::Vec::new();
        while i < args.len() {
            if let Ok(v) = args[i].parse::<i32>() { let _ = weights.push(v); }
            i += 1;
        }
        if crate::neural::load_all_milli((di, dh, do_), &weights) {
            unsafe { crate::uart_print(b"[NN] load ok\n"); }
        } else {
            unsafe { crate::uart_print(b"[NN] load failed\n"); }
        }
    }
}
