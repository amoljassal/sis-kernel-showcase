// Helpers for netctl commands (predict/buffers/flows/add-conn/simulate)

impl super::Shell {
    /// Predict congestion for a connection
    pub(crate) fn netctl_predict(&self, args: &[&str]) {
        if args.is_empty() || args[0] != "congestion" {
            unsafe { crate::uart_print(b"Usage: netctl predict congestion [conn_id]\n"); }
            return;
        }

        let conn_id = if args.len() > 1 {
            args[1].parse::<u32>().unwrap_or(0)
        } else {
            0
        };

        let (probability, confidence, should_throttle) = crate::network_predictor::predict_congestion(conn_id);

        unsafe {
            crate::uart_print(b"[NET_PREDICTOR] Congestion Prediction:\n");
            crate::uart_print(b"  Connection ID: ");
            self.print_number_simple(conn_id as u64);
            crate::uart_print(b"\n  Congestion probability: ");
            self.print_number_simple((probability / 10) as u64);
            crate::uart_print(b".");
            self.print_number_simple((probability % 10) as u64);
            crate::uart_print(b"%\n  Confidence: ");
            self.print_number_simple(confidence as u64);
            crate::uart_print(b"/1000\n  Recommendation: ");
            crate::uart_print(if should_throttle {
                b"THROTTLE (reduce send rate)\n"
            } else {
                b"NORMAL (no throttling needed)\n"
            });
        }

        // Show predictor statistics
        let predictor = crate::network_predictor::FLOW_CONTROL_PREDICTOR.lock();
        unsafe {
            crate::uart_print(b"  Inferences: ");
            self.print_number_simple(predictor.infer_count as u64);
            crate::uart_print(b"\n  Training updates: ");
            self.print_number_simple(predictor.train_count as u64);
            crate::uart_print(b"\n");
        }
    }

    /// Show adaptive buffer sizes
    pub(crate) fn netctl_buffers(&self, _args: &[&str]) {
        let net_state = crate::network_predictor::NETWORK_STATE.lock();

        unsafe {
            crate::uart_print(b"[NET_PREDICTOR] Adaptive Buffer Sizes:\n");
            crate::uart_print(b"  Active connections: ");
            self.print_number_simple(net_state.connections.len() as u64);
            crate::uart_print(b"\n\n");
        }

        if net_state.connections.is_empty() {
            unsafe { crate::uart_print(b"  No active connections\n"); }
            return;
        }

        for conn in net_state.connections.iter().take(10) {
            let buffer_pred = crate::network_predictor::predict_buffer_size(conn);

            unsafe {
                crate::uart_print(b"  Connection ");
                self.print_number_simple(conn.id as u64);
                crate::uart_print(b":\n");
                crate::uart_print(b"    RTT: ");
                self.print_number_simple(conn.rtt as u64);
                crate::uart_print(b" ms\n");
                crate::uart_print(b"    Congestion window: ");
                self.print_number_simple(conn.cwnd as u64);
                crate::uart_print(b"\n");
                crate::uart_print(b"    Loss rate: ");
                self.print_number_simple(conn.loss_rate as u64);
                crate::uart_print(b"%\n");
                crate::uart_print(b"    Optimal buffer: ");
                self.print_number_simple(buffer_pred.optimal_size as u64);
                crate::uart_print(b" bytes (confidence: ");
                self.print_number_simple(buffer_pred.confidence as u64);
                crate::uart_print(b"/1000)\n\n");
            }
        }
    }

    /// Show learned flow priorities
    pub(crate) fn netctl_flows(&self, _args: &[&str]) {
        let net_state = crate::network_predictor::NETWORK_STATE.lock();
        let priority_tracker = crate::network_predictor::PRIORITY_TRACKER.lock();

        unsafe {
            crate::uart_print(b"[NET_PREDICTOR] Flow Priorities:\n");
            crate::uart_print(b"  Total packets sent: ");
            self.print_number_simple(net_state.total_packets_sent as u64);
            crate::uart_print(b"\n  Total packets lost: ");
            self.print_number_simple(net_state.total_packets_lost as u64);
            crate::uart_print(b"\n  Congestion events: ");
            self.print_number_simple(net_state.total_congestion_events as u64);
            crate::uart_print(b"\n\n");
        }

        if net_state.connections.is_empty() {
            unsafe { crate::uart_print(b"  No active connections\n"); }
            return;
        }

        unsafe { crate::uart_print(b"  Connection Priorities:\n"); }
        for conn in net_state.connections.iter().take(10) {
            let priority = priority_tracker.get_priority(conn.id);
            let is_latency_sensitive = priority_tracker.is_latency_sensitive(conn.id);

            unsafe {
                crate::uart_print(b"    Conn ");
                self.print_number_simple(conn.id as u64);
                crate::uart_print(b": priority=");
                if priority < 0 {
                    crate::uart_print(b"-");
                    self.print_number_simple((-priority) as u64);
                } else {
                    crate::uart_print(b"+");
                    self.print_number_simple(priority as u64);
                }
                crate::uart_print(b" | ");
                crate::uart_print(if is_latency_sensitive {
                    b"LATENCY-SENSITIVE"
                } else {
                    b"THROUGHPUT"
                });
                crate::uart_print(b"\n");
            }
        }
    }

