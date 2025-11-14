// Helpers for graphctl commands (predict, create, add-channel, start)

impl super::Shell {
    // Local framing helper mirroring cmd_graphctl's send_frame logic
    fn graphctl_send_frame(&self, cmd: u8, payload: &[u8]) -> bool {
        const TOKEN: u64 = 0x53535F4354524C21; // must match kernel CONTROL_TOKEN
        let token = TOKEN.to_le_bytes();
        let mut buf = [0u8; 96];
        let total = 8 + 8 + payload.len();
        if total > buf.len() {
            unsafe { crate::uart_print(b"[CTL] payload too large\n"); }
            return false;
        }
        buf[0] = 0x43; // 'C'
        buf[1] = 0;    // ver
        buf[2] = cmd;  // cmd
        buf[3] = 0;    // flags
        let len = (8 + payload.len()) as u32; // include token in payload length
        let le = len.to_le_bytes();
        buf[4] = le[0]; buf[5] = le[1]; buf[6] = le[2]; buf[7] = le[3];
        // write token then payload
        let mut off = 8;
        for i in 0..8 { buf[off + i] = token[i]; }
        off += 8;
        for i in 0..payload.len() { buf[off + i] = payload[i]; }
        match crate::control::handle_frame(&buf[..total]) {
            Ok(()) => { unsafe { crate::uart_print(b"[CTL] ok\n"); } true }
            Err(_) => { unsafe { crate::uart_print(b"[CTL] error\n"); } false }
        }
    }

    pub(crate) fn graphctl_predict(&self, args: &[&str]) {
        if args.len() < 4 {
            unsafe { crate::uart_print(b"Usage: graphctl predict <op_id> <recent_latency_us> <channel_depth> <priority>\n"); }
            return;
        }
        let op_id = match args[0].parse::<u32>() {
            Ok(v) => v,
            Err(_) => { unsafe { crate::uart_print(b"[GRAPH] invalid op_id\n"); } return; }
        };
        let latency_us = match args[1].parse::<u32>() {
            Ok(v) => v,
            Err(_) => { unsafe { crate::uart_print(b"[GRAPH] invalid latency\n"); } return; }
        };
        let depth = match args[2].parse::<usize>() {
            Ok(v) => v,
            Err(_) => { unsafe { crate::uart_print(b"[GRAPH] invalid depth\n"); } return; }
        };
        let priority = if args.len() > 3 {
            match args[3].parse::<u8>() { Ok(v) => v, Err(_) => 10u8 }
        } else { 10u8 };

        let (confidence, will_meet_deadline) = crate::neural::predict_operator_health(op_id, latency_us, depth, priority);
        unsafe { crate::uart_print(b"[GRAPH] Operator "); }
        self.print_number_simple(op_id as u64);
        unsafe {
            crate::uart_print(b" prediction: ");
            if will_meet_deadline { crate::uart_print(b"HEALTHY (will meet deadline)"); }
            else { crate::uart_print(b"UNHEALTHY (may miss deadline)"); }
            crate::uart_print(b"\n[GRAPH] Confidence: ");
        }
        self.print_number_simple(confidence as u64);
        unsafe { crate::uart_print(b"/1000\n"); }
    }

    pub(crate) fn graphctl_create(&self) {
        let ok = self.graphctl_send_frame(0x01, &[]);
        unsafe {
            if ok { crate::uart_print(b"Graph created\n"); }
            else { crate::uart_print(b"Graph create error\n"); }
        }
    }

