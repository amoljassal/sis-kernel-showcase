// Week 11: Simple Networking (AI-Enhanced)
//
// Features:
// - Learned flow control (6→8→1 network predicts congestion)
// - Adaptive buffering (dynamic buffer size prediction)
// - Connection priority learning (latency-sensitive detection)

use alloc::vec::Vec;
use spin::Mutex;

// ============================================================================
// Network Connection State (Simulated)
// ============================================================================

#[derive(Copy, Clone, Debug)]
pub struct ConnectionState {
    pub id: u32,
    pub rtt: u16,              // Round-trip time in ms
    pub cwnd: u16,             // Congestion window size
    pub loss_rate: u8,         // Packet loss rate 0-100%
    pub bytes_sent: u32,
    pub bytes_received: u32,
    pub priority: i8,          // Learned priority -100 to 100
    pub is_latency_sensitive: bool,
}

impl ConnectionState {
    pub const fn new(id: u32) -> Self {
        Self {
            id,
            rtt: 50,               // Default 50ms
            cwnd: 10,              // Default window
            loss_rate: 0,
            bytes_sent: 0,
            bytes_received: 0,
            priority: 0,
            is_latency_sensitive: false,
        }
    }
}

pub struct NetworkState {
    pub connections: Vec<ConnectionState>,
    pub total_packets_sent: u32,
    pub total_packets_lost: u32,
    pub total_congestion_events: u32,
}

impl NetworkState {
    pub const fn new() -> Self {
        Self {
            connections: Vec::new(),
            total_packets_sent: 0,
            total_packets_lost: 0,
            total_congestion_events: 0,
        }
    }

    pub fn add_connection(&mut self, id: u32) {
        if self.connections.len() < 32 {
            self.connections.push(ConnectionState::new(id));
        }
    }

    pub fn get_connection(&self, id: u32) -> Option<&ConnectionState> {
        self.connections.iter().find(|c| c.id == id)
    }

    pub fn get_connection_mut(&mut self, id: u32) -> Option<&mut ConnectionState> {
        self.connections.iter_mut().find(|c| c.id == id)
    }

    pub fn record_packet_sent(&mut self, conn_id: u32, size: u32) {
        self.total_packets_sent += 1;
        if let Some(conn) = self.get_connection_mut(conn_id) {
            conn.bytes_sent += size;
        }
    }

    pub fn record_packet_loss(&mut self, conn_id: u32) {
        self.total_packets_lost += 1;
        if let Some(conn) = self.get_connection_mut(conn_id) {
            conn.loss_rate = ((conn.loss_rate as u32 * 9 + 100) / 10).min(100) as u8;
        }
    }

    pub fn record_congestion(&mut self) {
        self.total_congestion_events += 1;
    }
}

pub static NETWORK_STATE: Mutex<NetworkState> = Mutex::new(NetworkState::new());

// ============================================================================
// Flow Control Predictor (6→8→1 Network)
// ============================================================================

pub struct FlowControlPredictor {
    // Layer 1: 6 inputs → 8 hidden
    weights_l1: [[i16; 6]; 8], // Q8.8 format
    biases_l1: [i16; 8],

    // Layer 2: 8 hidden → 1 output
    weights_l2: [i16; 8], // Q8.8 format
    bias_l2: i16,

    // Statistics
    pub infer_count: u32,
    pub train_count: u32,
    pub avg_error: i16, // Q8.8 format
}

impl FlowControlPredictor {
    pub const fn new() -> Self {
        Self {
            // Initialize with small random-ish values
            weights_l1: [
                [18, -12, 10, -8, 15, -10],
                [-10, 15, -8, 12, -10, 8],
                [12, -8, 15, -10, 8, -12],
                [-15, 10, -12, 8, -10, 15],
                [10, -15, 8, -12, 15, -8],
                [-8, 12, -10, 15, -8, 10],
                [15, -10, 12, -8, 10, -15],
                [-12, 8, -15, 10, -12, 8],
            ],
            biases_l1: [10, -8, 12, -10, 8, -12, 10, -8],
            weights_l2: [15, -12, 10, -8, 12, -10, 8, -12],
            bias_l2: 50,
            infer_count: 0,
            train_count: 0,
            avg_error: 0,
        }
    }

    /// ReLU activation
    fn relu(x: i16) -> i16 {
        x.max(0)
    }

    /// Sigmoid approximation for output (0-1000 range)
    fn sigmoid_approx(x: i16) -> u16 {
        // Approximation: sigmoid(x) ≈ 500 + x/4 for x in [-2000, 2000]
        let clamped = x.clamp(-2000, 2000);
        ((500 + clamped / 4) as u16).clamp(0, 1000)
    }

