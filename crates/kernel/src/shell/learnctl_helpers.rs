// Helpers for learnctl (prediction tracker / OOD / feedback)

impl super::Shell {
    pub(crate) fn learnctl_cmd(&self, args: &[&str]) {
        if args.is_empty() { unsafe { crate::uart_print(b"Usage: learnctl <stats|dump|train|feedback good|bad|verybad ID>\n"); } return; }
        match args[0] {
            "stats" => {
                let ledger = crate::prediction_tracker::get_ledger();
                unsafe {
                    crate::uart_print(b"\n=== Prediction Statistics ===\n");
                    crate::uart_print(b"  Total Predictions: "); self.print_number_simple(ledger.len() as u64); crate::uart_print(b"/1000\n\n");
                }
                if ledger.len() == 0 { unsafe { crate::uart_print(b"  No predictions recorded yet.\n\n"); } drop(ledger); return; }
                let (c100, t100) = ledger.compute_accuracy(100);
                let (c500, t500) = ledger.compute_accuracy(500);
                let (call, tall) = ledger.compute_accuracy(1000);
                unsafe {
                    crate::uart_print(b"Overall Accuracy:\n  Last 100: ");
                    if t100 > 0 { self.print_number_simple(c100 as u64); crate::uart_print(b"/"); self.print_number_simple(t100 as u64); let pct=(c100*100)/t100; crate::uart_print(b" ("); self.print_number_simple(pct as u64); crate::uart_print(b"%)\n"); } else { crate::uart_print(b"N/A (no outcomes yet)\n"); }
                    crate::uart_print(b"  Last 500: ");
                    if t500 > 0 { self.print_number_simple(c500 as u64); crate::uart_print(b"/"); self.print_number_simple(t500 as u64); let pct=(c500*100)/t500; crate::uart_print(b" ("); self.print_number_simple(pct as u64); crate::uart_print(b"%)\n"); } else { crate::uart_print(b"N/A\n"); }
                    crate::uart_print(b"  All time: ");
                    if tall > 0 { self.print_number_simple(call as u64); crate::uart_print(b"/"); self.print_number_simple(tall as u64); let pct=(call*100)/tall; crate::uart_print(b" ("); self.print_number_simple(pct as u64); crate::uart_print(b"%)\n\n"); } else { crate::uart_print(b"N/A\n\n"); }
                }
                use crate::prediction_tracker::PredictionType;
                let types = [
                    (PredictionType::MemoryPressure, b"Memory Pressure"),
                    (PredictionType::MemoryCompactionNeeded, b"Memory Compact "),
                    (PredictionType::SchedulingDeadlineMiss, b"Deadline Miss  "),
                    (PredictionType::CommandHeavy, b"Command Heavy  "),
                    (PredictionType::CommandRapidStream, b"Rapid Stream   "),
                ];
                unsafe {
                    crate::uart_print(b"Accuracy by Type:\n  Type             | Count | Accuracy\n  -----------------|-------|----------\n");
                }
                for (pred_type, name) in &types {
                    let (correct, total_with_outcomes, total_all) = ledger.compute_accuracy_by_type(*pred_type, 1000);
                    unsafe {
                        crate::uart_print(b"  "); crate::uart_print(*name); crate::uart_print(b" | "); self.print_number_simple(total_all as u64);
                        crate::uart_print(b" | "); if total_with_outcomes > 0 { let pct = (correct * 100) / total_with_outcomes; self.print_number_simple(pct as u64); crate::uart_print(b"%\n"); } else { crate::uart_print(b"N/A\n"); }
                    }
                }
                unsafe { crate::uart_print(b"\n"); }
                drop(ledger);
            }
            "dump" => {
                let (ood_count, stats) = crate::prediction_tracker::get_ood_stats();
                unsafe {
                    crate::uart_print(b"[LEARNCTL] OOD Detector Statistics\n");
                    crate::uart_print(b"  Total OOD Detections: "); self.print_number_simple(ood_count); crate::uart_print(b"\n");
                }
                if stats.valid {
                    unsafe {
                        crate::uart_print(b"Training Distribution ("); self.print_number_simple(stats.sample_count as u64); crate::uart_print(b" samples):\n");
                        crate::uart_print(b"  Feature      | Mean  | StdDev | Min   | Max\n");
                        crate::uart_print(b"  -------------|-------|--------|-------|-------\n");
                    }
                    let feature_names: &[&[u8]] = &[
                        b"MemPressure ", b"MemFragment ", b"MemAllocRate", b"MemFailures ",
                        b"SchedLoad   ", b"Deadlines   ", b"OpLatency   ", b"CriticalOps ",
                        b"CmdRate     ", b"CmdHeavy    ", b"PredictAcc  ", b"RapidStream ",
                    ];
                    for i in 0..12 {
                        unsafe {
                            crate::uart_print(b"  "); crate::uart_print(feature_names[i]); crate::uart_print(b"| ");
                            self.print_number_simple(stats.means[i] as u64); crate::uart_print(b" | ");
                            self.print_number_simple(stats.stddevs[i] as u64); crate::uart_print(b"   | ");
                            self.print_number_simple(stats.mins[i] as u64); crate::uart_print(b" | ");
                            self.print_number_simple(stats.maxs[i] as u64); crate::uart_print(b"\n");
                        }
                    }
                    unsafe { crate::uart_print(b"\n"); }
                } else {
                    unsafe {
                        crate::uart_print(b"Training distribution: Not yet initialized\n");
                        crate::uart_print(b"Run 'learnctl train' multiple times to build history\n\n");
                    }
                }
            }
            "train" => {
                let state = crate::meta_agent::collect_telemetry();
                let mut features = [0i16; 12];
                features[0] = state.memory_pressure as i16; features[1] = state.memory_fragmentation as i16; features[2] = state.memory_alloc_rate as i16; features[3] = state.memory_failures as i16;
                features[4] = state.scheduling_load as i16; features[5] = state.deadline_misses as i16; features[6] = state.operator_latency_ms as i16; features[7] = state.critical_ops_count as i16;
                features[8] = state.command_rate as i16; features[9] = state.command_heaviness as i16; features[10] = state.prediction_accuracy as i16; features[11] = state.rapid_stream_detected as i16;
                crate::prediction_tracker::train_ood_detector(&features);
                let (_, ood_stats) = crate::prediction_tracker::get_ood_stats(); if ood_stats.valid { crate::prediction_tracker::record_distribution_snapshot(ood_stats); }
                let (new_lr, adjusted) = crate::prediction_tracker::adapt_learning_rate();
                unsafe {
                    crate::uart_print(b"[LEARNCTL] OOD detector trained with current state\n");
                    if adjusted { crate::uart_print(b"[LEARNCTL] Learning rate adapted: "); let lr_pct=(new_lr as u64*100)/256; self.print_number_simple(lr_pct); crate::uart_print(b"/100\n"); }
                    crate::uart_print(b"Run 'autoctl oodcheck' to see updated distribution\nRun 'autoctl driftcheck' to check for distribution shift\n");
                }
            }
            "feedback" => {
                if args.len() < 3 { unsafe { crate::uart_print(b"Usage: learnctl feedback <good|bad|verybad> <decision_id>\n"); } return; }
                let decision_id = args[2].parse::<u64>().unwrap_or(0); if decision_id == 0 { unsafe { crate::uart_print(b"[ERROR] Invalid decision ID\n"); } return; }
                let reward_override = match args[1] { "good"=>100i16, "bad"=>-50i16, "verybad"=>-200i16, _=>{ unsafe { crate::uart_print(b"[ERROR] Feedback must be good, bad, or verybad\n"); } return; } };
                let success = crate::autonomy::apply_human_feedback(decision_id, reward_override);
                if success {
                    unsafe {
                        crate::uart_print(b"[LEARNCTL] Human feedback applied to decision #"); self.print_number_simple(decision_id); crate::uart_print(b": ");
                        crate::uart_print(match reward_override { 100=>b"GOOD (+100 reward)\n", -50=>b"BAD (-50 reward)\n", -200=>b"VERY BAD (-200 reward)\n", _=>b"UNKNOWN\n" });
                        crate::uart_print(b"Reward overridden in decision record.\n");
                        if reward_override == -200 { crate::uart_print(b"[WARNING] VERY BAD feedback recorded. Decision marked for analysis.\n"); }
                    }
                } else {
                    unsafe {
                        crate::uart_print(b"[ERROR] Decision ID #"); self.print_number_simple(decision_id); crate::uart_print(b" not found in audit log\n");
                        crate::uart_print(b"Use 'autoctl dashboard' to see recent decisions\n");
                    }
                }
            }
            _ => unsafe { crate::uart_print(b"Usage: learnctl <stats|dump|train|feedback good|bad|verybad ID>\n"); }
        }
    }
}