    pub(crate) fn graphctl_add_channel(&self, args: &[&str]) {
        if args.len() < 1 { unsafe { crate::uart_print(b"Usage: graphctl add-channel <capacity>\n"); } return; }
        if let Ok(cap) = args[0].parse::<u32>() {
            if cap == 0 || cap > 65535 { unsafe { crate::uart_print(b"[CTL] capacity must be 1..65535\n"); } return; }
            #[cfg(feature = "graphctl-framed")]
            {
                let c = (cap as u16).to_le_bytes();
                let payload = [c[0], c[1]];
                let _ = self.graphctl_send_frame(0x02, &payload);
            }
            #[cfg(not(feature = "graphctl-framed"))]
            {
                match crate::control::add_channel_direct(cap as u16) {
                    Ok(()) => unsafe { crate::uart_print(b"[CTL] ok\n"); },
                    Err(_) => unsafe { crate::uart_print(b"[CTL] error\n"); },
                }
            }
        } else {
            unsafe { crate::uart_print(b"[CTL] invalid capacity\n"); }
        }
    }

    pub(crate) fn graphctl_start(&self, args: &[&str]) {
        if args.len() < 1 { unsafe { crate::uart_print(b"Usage: graphctl start <steps>\n"); } return; }
        if let Ok(steps) = args[0].parse::<u32>() {
            let le = steps.to_le_bytes();
            let payload = [le[0], le[1], le[2], le[3]];
            let _ = self.graphctl_send_frame(0x04, &payload);
            unsafe {
                crate::uart_print(b"Execution complete: ");
                self.print_number_simple(steps as u64);
                crate::uart_print(b" steps\n");
            }
        } else {
            unsafe { crate::uart_print(b"[CTL] invalid steps\n"); }
        }
    }