    /// Inference: predict congestion probability
    /// Returns (congestion_probability_0_1000, confidence)
    pub fn infer(&mut self, features: &[i16; 6]) -> (u16, u16) {
        self.infer_count += 1;

        // Layer 1: 6 → 8 with ReLU
        let mut hidden = [0i16; 8];
        for i in 0..8 {
            let mut sum = self.biases_l1[i] as i32;
            for j in 0..6 {
                sum += ((self.weights_l1[i][j] as i32) * (features[j] as i32)) >> 8;
            }
            hidden[i] = Self::relu(sum.clamp(-32768, 32767) as i16);
        }

        // Layer 2: 8 → 1 (sigmoid for probability)
        let mut output = self.bias_l2 as i32;
        for i in 0..8 {
            output += ((self.weights_l2[i] as i32) * (hidden[i] as i32)) >> 8;
        }
        let output = output.clamp(-32768, 32767) as i16;

        // Convert to probability (0-1000)
        let probability = Self::sigmoid_approx(output);

        // Confidence based on how decisive the output is (closer to 0 or 1000 = higher confidence)
        let distance_from_middle = ((probability as i16) - 500).abs();
        let confidence = (500 + distance_from_middle).min(1000) as u16;

        (probability, confidence)
    }

    /// Train on actual congestion outcome
    pub fn train(&mut self, features: &[i16; 6], did_congest: bool, learning_rate: i16) {
        self.train_count += 1;

        // Forward pass
        let mut hidden = [0i16; 8];
        let mut hidden_raw = [0i32; 8];
        for i in 0..8 {
            let mut sum = self.biases_l1[i] as i32;
            for j in 0..6 {
                sum += ((self.weights_l1[i][j] as i32) * (features[j] as i32)) >> 8;
            }
            hidden_raw[i] = sum;
            hidden[i] = Self::relu(sum.clamp(-32768, 32767) as i16);
        }

        let mut output = self.bias_l2 as i32;
        for i in 0..8 {
            output += ((self.weights_l2[i] as i32) * (hidden[i] as i32)) >> 8;
        }
        let predicted = Self::sigmoid_approx(output.clamp(-32768, 32767) as i16);

        // Compute error
        let target = if did_congest { 1000 } else { 0 };
        let error = (predicted as i32) - target;

        // Update average error
        self.avg_error = ((self.avg_error as i32 * 9 + error.abs()) / 10).min(32767) as i16;

        // Backpropagation (simplified)
        let output_grad = error;

        // Update output weights and bias
        for i in 0..8 {
            let grad = ((output_grad * (hidden[i] as i32)) >> 8) * (learning_rate as i32) / 256;
            self.weights_l2[i] = (self.weights_l2[i] as i32 - grad).clamp(-32768, 32767) as i16;
        }
        let bias_grad = (output_grad * (learning_rate as i32)) / 256;
        self.bias_l2 = (self.bias_l2 as i32 - bias_grad).clamp(-32768, 32767) as i16;

        // Hidden layer gradient
        let mut hidden_grad = [0i32; 8];
        for i in 0..8 {
            let relu_deriv = if hidden_raw[i] > 0 { 1 } else { 0 };
            hidden_grad[i] = ((output_grad * (self.weights_l2[i] as i32)) >> 8) * relu_deriv;
        }

        // Update hidden weights and biases
        for i in 0..8 {
            for j in 0..6 {
                let grad = ((hidden_grad[i] * (features[j] as i32)) >> 8) * (learning_rate as i32) / 256;
                self.weights_l1[i][j] = (self.weights_l1[i][j] as i32 - grad).clamp(-32768, 32767) as i16;
            }
            let bias_grad = (hidden_grad[i] * (learning_rate as i32)) / 256;
            self.biases_l1[i] = (self.biases_l1[i] as i32 - bias_grad).clamp(-32768, 32767) as i16;
        }
    }
}

pub static FLOW_CONTROL_PREDICTOR: Mutex<FlowControlPredictor> =
    Mutex::new(FlowControlPredictor::new());

// ============================================================================
// Adaptive Buffering
// ============================================================================

#[derive(Copy, Clone)]
pub struct BufferPrediction {
    pub optimal_size: usize,
    pub confidence: u16,
}

impl BufferPrediction {
    pub const fn new() -> Self {
        Self {
            optimal_size: 8192,
            confidence: 500,
        }
    }
}

/// Predict optimal buffer size based on connection state
pub fn predict_buffer_size(conn: &ConnectionState) -> BufferPrediction {
    // Simple heuristic predictor (can be replaced with neural network)
    // Features: rtt, cwnd, loss_rate, memory_pressure

    let heap_stats = crate::heap::get_heap_stats();
    let heap_size = crate::heap::heap_total_size(); // Single source of truth from heap.rs
    let used = heap_stats.current_allocated();
    let free = heap_size.saturating_sub(used);
    let memory_pressure = (100 - (free * 100 / heap_size)).min(100);

    // Base size on RTT and congestion window
    let base_size = (conn.rtt as usize * conn.cwnd as usize).clamp(4096, 65536);

    // Reduce if memory pressure high
    let adjusted_size = if memory_pressure > 80 {
        base_size / 2
    } else if memory_pressure > 60 {
        (base_size * 3) / 4
    } else {
        base_size
    };

    // Reduce if high packet loss
    let final_size = if conn.loss_rate > 20 {
        adjusted_size / 2
    } else if conn.loss_rate > 10 {
        (adjusted_size * 3) / 4
    } else {
        adjusted_size
    };

    let confidence = if memory_pressure < 50 && conn.loss_rate < 5 {
        800
    } else if memory_pressure < 70 && conn.loss_rate < 15 {
        600
    } else {
        400
    };

    BufferPrediction {
        optimal_size: final_size,
        confidence,
    }
}

