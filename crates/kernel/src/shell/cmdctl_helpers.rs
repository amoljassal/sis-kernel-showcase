// Helpers for cmdctl commands (predict/batch/learn)

impl super::Shell {
    /// Preview predicted execution time for a command
    pub(crate) fn cmdctl_predict(&self, args: &[&str]) {
        if args.is_empty() {
            unsafe { crate::uart_print(b"Usage: cmdctl predict <command> [args...]\n"); }
            return;
        }

        let cmd = args[0];
        let cmd_args = if args.len() > 1 { &args[1..] } else { &[] };

        let (predicted_us, confidence) = crate::command_predictor::predict_command_execution(cmd, cmd_args);

        unsafe { crate::uart_print(b"[CMD_PREDICTOR] Execution Time Prediction:\n"); }
        unsafe { crate::uart_print(b"  Command: "); }
        unsafe { crate::uart_print(cmd.as_bytes()); }
        unsafe { crate::uart_print(b"\n  Predicted time: "); }
        self.print_number_simple((predicted_us / 1000) as u64);
        unsafe { crate::uart_print(b" ms\n  Confidence: "); }
        self.print_number_simple(confidence as u64);
        unsafe { crate::uart_print(b"/1000\n"); }
    }

    /// Show command batching status
    pub(crate) fn cmdctl_batch(&self, args: &[&str]) {
        if args.is_empty() || args[0] == "status" {
            let batcher = crate::command_predictor::COMMAND_BATCHER.lock();
            unsafe { crate::uart_print(b"[CMD_BATCHER] Batching Status:\n"); }
            unsafe { crate::uart_print(b"  Learned optimal batch size: "); }
            self.print_number_simple(batcher.learned_optimal_size as u64);
            unsafe { crate::uart_print(b"\n  Max batch size: "); }
            self.print_number_simple(batcher.max_batch_size as u64);
            unsafe { crate::uart_print(b"\n  Batches executed: "); }
            self.print_number_simple(batcher.batches_executed as u64);
            unsafe { crate::uart_print(b"\n  Avg throughput gain: "); }
            let gain = batcher.total_throughput_gain / (batcher.batches_executed.max(1) as i32);
            self.print_number_simple((gain / 256) as u64);
            unsafe { crate::uart_print(b"."); }
            let frac = ((gain % 256) * 100) / 256;
            self.print_number_simple(frac.abs() as u64);
            unsafe { crate::uart_print(b"x\n"); }
        } else {
            unsafe { crate::uart_print(b"Usage: cmdctl batch [status]\n"); }
        }
    }

    /// Show command prediction learning statistics
    pub(crate) fn cmdctl_learn(&self, args: &[&str]) {
        if args.is_empty() || args[0] == "stats" {
            let accuracy = crate::command_predictor::get_prediction_accuracy();
            let predictor = crate::command_predictor::COMMAND_PREDICTOR.lock();
            let ledger = crate::command_predictor::PREDICTION_LEDGER.lock();

            unsafe { crate::uart_print(b"[CMD_PREDICTOR] Learning Statistics:\n"); }
            unsafe { crate::uart_print(b"  Total predictions: "); }
            self.print_number_simple(ledger.total_predictions as u64);
            unsafe { crate::uart_print(b"\n  Prediction accuracy (last 100): "); }
            self.print_number_simple(accuracy as u64);
            unsafe { crate::uart_print(b"%\n  Inference count: "); }
            self.print_number_simple(predictor.infer_count as u64);
            unsafe { crate::uart_print(b"\n  Training updates: "); }
            self.print_number_simple(predictor.train_count as u64);
            unsafe { crate::uart_print(b"\n  Avg prediction error: "); }
            self.print_number_simple((predictor.avg_error / 256) as u64);
            unsafe { crate::uart_print(b"."); }
            let frac = ((predictor.avg_error % 256) * 100) / 256;
            self.print_number_simple(frac.abs() as u64);
            unsafe { crate::uart_print(b" (Q8.8)\n"); }
        } else {
            unsafe { crate::uart_print(b"Usage: cmdctl learn [stats]\n"); }
        }
    }
}