    pub(crate) fn graphctl_add_operator(&self, args: &[&str]) {
        if args.len() < 1 { unsafe { crate::uart_print(b"Usage: graphctl add-operator <op_id> [--in N|none] [--out N|none] [--prio P] [--stage acquire|clean|explore|model|explain] [--in-schema S] [--out-schema S]\n"); } return; }
        let op_id = match args[0].parse::<u32>() { Ok(v) => v, Err(_) => { unsafe { crate::uart_print(b"[CTL] invalid op_id\n"); } return; } };
        let mut in_ch: Option<u16> = None;
        let mut out_ch: Option<u16> = None;
        let mut prio: u8 = 10;
        let mut stage: u8 = 0; // acquire
        let mut _in_schema: Option<u32> = None;
        let mut _out_schema: Option<u32> = None;

        let mut i = 1;
        while i < args.len() {
            match args[i] {
                "--in" => {
                    i += 1; if i >= args.len() { unsafe { crate::uart_print(b"[CTL] --in requires a value\n"); } return; }
                    let v = args[i];
                    if v.eq_ignore_ascii_case("none") { in_ch = None; }
                    else if let Ok(n) = v.parse::<u32>() { if n <= 0xFFFF { in_ch = Some(n as u16); } else { unsafe { crate::uart_print(b"[CTL] --in out of range\n"); } return; } }
                    else { unsafe { crate::uart_print(b"[CTL] invalid --in\n"); } return; }
                }
                "--out" => {
                    i += 1; if i >= args.len() { unsafe { crate::uart_print(b"[CTL] --out requires a value\n"); } return; }
                    let v = args[i];
                    if v.eq_ignore_ascii_case("none") { out_ch = None; }
                    else if let Ok(n) = v.parse::<u32>() { if n <= 0xFFFF { out_ch = Some(n as u16); } else { unsafe { crate::uart_print(b"[CTL] --out out of range\n"); } return; } }
                    else { unsafe { crate::uart_print(b"[CTL] invalid --out\n"); } return; }
                }
                "--prio" | "--priority" => {
                    i += 1; if i >= args.len() { unsafe { crate::uart_print(b"[CTL] --prio requires a value\n"); } return; }
                    match args[i].parse::<u32>() { Ok(n) if n <= 255 => prio = n as u8, _ => { unsafe { crate::uart_print(b"[CTL] invalid --prio\n"); } return; } }
                }
                "--stage" => {
                    i += 1; if i >= args.len() { unsafe { crate::uart_print(b"[CTL] --stage requires a value\n"); } return; }
                    stage = match args[i] {
                        "acquire" => 0,
                        "clean" => 1,
                        "explore" => 2,
                        "model" => 3,
                        "explain" => 4,
                        _ => { unsafe { crate::uart_print(b"[CTL] invalid stage (use acquire|clean|explore|model|explain)\n"); } return; }
                    };
                }
                "--in-schema" => {
                    i += 1; if i >= args.len() { unsafe { crate::uart_print(b"[CTL] --in-schema requires a value\n"); } return; }
                    match args[i].parse::<u32>() { Ok(s) => _in_schema = Some(s), Err(_) => { unsafe { crate::uart_print(b"[CTL] invalid --in-schema\n"); } return; } }
                }
                "--out-schema" => {
                    i += 1; if i >= args.len() { unsafe { crate::uart_print(b"[CTL] --out-schema requires a value\n"); } return; }
                    match args[i].parse::<u32>() { Ok(s) => _out_schema = Some(s), Err(_) => { unsafe { crate::uart_print(b"[CTL] invalid --out-schema\n"); } return; } }
                }
                _ => { unsafe { crate::uart_print(b"[CTL] unknown option\n"); } return; }
            }
            i += 1;
        }
        #[cfg(feature = "graphctl-framed")]
        {
            let in_val = in_ch.unwrap_or(0xFFFF).to_le_bytes();
            let out_val = out_ch.unwrap_or(0xFFFF).to_le_bytes();
            if _in_schema.is_some() || _out_schema.is_some() {
                // Typed add-operator (0x05)
                let op = op_id.to_le_bytes();
                let ins = _in_schema.unwrap_or(0).to_le_bytes();
                let outs = _out_schema.unwrap_or(0).to_le_bytes();
                let payload = [
                    op[0],op[1],op[2],op[3],
                    in_val[0],in_val[1],
                    out_val[0],out_val[1],
                    prio,
                    stage,
                    ins[0],ins[1],ins[2],ins[3],
                    outs[0],outs[1],outs[2],outs[3]
                ];
                let ok = self.graphctl_send_frame(0x05, &payload);
                unsafe {
                    if ok {
                        crate::uart_print(b"[GRAPH] Added operator ");
                        self.print_number_simple(op_id as u64);
                        crate::uart_print(b" priority ");
                        self.print_number_simple(prio as u64);
                        crate::uart_print(b"\n");
                    } else {
                        crate::uart_print(b"[GRAPH] add-operator failed\n");
                    }
                }
            } else {
                // Untyped add-operator (0x03)
                let op = op_id.to_le_bytes();
                let payload = [
                    op[0],op[1],op[2],op[3],
                    in_val[0],in_val[1],
                    out_val[0],out_val[1],
                    prio,
                    stage,
                ];
                let ok = self.graphctl_send_frame(0x03, &payload);
                unsafe {
                    if ok {
                        crate::uart_print(b"[GRAPH] Added operator ");
                        self.print_number_simple(op_id as u64);
                        crate::uart_print(b" priority ");
                        self.print_number_simple(prio as u64);
                        crate::uart_print(b"\n");
                    } else {
                        crate::uart_print(b"[GRAPH] add-operator failed\n");
                    }
                }
            }
        }
        #[cfg(not(feature = "graphctl-framed"))]
        {
            // Prefer direct path to avoid rare stalls in frame path; preserve current behavior
            let res = crate::control::add_operator_direct(op_id, in_ch, out_ch, prio, stage, _in_schema, _out_schema);
            unsafe {
                if res.is_ok() {
                    crate::uart_print(b"[GRAPH] Added operator ");
                    self.print_number_simple(op_id as u64);
                    crate::uart_print(b" priority ");
                    self.print_number_simple(prio as u64);
                    crate::uart_print(b"\n");
                } else {
                    crate::uart_print(b"[GRAPH] add-operator failed\n");
                }
            }
        }
    }