    /// Add a simulated connection for testing
    pub(crate) fn netctl_add_conn(&self, args: &[&str]) {
        if args.is_empty() {
            unsafe { crate::uart_print(b"Usage: netctl add-conn <conn_id> [rtt] [cwnd] [loss_rate]\n"); }
            return;
        }

        let conn_id = args[0].parse::<u32>().unwrap_or(0);
        let rtt = if args.len() > 1 {
            args[1].parse::<u16>().unwrap_or(50)
        } else {
            50
        };
        let cwnd = if args.len() > 2 {
            args[2].parse::<u16>().unwrap_or(10)
        } else {
            10
        };
        let loss_rate = if args.len() > 3 {
            args[3].parse::<u8>().unwrap_or(0)
        } else {
            0
        };

        let mut net_state = crate::network_predictor::NETWORK_STATE.lock();
        net_state.add_connection(conn_id);

        if let Some(conn) = net_state.get_connection_mut(conn_id) {
            conn.rtt = rtt;
            conn.cwnd = cwnd;
            conn.loss_rate = loss_rate;

            unsafe {
                crate::uart_print(b"[NET_PREDICTOR] Added connection ");
                self.print_number_simple(conn_id as u64);
                crate::uart_print(b":\n");
                crate::uart_print(b"  RTT: ");
                self.print_number_simple(rtt as u64);
                crate::uart_print(b" ms\n");
                crate::uart_print(b"  Congestion window: ");
                self.print_number_simple(cwnd as u64);
                crate::uart_print(b"\n");
                crate::uart_print(b"  Loss rate: ");
                self.print_number_simple(loss_rate as u64);
                crate::uart_print(b"%\n");
            }
        }
    }

    /// Simulate network activity
    pub(crate) fn netctl_simulate(&self, args: &[&str]) {
        if args.is_empty() {
            unsafe { crate::uart_print(b"Usage: netctl simulate <packets|congestion|priority> ...\n"); }
            return;
        }

        match args[0] {
            "packets" => {
                let conn_id = if args.len() > 1 {
                    args[1].parse::<u32>().unwrap_or(0)
                } else {
                    0
                };
                let count = if args.len() > 2 {
                    args[2].parse::<u32>().unwrap_or(10)
                } else {
                    10
                };

                let mut net_state = crate::network_predictor::NETWORK_STATE.lock();
                for _ in 0..count {
                    net_state.record_packet_sent(conn_id, 1500);
                }

                unsafe {
                    crate::uart_print(b"[NET_PREDICTOR] Simulated ");
                    self.print_number_simple(count as u64);
                    crate::uart_print(b" packets sent on connection ");
                    self.print_number_simple(conn_id as u64);
                    crate::uart_print(b"\n");
                }
            }
            "congestion" => {
                let conn_id = if args.len() > 1 {
                    args[1].parse::<u32>().unwrap_or(0)
                } else {
                    0
                };

                let mut net_state = crate::network_predictor::NETWORK_STATE.lock();
                net_state.record_congestion();
                net_state.record_packet_loss(conn_id);

                unsafe {
                    crate::uart_print(b"[NET_PREDICTOR] Simulated congestion event on connection ");
                    self.print_number_simple(conn_id as u64);
                    crate::uart_print(b"\n");
                }

                // Train predictor
                drop(net_state);
                crate::network_predictor::record_congestion_outcome(conn_id, true);
            }
            "priority" => {
                let conn_id = if args.len() > 1 {
                    args[1].parse::<u32>().unwrap_or(0)
                } else {
                    0
                };
                let rtt_variance = if args.len() > 2 {
                    args[2].parse::<u16>().unwrap_or(5)
                } else {
                    5
                };
                let burst_size = if args.len() > 3 {
                    args[3].parse::<u32>().unwrap_or(512)
                } else {
                    512
                };

                let mut tracker = crate::network_predictor::PRIORITY_TRACKER.lock();
                tracker.update_priority(conn_id, rtt_variance, burst_size);

                unsafe {
                    crate::uart_print(b"[NET_PREDICTOR] Updated priority for connection ");
                    self.print_number_simple(conn_id as u64);
                    crate::uart_print(b"\n");
                }
            }
            _ => unsafe {
                crate::uart_print(b"Usage: netctl simulate <packets|congestion|priority> ...\n");
            }
        }
    }
}