// ============================================================================
// Connection Priority Learning
// ============================================================================

pub struct ConnectionPriorityTracker {
    priorities: [i8; 32],           // Per-connection priority
    latency_sensitive: [bool; 32], // Is connection latency-sensitive?
    connection_count: usize,
}

impl ConnectionPriorityTracker {
    pub const fn new() -> Self {
        Self {
            priorities: [0; 32],
            latency_sensitive: [false; 32],
            connection_count: 0,
        }
    }

    /// Learn if connection is latency-sensitive based on patterns
    pub fn update_priority(&mut self, conn_id: u32, rtt_variance: u16, burst_size: u32) {
        if (conn_id as usize) < 32 {
            // Low RTT variance + small bursts = latency-sensitive
            let is_latency_sensitive = rtt_variance < 10 && burst_size < 1024;
            self.latency_sensitive[conn_id as usize] = is_latency_sensitive;

            // Increase priority if latency-sensitive
            if is_latency_sensitive {
                self.priorities[conn_id as usize] = (self.priorities[conn_id as usize] + 10).min(100);
            } else {
                // Gradually normalize priority
                let current = self.priorities[conn_id as usize];
                self.priorities[conn_id as usize] = if current > 0 {
                    current - 1
                } else if current < 0 {
                    current + 1
                } else {
                    0
                };
            }
        }
    }

    pub fn get_priority(&self, conn_id: u32) -> i8 {
        if (conn_id as usize) < 32 {
            self.priorities[conn_id as usize]
        } else {
            0
        }
    }

    pub fn is_latency_sensitive(&self, conn_id: u32) -> bool {
        if (conn_id as usize) < 32 {
            self.latency_sensitive[conn_id as usize]
        } else {
            false
        }
    }

    pub fn get_connection_count(&self) -> usize {
        self.connection_count
    }
}

pub static PRIORITY_TRACKER: Mutex<ConnectionPriorityTracker> =
    Mutex::new(ConnectionPriorityTracker::new());

// ============================================================================
// Feature Extraction for Flow Control
// ============================================================================

/// Extract 6 features for flow control prediction
pub fn extract_flow_features(conn: &ConnectionState) -> [i16; 6] {
    let mut features = [0i16; 6];

    // Feature 0: RTT (round-trip time in ms, normalized)
    features[0] = (conn.rtt as i16).min(1000);

    // Feature 1: Congestion window size (normalized)
    features[1] = (conn.cwnd as i16 * 10).min(32767);

    // Feature 2: Packet loss rate (0-100 → 0-25600 in Q8.8)
    features[2] = (conn.loss_rate as i16) << 8;

    // Feature 3: Current memory pressure
    let heap_stats = crate::heap::get_heap_stats();
    let heap_size = crate::heap::heap_total_size(); // Single source of truth from heap.rs
    let used = heap_stats.current_allocated();
    let free = heap_size.saturating_sub(used);
    let pressure = (100 - (free * 100 / heap_size)).min(100);
    features[3] = (pressure as i16) << 8;

    // Feature 4: Send rate (bytes sent per second estimate)
    let send_rate = (conn.bytes_sent / 1000).min(32767) as i16;
    features[4] = send_rate;

    // Feature 5: Current system load (from agent bus)
    let messages = crate::internal_agent_bus::get_all_messages();
    let load = messages.len().min(255) as i16;
    features[5] = load * 100;

    features
}

// ============================================================================
// Public API
// ============================================================================

/// Predict congestion and decide if should throttle
pub fn predict_congestion(conn_id: u32) -> (u16, u16, bool) {
    let net_state = NETWORK_STATE.lock();

    if let Some(conn) = net_state.get_connection(conn_id) {
        let features = extract_flow_features(conn);
        let (probability, confidence) = FLOW_CONTROL_PREDICTOR.lock().infer(&features);

        // Throttle if probability > 600 (60%) and confidence > 600
        let should_throttle = probability > 600 && confidence > 600;

        (probability, confidence, should_throttle)
    } else {
        (0, 0, false)
    }
}

/// Update connection state with actual outcome
pub fn record_congestion_outcome(conn_id: u32, did_congest: bool) {
    let net_state = NETWORK_STATE.lock();

    if let Some(conn) = net_state.get_connection(conn_id) {
        let features = extract_flow_features(conn);
        FLOW_CONTROL_PREDICTOR.lock().train(&features, did_congest, 51); // ~0.2 learning rate
    }
}

/// Get adaptive buffer size for connection
pub fn get_adaptive_buffer_size(conn_id: u32) -> BufferPrediction {
    let net_state = NETWORK_STATE.lock();

    if let Some(conn) = net_state.get_connection(conn_id) {
        predict_buffer_size(conn)
    } else {
        BufferPrediction::new()
    }
}