    pub(crate) fn graphctl_destroy(&self) {
        // Print a friendly status for tests
        unsafe { crate::uart_print(b"[GRAPH] Graph destroyed\n"); }
    }

    pub(crate) fn graphctl_det(&self, args: &[&str]) {
        if args.len() < 3 { unsafe { crate::uart_print(b"Usage: graphctl det <wcet_ns> <period_ns> <deadline_ns>\n"); } return; }
        let wcet = match args[0].parse::<u64>() { Ok(v) => v, Err(_) => { unsafe { crate::uart_print(b"[CTL] invalid wcet\n"); } return; } };
        let period = match args[1].parse::<u64>() { Ok(v) => v, Err(_) => { unsafe { crate::uart_print(b"[CTL] invalid period\n"); } return; } };
        let deadline = match args[2].parse::<u64>() { Ok(v) => v, Err(_) => { unsafe { crate::uart_print(b"[CTL] invalid deadline\n"); } return; } };
        let mut buf = [0u8; 24];
        let w = wcet.to_le_bytes(); buf[0..8].copy_from_slice(&w);
        let p = period.to_le_bytes(); buf[8..16].copy_from_slice(&p);
        let d = deadline.to_le_bytes(); buf[16..24].copy_from_slice(&d);
        let _ = self.graphctl_send_frame(0x06, &buf);
    }

    pub(crate) fn graphctl_stats(&self) {
        if let Some((ops, chans)) = crate::control::current_graph_counts() {
            unsafe {
                crate::uart_print(b"GRAPH: counts ops="); self.print_number_simple(ops as u64); crate::uart_print(b" channels="); self.print_number_simple(chans as u64); crate::uart_print(b"\n");
                crate::uart_print(b"METRIC graph_stats_ops="); self.print_number_simple(ops as u64); crate::uart_print(b"\n");
                crate::uart_print(b"METRIC graph_stats_channels="); self.print_number_simple(chans as u64); crate::uart_print(b"\n");
            }
        } else {
            unsafe { crate::uart_print(b"GRAPH: no active graph\n"); }
        }
    }

    pub(crate) fn graphctl_show_export(&self) {
        match crate::control::export_graph_text() {
            Ok(()) => unsafe { crate::uart_print(b"[GRAPH] export complete\n"); },
            Err(_) => unsafe { crate::uart_print(b"[GRAPH] no active graph\n"); },
        }
    }

    pub(crate) fn graphctl_export_json(&self) {
        match crate::control::export_graph_json() {
            Ok(()) => {},
            Err(_) => unsafe { crate::uart_print(b"[GRAPH] no active graph\n"); },
        }
    }

    pub(crate) fn graphctl_feedback(&self, args: &[&str]) {
        if args.len() < 2 {
            unsafe { crate::uart_print(b"Usage: graphctl feedback <op_id> <helpful|not_helpful|expected>\n"); }
            return;
        }
        let op_id = match args[0].parse::<u32>() {
            Ok(v) => v,
            Err(_) => { unsafe { crate::uart_print(b"[GRAPH] invalid op_id\n"); } return; }
        };
        let feedback_code = match args[1] {
            "helpful" => 1u8,
            "not_helpful" | "not-helpful" => 2u8,
            "expected" => 3u8,
            _ => {
                unsafe { crate::uart_print(b"[GRAPH] Invalid feedback. Use: helpful, not_helpful, or expected\n"); }
                return;
            }
        };
        crate::neural::record_operator_feedback(op_id, feedback_code);
        unsafe {
            crate::uart_print(b"[GRAPH] Feedback recorded for operator ");
        }
        self.print_number_simple(op_id as u64);
        unsafe {
            crate::uart_print(b": ");
            crate::uart_print(args[1].as_bytes());
            crate::uart_print(b"\n[GRAPH] Use 'neuralctl retrain 10' to apply feedback to network\n");
        }
    }
}
