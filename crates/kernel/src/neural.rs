//! Minimal fixed-point MLP core for a kernel-resident "neural agent".
//! - Single hidden layer MLP with fixed maximum sizes (bounded compute)
//! - Q8.8 fixed-point arithmetic (i16 weights/activations), accumulators in i32
//! - Deterministic, no heap allocations, safe-by-default caps

use spin::Mutex;
use crate::trace::metric_kv;

pub const MAX_IN: usize = 16;
pub const MAX_H: usize = 16;
pub const MAX_OUT: usize = 4;

#[inline(always)]
const fn q88(v: i16) -> i16 { v }

#[inline(always)]
fn q88_relu(x: i16) -> i16 { if x < 0 { 0 } else { x } }

#[inline(always)]
fn q88_mul(a: i16, b: i16) -> i32 { (a as i32) * (b as i32) }

#[inline(always)]
fn q88_acc_scale(acc: i32) -> i16 {
    // Scale back from Q16.16 to Q8.8 by >> 8, with clamp
    let v = (acc >> 8).clamp(i16::MIN as i32, i16::MAX as i32);
    v as i16
}

pub struct NeuralAgent {
    pub in_sz: usize,
    pub hid_sz: usize,
    pub out_sz: usize,
    pub w1: [[i16; MAX_IN]; MAX_H],
    pub b1: [i16; MAX_H],
    pub w2: [[i16; MAX_H]; MAX_OUT],
    pub b2: [i16; MAX_OUT],
    pub last_in: [i16; MAX_IN],
    pub last_in_len: usize,
    pub last_out: [i16; MAX_OUT],
    pub last_out_len: usize,
    pub infer_count: usize,
    pub teach_count: usize,
}

impl NeuralAgent {
    pub const fn new() -> Self {
        Self {
            in_sz: 3, hid_sz: 3, out_sz: 2,
            w1: [[0; MAX_IN]; MAX_H],
            b1: [0; MAX_H],
            w2: [[0; MAX_H]; MAX_OUT],
            b2: [0; MAX_OUT],
            last_in: [0; MAX_IN], last_in_len: 0,
            last_out: [0; MAX_OUT], last_out_len: 0,
            infer_count: 0,
            teach_count: 0,
        }
    }

    pub fn reset_defaults(&mut self) {
        // Default small identity-like mapping with mild gain
        self.in_sz = 3; self.hid_sz = 3; self.out_sz = 2;
        for r in 0..self.hid_sz { for c in 0..self.in_sz { self.w1[r][c] = if r==c { q88(256) } else { 0 }; } }
        for r in 0..self.out_sz { for c in 0..self.hid_sz { self.w2[r][c] = if r==c { q88(256) } else { 0 }; } }
        for i in 0..self.hid_sz { self.b1[i] = 0; }
        for i in 0..self.out_sz { self.b2[i] = 0; }
        self.last_in_len = 0; self.last_out_len = 0;
    }
    pub fn set_dims(&mut self, in_sz: usize, hid_sz: usize, out_sz: usize) -> bool {
        if in_sz == 0 || hid_sz == 0 || out_sz == 0 { return false; }
        if in_sz > MAX_IN || hid_sz > MAX_H || out_sz > MAX_OUT { return false; }
        self.in_sz = in_sz; self.hid_sz = hid_sz; self.out_sz = out_sz;
        for r in 0..self.hid_sz { for c in 0..self.in_sz { self.w1[r][c] = if r==c { q88(256) } else { 0 }; } }
        for r in 0..self.out_sz { for c in 0..self.hid_sz { self.w2[r][c] = if r==c { q88(256) } else { 0 }; } }
        for i in 0..self.hid_sz { self.b1[i] = 0; }
        for i in 0..self.out_sz { self.b2[i] = 0; }
        self.last_in_len = 0; self.last_out_len = 0;
        true
    }

    #[inline(always)]
    fn dims_ok(&self) -> bool {
        self.in_sz <= MAX_IN && self.hid_sz <= MAX_H && self.out_sz <= MAX_OUT && self.in_sz > 0 && self.hid_sz > 0 && self.out_sz > 0
    }

    /// Run a bounded MLP inference with Q8.8 inputs; updates last_out and returns out_len.
    /// Lazy initialization: on first use (infer_count == 0), auto-initializes to identity-like defaults.
    pub fn infer(&mut self, input_q88: &[i16]) -> usize {
        // Lazy init: if this is the first inference, initialize to sane defaults
        if self.infer_count == 0 {
            self.reset_defaults();
        }
        let t0 = crate::graph::now_cycles();
        let mut hid = [0i16; MAX_H];
        let mut out = [0i16; MAX_OUT];
        let in_len = input_q88.len().min(self.in_sz);
        for i in 0..in_len { self.last_in[i] = input_q88[i]; }
        self.last_in_len = in_len;
        if !self.dims_ok() { return 0; }
        // hidden = relu(W1 * x + b1)
        for r in 0..self.hid_sz {
            let mut acc: i32 = (self.b1[r] as i32) << 8;
            for c in 0..self.in_sz { acc = acc.saturating_add(q88_mul(self.w1[r][c], self.last_in[c])); }
            hid[r] = q88_relu(q88_acc_scale(acc));
        }
        // out = W2 * hidden + b2
        for r in 0..self.out_sz {
            let mut acc: i32 = (self.b2[r] as i32) << 8;
            for c in 0..self.hid_sz { acc = acc.saturating_add(q88_mul(self.w2[r][c], hid[c])); }
            out[r] = q88_acc_scale(acc);
        }
        for i in 0..self.out_sz { self.last_out[i] = out[i]; }
        self.last_out_len = self.out_sz;
        self.infer_count = self.infer_count.saturating_add(1);
        let t1 = crate::graph::now_cycles();
        let us = (crate::graph::cycles_to_ns(t1.saturating_sub(t0)) / 1000) as usize;
        metric_kv("nn_infer_us", us);
        metric_kv("nn_infer_count", self.infer_count);
        // Audit infer
        let mut entry = NN_AUDIT_ZERO;
        entry.op = 1;
        entry.in_len = self.last_in_len as u8;
        entry.out_len = self.last_out_len as u8;
        entry.ts_ns = crate::graph::cycles_to_ns(t1);
        entry.latency_us = us as u32;
        for i in 0..self.last_in_len { entry.inputs_q88[i] = self.last_in[i]; }
        for i in 0..self.last_out_len { entry.outputs_q88[i] = self.last_out[i]; }
        NN_AUDIT.lock().push(entry);
        self.last_out_len
    }

    /// Update all weights/biases from a flat Q8.8 vector in this order: w1 (hid*in), b1 (hid), w2 (out*hid), b2 (out)
    pub fn update_weights(&mut self, vals: &[i16]) -> bool {
        if !self.dims_ok() { return false; }
        let need = self.hid_sz * self.in_sz + self.hid_sz + self.out_sz * self.hid_sz + self.out_sz;
        if vals.len() < need { return false; }
        let mut idx = 0;
        for r in 0..self.hid_sz { for c in 0..self.in_sz { self.w1[r][c] = vals[idx]; idx+=1; } }
        for r in 0..self.hid_sz { self.b1[r] = vals[idx]; idx+=1; }
        for r in 0..self.out_sz { for c in 0..self.hid_sz { self.w2[r][c] = vals[idx]; idx+=1; } }
        for r in 0..self.out_sz { self.b2[r] = vals[idx]; idx+=1; }
        true
    }

    /// Print a concise status to UART
    pub fn print_status(&self) {
        unsafe { crate::uart_print(b"[NN] dims in="); }
        crate::shell::print_number_simple(self.in_sz as u64);
        unsafe { crate::uart_print(b" hid="); }
        crate::shell::print_number_simple(self.hid_sz as u64);
        unsafe { crate::uart_print(b" out="); }
        crate::shell::print_number_simple(self.out_sz as u64);
        unsafe { crate::uart_print(b" infer_count="); }
        crate::shell::print_number_simple(self.infer_count as u64);
        unsafe { crate::uart_print(b"\n"); }
        // Print last I/O as milli integers
        unsafe { crate::uart_print(b"[NN] last_in_milli="); }
        for i in 0..self.last_in_len {
            if i>0 { unsafe { crate::uart_print(b","); } }
            let v = ((self.last_in[i] as i32) * 1000 / 256).clamp(-1_000_000, 1_000_000);
            print_i32(v);
        }
        unsafe { crate::uart_print(b"\n"); }
        unsafe { crate::uart_print(b"[NN] last_out_milli="); }
        for i in 0..self.last_out_len {
            if i>0 { unsafe { crate::uart_print(b","); } }
            let v = ((self.last_out[i] as i32) * 1000 / 256).clamp(-1_000_000, 1_000_000);
            print_i32(v);
        }
        unsafe { crate::uart_print(b"\n"); }
    }
}

static NEURAL: Mutex<NeuralAgent> = Mutex::new(NeuralAgent::new());

// --- Audit ring for observability ---
#[derive(Copy, Clone)]
struct NnAuditEntry {
    op: u8,            // 1=infer, 2=teach, 3=command_predict, 4=operator_predict
    in_len: u8,
    out_len: u8,
    ts_ns: u64,
    latency_us: u32,
    inputs_q88: [i16; MAX_IN],
    targets_q88: [i16; MAX_OUT], // only set for teach
    outputs_q88: [i16; MAX_OUT], // only set for infer
    // Command prediction tracking (op=3)
    command_hash: u32,  // Simple hash of command name
    outcome: u8,        // 0=pending, 1=success, 2=fail, 3=error
    feedback: u8,       // 0=none, 1=helpful, 2=not_helpful, 3=expected
    confidence: u16,    // Predicted confidence (0-1000)
    // Operator health prediction (op=4)
    operator_id: u32,       // Operator ID being predicted
    predicted_latency: u32, // Predicted latency in microseconds
    actual_latency: u32,    // Actual latency after execution
    deadline_miss: u8,      // 0=pending, 1=met_deadline, 2=missed_deadline
}

const NN_AUDIT_ZERO: NnAuditEntry = NnAuditEntry {
    op: 0,
    in_len: 0,
    out_len: 0,
    ts_ns: 0,
    latency_us: 0,
    inputs_q88: [0; MAX_IN],
    targets_q88: [0; MAX_OUT],
    outputs_q88: [0; MAX_OUT],
    command_hash: 0,
    outcome: 0,
    feedback: 0,
    confidence: 0,
    operator_id: 0,
    predicted_latency: 0,
    actual_latency: 0,
    deadline_miss: 0,
};

struct NnAuditRing<const N: usize> {
    buf: [NnAuditEntry; N],
    idx: usize,
    filled: bool,
}

impl<const N: usize> NnAuditRing<N> {
    const fn new() -> Self { Self { buf: [NN_AUDIT_ZERO; N], idx: 0, filled: false } }
    fn push(&mut self, e: NnAuditEntry) {
        self.buf[self.idx] = e;
        self.idx = (self.idx + 1) % N;
        if self.idx == 0 { self.filled = true; }
    }
}

static NN_AUDIT: Mutex<NnAuditRing<8>> = Mutex::new(NnAuditRing::new());
// Learning mode state
struct LearnState { on: bool, limit: usize }
static LEARN: Mutex<LearnState> = Mutex::new(LearnState { on: false, limit: 1 });

// --- Autonomous Scheduling Configuration ---
pub struct NeuralSchedulingConfig {
    pub enabled: bool,
    pub confidence_threshold: u16,  // 0-1000 milli-units
    pub priority_boost: u8,
    pub max_boosts_per_window: usize,
    boost_count: usize,  // Current boost count (for rate limiting)
}

impl NeuralSchedulingConfig {
    pub const fn new() -> Self {
        Self {
            enabled: true,           // Autonomous scheduling ON by default
            confidence_threshold: 700,  // Require 70% confidence to boost
            priority_boost: 20,      // Boost priority by 20 points
            max_boosts_per_window: 100,  // Max 100 boosts per window
            boost_count: 0,
        }
    }

    pub fn reset_boost_count(&mut self) {
        self.boost_count = 0;
    }

    pub fn can_boost(&mut self) -> bool {
        if self.boost_count >= self.max_boosts_per_window {
            return false;
        }
        self.boost_count += 1;
        true
    }
}

static SCHED_CONFIG: Mutex<NeuralSchedulingConfig> = Mutex::new(NeuralSchedulingConfig::new());

// --- Scheduling Audit Ring ---
#[derive(Copy, Clone)]
struct SchedulingAuditEntry {
    timestamp_ns: u64,
    operator_id: u8,
    event_type: u8,  // 1=prediction, 2=boost, 3=retrain
    confidence: u16,
    old_priority: u8,
    new_priority: u8,
    latency_us: u32,
    deadline_missed: u8,  // 0=met, 1=missed
}

impl SchedulingAuditEntry {
    pub const fn empty() -> Self {
        Self {
            timestamp_ns: 0,
            operator_id: 0,
            event_type: 0,
            confidence: 0,
            old_priority: 0,
            new_priority: 0,
            latency_us: 0,
            deadline_missed: 0,
        }
    }
}

struct SchedulingAuditRing<const N: usize> {
    buf: [SchedulingAuditEntry; N],
    idx: usize,
    filled: bool,
}

impl<const N: usize> SchedulingAuditRing<N> {
    const fn new() -> Self {
        Self {
            buf: [SchedulingAuditEntry::empty(); N],
            idx: 0,
            filled: false,
        }
    }

    fn push(&mut self, entry: SchedulingAuditEntry) {
        self.buf[self.idx] = entry;
        self.idx += 1;
        if self.idx >= N {
            self.idx = 0;
            self.filled = true;
        }
    }

    fn iter(&self) -> impl Iterator<Item = &SchedulingAuditEntry> {
        let n = if self.filled { N } else { self.idx };
        let start = if self.filled { self.idx } else { 0 };
        (0..n).map(move |i| &self.buf[(start + i) % N])
    }
}

static SCHED_AUDIT: Mutex<SchedulingAuditRing<32>> = Mutex::new(SchedulingAuditRing::new());

pub fn reset() {
    let mut n = NEURAL.lock();
    n.reset_defaults();
}

pub fn infer_from_milli(inputs_milli: &[i32]) -> usize {
    let mut n = NEURAL.lock();
    // Convert 0..1000 milli to Q8.8
    let mut buf = [0i16; MAX_IN];
    let len = inputs_milli.len().min(n.in_sz);
    for i in 0..len {
        let q = (inputs_milli[i]).clamp(-32768, 32767);
        buf[i] = ((q * 256) / 1000) as i16;
    }
    n.infer(&buf[..len]);
    n.last_out_len
}

pub fn update_from_milli(vals_milli: &[i32]) -> bool {
    let mut n = NEURAL.lock();
    let need = n.hid_sz * n.in_sz + n.hid_sz + n.out_sz * n.hid_sz + n.out_sz;
    if vals_milli.len() < need { return false; }
    let mut tmp = [0i16; (MAX_H*MAX_IN + MAX_H + MAX_OUT*MAX_H + MAX_OUT)];
    for i in 0..need {
        let q = vals_milli[i].clamp(-32768, 32767);
        tmp[i] = ((q * 256) / 1000) as i16;
    }
    n.update_weights(&tmp[..need])
}

pub fn print_status() { NEURAL.lock().print_status(); }

#[inline(always)]
fn print_i32(v: i32) {
    if v < 0 {
        unsafe { crate::uart_print(b"-"); }
        let n = -(v as i64) as u64;
        crate::shell::print_number_simple(n);
    } else {
        crate::shell::print_number_simple(v as u64);
    }
}

pub fn selftest() -> bool {
    let mut n = NEURAL.lock();
    n.reset_defaults();
    // Test vector in milli
    let inputs = [1000i32, 200i32, 300i32];
    let mut buf = [0i16; MAX_IN];
    let insz = n.in_sz;
    for i in 0..insz { buf[i] = ((inputs[i] * 256) / 1000) as i16; }
    n.infer(&buf[..insz]);
    // Convert last_out to milli
    let to_milli = |q: i16| -> i32 { (q as i32) * 1000 / 256 };
    let o0 = to_milli(n.last_out[0]);
    let o1 = to_milli(n.last_out[1]);
    let ok0 = (o0 - inputs[0]).abs() <= 8;
    let ok1 = (o1 - inputs[1]).abs() <= 8;
    let ok = ok0 && ok1;
    metric_kv("nn_selftest_ok", if ok { 1 } else { 0 });
    ok
}

/// Get last outputs in milli; returns number of outputs copied
pub fn last_outputs_milli(buf: &mut [i32]) -> usize {
    let n = NEURAL.lock();
    let len = n.last_out_len.min(buf.len());
    for i in 0..len { buf[i] = (n.last_out[i] as i32) * 1000 / 256; }
    len
}

/// Get current neural agent dimensions (in, hidden, out)
pub fn dims() -> (usize, usize, usize) {
    let n = NEURAL.lock();
    (n.in_sz, n.hid_sz, n.out_sz)
}

/// One bounded gradient-like update step using milli inputs/targets;
/// performs simple backprop with small learning-rate shifts.
pub fn teach_milli(inputs_milli: &[i32], targets_milli: &[i32]) -> bool {
    let mut n = NEURAL.lock();
    if inputs_milli.len() < n.in_sz || targets_milli.len() < n.out_sz { return false; }
    if !n.dims_ok() { return false; }
    // Config: learning rate via bit shifts (smaller => slower updates)
    const LR_SHIFT_B2: i32 = 6;   // biases out
    const LR_SHIFT_W2: i32 = 8;   // weights out
    const LR_SHIFT_B1: i32 = 7;   // biases hidden
    const LR_SHIFT_W1: i32 = 9;   // weights hidden

    // Prepare Q8.8 inputs
    let mut x = [0i16; MAX_IN];
    for i in 0..n.in_sz { x[i] = ((inputs_milli[i] * 256) / 1000) as i16; }

    // Forward pass to get hidden and out
    let mut hid = [0i16; MAX_H];
    let mut out = [0i16; MAX_OUT];
    for r in 0..n.hid_sz {
        let mut acc: i32 = (n.b1[r] as i32) << 8;
        for c in 0..n.in_sz { acc = acc.saturating_add(q88_mul(n.w1[r][c], x[c])); }
        hid[r] = q88_relu(q88_acc_scale(acc));
    }
    for r in 0..n.out_sz {
        let mut acc: i32 = (n.b2[r] as i32) << 8;
        for c in 0..n.hid_sz { acc = acc.saturating_add(q88_mul(n.w2[r][c], hid[c])); }
        out[r] = q88_acc_scale(acc);
    }

    // Compute error at output: e = t - y (Q8.8)
    let mut e = [0i16; MAX_OUT];
    for r in 0..n.out_sz {
        let t = ((targets_milli[r] * 256) / 1000) as i16;
        e[r] = t.saturating_sub(out[r]);
    }

    // Update W2 and b2: w2 += (e*hid) >> (8+LR_SHIFT_W2), b2 += e >> LR_SHIFT_B2
    for r in 0..n.out_sz {
        let eb = (e[r] as i32) >> LR_SHIFT_B2; // Q8.8 scaled down
        let nb = (n.b2[r] as i32).saturating_add(eb);
        n.b2[r] = nb.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        for c in 0..n.hid_sz {
            let prod = (e[r] as i32) * (hid[c] as i32); // Q16.16
            let delta = prod >> (8 + LR_SHIFT_W2);
            let nw = (n.w2[r][c] as i32).saturating_add(delta);
            n.w2[r][c] = nw.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        }
    }

    // Hidden error term (approx relu' by hid>0)
    let mut dh = [0i16; MAX_H];
    for c in 0..n.hid_sz {
        if hid[c] <= 0 { dh[c] = 0; continue; }
        let mut acc: i32 = 0;
        for r in 0..n.out_sz {
            acc = acc.saturating_add(((e[r] as i32) * (n.w2[r][c] as i32)) >> 8); // backprop to hidden (Q8.8)
        }
        // Scale to i16
        dh[c] = acc.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
    }

    // Update W1 and b1: w1 += (dh * x) >> (8+LR_SHIFT_W1), b1 += dh >> LR_SHIFT_B1
    for r in 0..n.hid_sz {
        let db = (dh[r] as i32) >> LR_SHIFT_B1;
        let nb = (n.b1[r] as i32).saturating_add(db);
        n.b1[r] = nb.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        for c in 0..n.in_sz {
            let prod = (dh[r] as i32) * (x[c] as i32); // Q16.16
            let delta = prod >> (8 + LR_SHIFT_W1);
            let nw = (n.w1[r][c] as i32).saturating_add(delta);
            n.w1[r][c] = nw.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        }
    }

    n.teach_count = n.teach_count.saturating_add(1);
    metric_kv("nn_teach_count", n.teach_count);
    // Audit teach with inputs/targets
    let mut entry = NN_AUDIT_ZERO;
    entry.op = 2;
    entry.in_len = n.in_sz as u8;
    entry.out_len = n.out_sz as u8;
    entry.ts_ns = crate::graph::cycles_to_ns(crate::graph::now_cycles());
    entry.latency_us = 0;
    for i in 0..n.in_sz { entry.inputs_q88[i] = x[i]; }
    for i in 0..n.out_sz { entry.targets_q88[i] = ((targets_milli[i] * 256) / 1000) as i16; }
    NN_AUDIT.lock().push(entry);
    true
}

// --- Command Outcome Prediction & Feedback ---

/// Simple hash function for command names (djb2)
fn hash_command(cmd: &str) -> u32 {
    let mut hash: u32 = 5381;
    for b in cmd.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(b as u32);
    }
    hash
}

/// Extract features from command string for prediction
fn extract_command_features(cmd: &str) -> [i16; MAX_IN] {
    let mut features = [0i16; MAX_IN];
    // Feature 0: command length (normalized to milli, max 50 chars)
    features[0] = ((cmd.len().min(50) * 1000 / 50) as i32 * 256 / 1000) as i16;

    // Feature 1: has arguments (0 or 1000 milli)
    let has_args = if cmd.contains(' ') { 1000 } else { 0 };
    features[1] = (has_args * 256 / 1000) as i16;

    // Feature 2: starts with known prefix (simplified heuristic)
    let known_prefixes = ["graphctl", "neuralctl", "llmctl", "help", "metrics"];
    let is_known = known_prefixes.iter().any(|prefix| cmd.starts_with(prefix));
    features[2] = if is_known { (1000 * 256 / 1000) as i16 } else { 0 };

    features
}

/// Command telemetry for tracking command patterns
struct CommandTelemetry {
    last_command_ts_us: u64,
    commands_last_second: u16,
    last_second_start_us: u64,
    total_predictions: u32,
    accurate_predictions: u32,
}

impl CommandTelemetry {
    const fn new() -> Self {
        CommandTelemetry {
            last_command_ts_us: 0,
            commands_last_second: 0,
            last_second_start_us: 0,
            total_predictions: 0,
            accurate_predictions: 0,
        }
    }
}

static COMMAND_TELEMETRY: Mutex<CommandTelemetry> = Mutex::new(CommandTelemetry::new());

/// Predict command outcome before execution; returns (confidence_0_1000, predicted_success)
pub fn predict_command(cmd: &str) -> (u16, bool) {
    let mut n = NEURAL.lock();

    // Lazy init
    if n.infer_count == 0 {
        n.reset_defaults();
    }

    let features = extract_command_features(cmd);
    n.infer(&features[..3]);

    // Interpret outputs: output[0] = success likelihood, output[1] = failure likelihood
    let success_q88 = n.last_out[0];
    let fail_q88 = n.last_out[1];

    // Convert to milli (0-1000)
    let success_milli = ((success_q88 as i32) * 1000 / 256).max(0).min(1000);
    let fail_milli = ((fail_q88 as i32) * 1000 / 256).max(0).min(1000);

    // Confidence is the max of both (how certain we are)
    let confidence = success_milli.max(fail_milli) as u16;
    let predicted_success = success_milli >= fail_milli;

    // Log prediction to audit ring
    let cmd_hash = hash_command(cmd);
    let mut entry = NN_AUDIT_ZERO;
    entry.op = 3; // command_predict
    entry.in_len = 3;
    entry.out_len = 2;
    entry.ts_ns = crate::graph::cycles_to_ns(crate::graph::now_cycles());
    entry.command_hash = cmd_hash;
    entry.outcome = 0; // pending
    entry.confidence = confidence;
    for i in 0..3 { entry.inputs_q88[i] = features[i]; }
    for i in 0..2 { entry.outputs_q88[i] = n.last_out[i]; }
    NN_AUDIT.lock().push(entry);

    // Update command telemetry and publish messages
    let timestamp = crate::agent_bus::get_timestamp_us();
    let mut telem = COMMAND_TELEMETRY.lock();

    // Track command rate
    if timestamp / 1_000_000 > telem.last_second_start_us / 1_000_000 {
        // New second started
        telem.commands_last_second = 1;
        telem.last_second_start_us = timestamp;
    } else {
        telem.commands_last_second += 1;
    }
    telem.last_command_ts_us = timestamp;
    telem.total_predictions += 1;

    // Publish messages to agent bus for cross-subsystem coordination
    if confidence >= 300 {  // Only publish if confidence is sufficient (30%+)
        // Check if this is a "heavy" command (long string, high complexity)
        let cmd_len = cmd.len();
        let is_heavy = cmd_len > 20 || cmd.contains("stress") || cmd.contains("test");

        // Record prediction for tracking accuracy
        let prediction_type = if is_heavy {
            crate::prediction_tracker::PredictionType::CommandHeavy
        } else {
            crate::prediction_tracker::PredictionType::CommandRapidStream
        };
        let predicted_value = if predicted_success { 1 } else { 0 }; // 1 = success, 0 = failure
        let _pred_id = crate::prediction_tracker::record_prediction(
            prediction_type,
            predicted_value,
            confidence as i16,
        );

        if is_heavy {
            crate::agent_bus::publish_message(crate::agent_bus::AgentMessage::CommandHeavyPredicted {
                command_hash: cmd_hash,
                confidence,
                timestamp_us: timestamp,
            });
        }

        // Detect rapid command stream
        if telem.commands_last_second >= 10 {
            crate::agent_bus::publish_message(crate::agent_bus::AgentMessage::CommandRapidStream {
                commands_per_sec: telem.commands_last_second,
                confidence,
                timestamp_us: timestamp,
            });
        }

        // Check prediction accuracy
        let accuracy_percent = if telem.total_predictions > 10 {
            (telem.accurate_predictions * 100 / telem.total_predictions).min(100) as u8
        } else {
            50  // Default to 50% if not enough data
        };

        if accuracy_percent < 50 && telem.total_predictions > 20 {
            crate::agent_bus::publish_message(crate::agent_bus::AgentMessage::CommandLowAccuracy {
                recent_accuracy: accuracy_percent,
                timestamp_us: timestamp,
            });
        }
    }

    // Check for quiet period (no commands in last 5 seconds)
    if timestamp - telem.last_command_ts_us > 5_000_000 {
        let idle_seconds = ((timestamp - telem.last_command_ts_us) / 1_000_000).min(65535) as u16;
        crate::agent_bus::publish_message(crate::agent_bus::AgentMessage::CommandQuiet {
            idle_seconds,
            timestamp_us: timestamp,
        });
    }

    drop(telem);

    (confidence, predicted_success)
}

/// Record actual command outcome (1=success, 2=fail, 3=error)
pub fn record_command_outcome(cmd: &str, outcome: u8) {
    let cmd_hash = hash_command(cmd);
    let mut audit = NN_AUDIT.lock();

    let mut predicted_success = false;
    let mut found = false;

    // Find most recent prediction entry for this command
    let start = if audit.filled { audit.buf.len() } else { audit.idx };
    for i in (0..start).rev() {
        let idx = if audit.filled {
            (audit.idx + audit.buf.len() - 1 - i) % audit.buf.len()
        } else {
            start - 1 - i
        };

        if audit.buf[idx].op == 3 && audit.buf[idx].command_hash == cmd_hash && audit.buf[idx].outcome == 0 {
            audit.buf[idx].outcome = outcome;
            // Check if prediction was correct (output[0] > output[1] means predicted success)
            predicted_success = audit.buf[idx].outputs_q88[0] >= audit.buf[idx].outputs_q88[1];
            found = true;
            break;
        }
    }

    drop(audit);

    // Update accuracy telemetry if we found a prediction
    if found {
        let actual_success = outcome == 1;  // outcome 1 = success
        let was_accurate = predicted_success == actual_success;

        let mut telem = COMMAND_TELEMETRY.lock();
        if was_accurate {
            telem.accurate_predictions += 1;
        }
    }
}

/// Record user feedback for last prediction (1=helpful, 2=not_helpful, 3=expected)
pub fn record_feedback(feedback: u8) {
    let mut audit = NN_AUDIT.lock();

    // Find most recent command prediction entry
    let start = if audit.filled { audit.buf.len() } else { audit.idx };
    for i in (0..start).rev() {
        let idx = if audit.filled {
            (audit.idx + audit.buf.len() - 1 - i) % audit.buf.len()
        } else {
            start - 1 - i
        };

        if audit.buf[idx].op == 3 {
            audit.buf[idx].feedback = feedback;
            return;
        }
    }
}

/// Retrain using feedback-labeled command predictions from audit ring
pub fn retrain_from_feedback(max_steps: usize) -> usize {
    // First, collect training examples from audit ring
    let mut training_examples: heapless::Vec<([i32; 3], [i32; 2]), 8> = heapless::Vec::new();

    {
        let audit = NN_AUDIT.lock();
        let total = if audit.filled { audit.buf.len() } else { audit.idx };

        for i in 0..total.min(max_steps) {
            let idx = if audit.filled {
                (audit.idx + audit.buf.len() - 1 - i) % audit.buf.len()
            } else {
                total - 1 - i
            };

            let entry = &audit.buf[idx];

            // Only retrain on command/operator predictions with feedback or confirmed outcomes
            if entry.op != 3 && entry.op != 4 { continue; }

            // For commands: check outcome or feedback
            // For operators: check deadline_miss or feedback
            let has_data = if entry.op == 3 {
                entry.feedback != 0 || entry.outcome != 0
            } else { // op == 4
                entry.feedback != 0 || entry.deadline_miss != 0
            };
            if !has_data { continue; }

            // Determine target based on outcome or feedback
            let target_success = if entry.op == 3 {
                // Command prediction
                match (entry.outcome, entry.feedback) {
                    (1, _) => true,  // outcome=success
                    (2, _) | (3, _) => false, // outcome=fail or error
                    (_, 1) | (_, 3) => true,  // feedback=helpful or expected
                    (_, 2) => false, // feedback=not_helpful
                    _ => continue,
                }
            } else {
                // Operator prediction (op == 4)
                match (entry.deadline_miss, entry.feedback) {
                    (1, _) => true,  // met_deadline
                    (2, _) => false, // missed_deadline
                    (_, 1) | (_, 3) => true,  // feedback=helpful or expected
                    (_, 2) => false, // feedback=not_helpful
                    _ => continue,
                }
            };

            // Create training targets: [success_target, fail_target]
            let targets_milli = if target_success {
                [1000i32, 0i32] // high success, low fail
            } else {
                [0i32, 1000i32] // low success, high fail
            };

            // Extract inputs from entry
            let inputs_milli: [i32; 3] = [
                (entry.inputs_q88[0] as i32) * 1000 / 256,
                (entry.inputs_q88[1] as i32) * 1000 / 256,
                (entry.inputs_q88[2] as i32) * 1000 / 256,
            ];

            // Store for training
            if training_examples.push((inputs_milli, targets_milli)).is_err() {
                break; // Vec full
            }
        }
    } // Drop audit lock here

    // Now perform training without holding the lock
    let mut trained = 0;
    for (inputs, targets) in training_examples.iter() {
        let _ = teach_milli(&inputs[..3], &targets[..2]);
        trained += 1;
    }

    metric_kv("nn_retrain_steps", trained);
    trained
}

// ===================================================================
// Operator Health Prediction
// ===================================================================

/// Extract features from operator metrics for health prediction
/// Features: [avg_recent_latency, channel_depth, operator_priority]
fn extract_operator_features(_op_id: u32, recent_latency_us: u32, channel_depth: usize, priority: u8) -> [i16; MAX_IN] {
    let mut features = [0i16; MAX_IN];

    // Feature 0: Recent average latency (normalized: 0-10ms → 0-1000 milli)
    let latency_milli = (recent_latency_us.min(10000) * 1000 / 10000) as i32;
    features[0] = (latency_milli * 256 / 1000) as i16;

    // Feature 1: Channel backpressure (normalized: 0-64 depth → 0-1000 milli)
    let depth_milli = (channel_depth.min(64) * 1000 / 64) as i32;
    features[1] = (depth_milli * 256 / 1000) as i16;

    // Feature 2: Operator priority (normalized: 0-255 → 0-1000 milli)
    let prio_milli = (priority as usize * 1000 / 255) as i32;
    features[2] = (prio_milli * 256 / 1000) as i16;

    features
}

/// Predict operator health before execution
/// Returns (confidence 0-1000, will_meet_deadline)
pub fn predict_operator_health(op_id: u32, recent_latency_us: u32, channel_depth: usize, priority: u8) -> (u16, bool) {
    let mut n = NEURAL.lock();
    if n.infer_count == 0 { n.reset_defaults(); }

    let features = extract_operator_features(op_id, recent_latency_us, channel_depth, priority);
    n.infer(&features[..3]);

    let healthy_q88 = n.last_out[0];
    let unhealthy_q88 = n.last_out[1];
    let healthy_milli = ((healthy_q88 as i32) * 1000 / 256).max(0).min(1000);
    let unhealthy_milli = ((unhealthy_q88 as i32) * 1000 / 256).max(0).min(1000);

    let confidence = healthy_milli.max(unhealthy_milli) as u16;
    let will_meet_deadline = healthy_milli >= unhealthy_milli;

    // Log to audit ring
    let mut entry = NN_AUDIT_ZERO;
    entry.op = 4; // operator_predict
    entry.in_len = 3;
    entry.out_len = 2;
    entry.operator_id = op_id;
    entry.confidence = confidence;
    entry.deadline_miss = 0; // pending
    entry.predicted_latency = if will_meet_deadline { recent_latency_us } else { recent_latency_us * 2 };

    // Store features and outputs
    for i in 0..3 {
        entry.inputs_q88[i] = features[i];
    }
    entry.outputs_q88[0] = healthy_q88;
    entry.outputs_q88[1] = unhealthy_q88;

    drop(n); // Drop neural lock before acquiring audit lock
    NN_AUDIT.lock().push(entry);

    // Publish messages to agent bus for cross-subsystem coordination
    if confidence >= 300 {  // Only publish if confidence is sufficient (30%+)
        let timestamp = crate::agent_bus::get_timestamp_us();

        if !will_meet_deadline {
            // Operator is predicted to miss deadline
            if priority > 200 {  // Critical operator (high priority)
                crate::agent_bus::publish_message(crate::agent_bus::AgentMessage::SchedulingCriticalOperatorLatency {
                    operator_id: op_id,
                    latency_us: recent_latency_us,
                    confidence,
                    timestamp_us: timestamp,
                });
            } else {
                // General high load situation
                // Calculate deadline misses from recent audit history
                let recent_misses = count_recent_deadline_misses();
                crate::agent_bus::publish_message(crate::agent_bus::AgentMessage::SchedulingLoadHigh {
                    deadline_misses: recent_misses,
                    avg_latency_us: recent_latency_us,
                    confidence,
                    timestamp_us: timestamp,
                });
            }
        } else if recent_latency_us < 1000 && channel_depth == 0 {
            // Low load: fast execution, no backpressure
            let idle_percent = ((10000 - recent_latency_us) * 100 / 10000).min(100);
            crate::agent_bus::publish_message(crate::agent_bus::AgentMessage::SchedulingLoadLow {
                idle_percent: idle_percent as u8,
                timestamp_us: timestamp,
            });
        }
    }

    (confidence, will_meet_deadline)
}

/// Count recent deadline misses from audit log
fn count_recent_deadline_misses() -> u8 {
    let audit = NN_AUDIT.lock();
    let start = if audit.filled { audit.buf.len() } else { audit.idx };
    let mut misses = 0;

    // Check last 20 entries for deadline misses
    let check_count = start.min(20);
    for i in 0..check_count {
        let idx = if audit.idx == 0 {
            start - 1 - i
        } else {
            (audit.idx + audit.buf.len() - 1 - i) % audit.buf.len()
        };
        if audit.buf[idx].deadline_miss == 2 {  // 2 = missed
            misses += 1;
        }
    }

    misses.min(255) as u8
}

/// Record actual operator outcome after execution
pub fn record_operator_outcome(op_id: u32, actual_latency_us: u32, missed_deadline: bool) {
    let mut audit = NN_AUDIT.lock();
    let start = if audit.filled { audit.buf.len() } else { audit.idx };

    // Find most recent prediction for this operator
    for i in (0..start).rev() {
        let idx = if audit.idx == 0 { start - 1 - i } else { (audit.idx + audit.buf.len() - 1 - i) % audit.buf.len() };
        if audit.buf[idx].op == 4 &&
           audit.buf[idx].operator_id == op_id &&
           audit.buf[idx].deadline_miss == 0 {
            audit.buf[idx].actual_latency = actual_latency_us;
            audit.buf[idx].deadline_miss = if missed_deadline { 2 } else { 1 };
            break;
        }
    }
}

/// Record user feedback on operator health prediction
pub fn record_operator_feedback(op_id: u32, feedback: u8) {
    let mut audit = NN_AUDIT.lock();
    let start = if audit.filled { audit.buf.len() } else { audit.idx };

    // Find most recent operator prediction
    for i in (0..start).rev() {
        let idx = if audit.idx == 0 { start - 1 - i } else { (audit.idx + audit.buf.len() - 1 - i) % audit.buf.len() };
        if audit.buf[idx].op == 4 && audit.buf[idx].operator_id == op_id {
            audit.buf[idx].feedback = feedback;
            return;
        }
    }
}

/// Print the audit ring as JSON array on UART (inputs/targets in milli)
pub fn audit_print_json() {
    let a = NN_AUDIT.lock();
    let total = if a.filled { a.buf.len() } else { a.idx };
    unsafe { crate::uart_print(b"["); }
    let mut first = true;
    for k in 0..total {
        let pos = if a.idx == 0 { total - 1 - k } else { (a.idx + a.buf.len() - 1 - k) % a.buf.len() };
        let e = a.buf[pos];
        if !first { unsafe { crate::uart_print(b", "); } } else { first = false; }
        unsafe { crate::uart_print(b"{\"op\":"); }
        crate::shell::print_number_simple(e.op as u64);
        unsafe { crate::uart_print(b",\"in_len\":"); }
        crate::shell::print_number_simple(e.in_len as u64);
        unsafe { crate::uart_print(b",\"out_len\":"); }
        crate::shell::print_number_simple(e.out_len as u64);
        unsafe { crate::uart_print(b",\"lat_us\":"); }
        crate::shell::print_number_simple(e.latency_us as u64);
        unsafe { crate::uart_print(b",\"ts_ns\":"); }
        crate::shell::print_number_simple(e.ts_ns as u64);
        // inputs
        unsafe { crate::uart_print(b",\"in\":["); }
        for i in 0..(e.in_len as usize) {
            if i>0 { unsafe { crate::uart_print(b","); } }
            let milli = ((e.inputs_q88[i] as i32) * 1000 / 256) as i64;
            if milli < 0 { unsafe { crate::uart_print(b"-"); } crate::shell::print_number_simple((-milli) as u64); }
            else { crate::shell::print_number_simple(milli as u64); }
        }
        unsafe { crate::uart_print(b"],\"target\":["); }
        for i in 0..(e.out_len as usize) {
            if i>0 { unsafe { crate::uart_print(b","); } }
            let milli = ((e.targets_q88[i] as i32) * 1000 / 256) as i64;
            if milli < 0 { unsafe { crate::uart_print(b"-"); } crate::shell::print_number_simple((-milli) as u64); }
            else { crate::shell::print_number_simple(milli as u64); }
        }
        unsafe { crate::uart_print(b"],\"out\":["); }
        for i in 0..(e.out_len as usize) {
            if i>0 { unsafe { crate::uart_print(b","); } }
            let milli = ((e.outputs_q88[i] as i32) * 1000 / 256) as i64;
            if milli < 0 { unsafe { crate::uart_print(b"-"); } crate::shell::print_number_simple((-milli) as u64); }
            else { crate::shell::print_number_simple(milli as u64); }
        }
        unsafe { crate::uart_print(b"]}"); }
    }
    unsafe { crate::uart_print(b"]\n"); }
}

/// Retrain: replay up to `count` recent teach entries
pub fn retrain(count: usize) -> usize {
    let mut applied = 0usize;
    // Snapshot entries first to avoid holding lock during teach
    let mut items: heapless::Vec<(heapless::Vec<i32, MAX_IN>, heapless::Vec<i32, MAX_OUT>), 8> = heapless::Vec::new();
    {
        let a = NN_AUDIT.lock();
        let total = if a.filled { a.buf.len() } else { a.idx };
        let mut k = 0usize;
        while k < total && items.len() < count {
            let pos = if a.idx == 0 { total - 1 - k } else { (a.idx + a.buf.len() - 1 - k) % a.buf.len() };
            let e = a.buf[pos];
            if e.op == 2 {
                let mut iv = heapless::Vec::<i32, MAX_IN>::new();
                let mut tv = heapless::Vec::<i32, MAX_OUT>::new();
                for i in 0..(e.in_len as usize) { let _ = iv.push((e.inputs_q88[i] as i32) * 1000 / 256); }
                for i in 0..(e.out_len as usize) { let _ = tv.push((e.targets_q88[i] as i32) * 1000 / 256); }
                let _ = items.push((iv, tv));
            }
            k += 1;
        }
    }
    for (iv, tv) in items.iter() {
        if teach_milli(iv, tv) { applied += 1; }
    }
    applied
}

/// Run an action on feature inputs (milli), record op=3 audit, and return number of outputs
pub fn act_milli(inputs_milli: &[i32]) -> usize {
    let out_len = infer_from_milli(inputs_milli);
    // Build audit entry (op=3) with inputs and last outputs
    let mut e = NN_AUDIT_ZERO;
    e.op = 3;
    e.ts_ns = crate::graph::cycles_to_ns(crate::graph::now_cycles());
    {
        let n = NEURAL.lock();
        e.in_len = n.last_in_len as u8;
        e.out_len = n.last_out_len as u8;
        for i in 0..n.last_in_len { e.inputs_q88[i] = n.last_in[i]; }
        for i in 0..n.last_out_len { e.outputs_q88[i] = n.last_out[i]; }
    }
    NN_AUDIT.lock().push(e);
    out_len
}

/// Enable/disable learning mode and optionally set replay limit per tick
pub fn learn_set(on: bool, limit: Option<usize>) {
    let mut s = LEARN.lock();
    s.on = on;
    if let Some(l) = limit { s.limit = l.max(1).min(16); }
}

/// Cooperative learning tick: if enabled, replay up to limit teach entries
pub fn learn_tick() -> usize {
    let s = LEARN.lock();
    if !s.on { return 0; }
    let limit = s.limit;
    drop(s);
    let applied = retrain(limit);
    metric_kv("nn_learn_tick_applied", applied);
    applied
}

/// Dump dims and all weights in milli for persistence
pub fn dump_milli() {
    let n = NEURAL.lock();
    unsafe { crate::uart_print(b"NN DUMP dims="); }
    crate::shell::print_number_simple(n.in_sz as u64);
    unsafe { crate::uart_print(b", "); }
    crate::shell::print_number_simple(n.hid_sz as u64);
    unsafe { crate::uart_print(b", "); }
    crate::shell::print_number_simple(n.out_sz as u64);
    unsafe { crate::uart_print(b" weights="); }
    for r in 0..n.hid_sz { for c in 0..n.in_sz {
        unsafe { crate::uart_print(b" "); }
        let milli = (n.w1[r][c] as i32) * 1000 / 256;
        print_i32(milli);
    }}
    for r in 0..n.hid_sz {
        unsafe { crate::uart_print(b" "); }
        let milli = (n.b1[r] as i32) * 1000 / 256;
        print_i32(milli);
    }
    for r in 0..n.out_sz { for c in 0..n.hid_sz {
        unsafe { crate::uart_print(b" "); }
        let milli = (n.w2[r][c] as i32) * 1000 / 256;
        print_i32(milli);
    }}
    for r in 0..n.out_sz {
        unsafe { crate::uart_print(b" "); }
        let milli = (n.b2[r] as i32) * 1000 / 256;
        print_i32(milli);
    }
    unsafe { crate::uart_print(b"\n"); }
}

/// Load both dims and weights from milli sequence: first three numbers are dims; then weights
pub fn load_all_milli(dims: (usize, usize, usize), weights_milli: &[i32]) -> bool {
    let (di, dh, do_) = dims;
    let mut n = NEURAL.lock();
    if !n.set_dims(di, dh, do_) { return false; }
    drop(n);
    update_from_milli(weights_milli)
}

// --- Autonomous Scheduling API ---

/// Enable or disable autonomous scheduling
pub fn set_autonomous_enabled(enabled: bool) {
    let mut cfg = SCHED_CONFIG.lock();
    cfg.enabled = enabled;
    metric_kv("neural_autonomous_mode", enabled as usize);
}

/// Get autonomous scheduling enabled state
pub fn get_autonomous_enabled() -> bool {
    SCHED_CONFIG.lock().enabled
}

/// Set scheduling configuration thresholds
pub fn set_scheduling_config(confidence_threshold: u16, priority_boost: u8, max_boosts: usize) {
    let mut cfg = SCHED_CONFIG.lock();
    cfg.confidence_threshold = confidence_threshold;
    cfg.priority_boost = priority_boost;
    cfg.max_boosts_per_window = max_boosts;
    metric_kv("neural_sched_conf_threshold", confidence_threshold as usize);
    metric_kv("neural_sched_boost", priority_boost as usize);
    metric_kv("neural_sched_max_boosts", max_boosts);
}

/// Get current scheduling configuration
pub fn get_scheduling_config() -> (bool, u16, u8, usize) {
    let cfg = SCHED_CONFIG.lock();
    (cfg.enabled, cfg.confidence_threshold, cfg.priority_boost, cfg.max_boosts_per_window)
}

/// Check if autonomous boost is allowed (respects rate limit)
pub fn can_autonomous_boost() -> bool {
    let mut cfg = SCHED_CONFIG.lock();
    cfg.enabled && cfg.can_boost()
}

/// Get boost configuration values for graph execution
pub fn get_boost_params() -> (u16, u8) {
    let cfg = SCHED_CONFIG.lock();
    (cfg.confidence_threshold, cfg.priority_boost)
}

/// Reset boost count (call at start of each graph run)
pub fn reset_boost_count() {
    SCHED_CONFIG.lock().reset_boost_count();
}

/// Log a scheduling event to audit ring
pub fn log_scheduling_event(
    operator_id: u8,
    event_type: u8,  // 1=prediction, 2=boost, 3=retrain
    confidence: u16,
    old_priority: u8,
    new_priority: u8,
    latency_us: u32,
    deadline_missed: bool,
) {
    let entry = SchedulingAuditEntry {
        timestamp_ns: crate::graph::cycles_to_ns(crate::graph::now_cycles()),
        operator_id,
        event_type,
        confidence,
        old_priority,
        new_priority,
        latency_us,
        deadline_missed: deadline_missed as u8,
    };
    SCHED_AUDIT.lock().push(entry);
}

/// Print scheduling audit log
pub fn print_scheduling_audit() {
    let audit = SCHED_AUDIT.lock();
    unsafe { crate::uart_print(b"[SCHED AUDIT] Recent scheduling events (up to 32):\n"); }

    let mut count = 0;
    for entry in audit.iter() {
        if entry.event_type == 0 { continue; }  // Skip empty entries

        unsafe { crate::uart_print(b"  ["); }
        crate::shell::print_number_simple(count as u64);
        unsafe { crate::uart_print(b"] ts="); }
        crate::shell::print_number_simple(entry.timestamp_ns / 1000); // print in microseconds
        unsafe { crate::uart_print(b"us op="); }
        crate::shell::print_number_simple(entry.operator_id as u64);

        unsafe { crate::uart_print(b" type="); }
        match entry.event_type {
            1 => unsafe { crate::uart_print(b"PREDICT"); },
            2 => unsafe { crate::uart_print(b"BOOST"); },
            3 => unsafe { crate::uart_print(b"RETRAIN"); },
            _ => unsafe { crate::uart_print(b"UNKNOWN"); },
        }

        unsafe { crate::uart_print(b" conf="); }
        crate::shell::print_number_simple(entry.confidence as u64);

        if entry.event_type == 2 {  // BOOST event
            unsafe { crate::uart_print(b" prio="); }
            crate::shell::print_number_simple(entry.old_priority as u64);
            unsafe { crate::uart_print(b"->"); }
            crate::shell::print_number_simple(entry.new_priority as u64);
        }

        if entry.latency_us > 0 {
            unsafe { crate::uart_print(b" lat="); }
            crate::shell::print_number_simple(entry.latency_us as u64);
            unsafe { crate::uart_print(b"us"); }
        }

        if entry.deadline_missed != 0 {
            unsafe { crate::uart_print(b" DEADLINE_MISS"); }
        }

        unsafe { crate::uart_print(b"\n"); }
        count += 1;
    }

    if count == 0 {
        unsafe { crate::uart_print(b"  (no events yet)\n"); }
    }
}

// --- Memory Subsystem Neural Agent ---

/// Memory neural agent: separate network for memory management predictions
static MEMORY_AGENT: Mutex<NeuralAgent> = Mutex::new(NeuralAgent::new());

/// Memory telemetry for neural predictions
struct MemoryTelemetry {
    free_memory_percent: u32,      // 0-100
    allocation_rate: u32,           // Allocs per second (windowed)
    fragmentation_level: u32,       // 0-100 (estimated)
    recent_failures: u32,           // Failed allocations in last window
    last_update_ns: u64,            // Timestamp of last telemetry update
    prev_alloc_count: usize,        // Previous allocation count for rate calculation
}

impl MemoryTelemetry {
    const fn new() -> Self {
        Self {
            free_memory_percent: 100,
            allocation_rate: 0,
            fragmentation_level: 0,
            recent_failures: 0,
            last_update_ns: 0,
            prev_alloc_count: 0,
        }
    }
}

static MEMORY_TELEMETRY: Mutex<MemoryTelemetry> = Mutex::new(MemoryTelemetry::new());

/// Initialize memory neural agent with proper dimensions
#[inline(never)]
pub fn init_memory_agent() {
    unsafe { crate::uart_print(b"[MEM AGENT] ENTER\n"); }
    // Disable IRQs during brief lock to avoid early-boot reentrancy/deadlock
    unsafe { crate::uart_print(b"[MEM AGENT] IRQ OFF\n"); core::arch::asm!("msr daifset, #2", options(nostack, preserves_flags)); }
    unsafe { crate::uart_print(b"[MEM AGENT] LOCKING\n"); }
    let mut agent = MEMORY_AGENT.lock();
    unsafe { crate::uart_print(b"[MEM AGENT] LOCKED\n"); }
    let _ = agent.set_dims(4, 8, 2);
    unsafe { crate::uart_print(b"[MEM AGENT] DIMS SET\n"); }
    agent.infer_count = 1; // prevent lazy reset
    drop(agent);
    unsafe { crate::uart_print(b"[MEM AGENT] UNLOCKED\n"); }
    // Re-enable IRQs
    unsafe { crate::uart_print(b"[MEM AGENT] IRQ ON\n"); core::arch::asm!("msr daifclr, #2", options(nostack, preserves_flags)); }
    metric_kv("memory_agent_init", 1);
    unsafe { crate::uart_print(b"[MEM AGENT] DONE\n"); }
}

/// Update memory telemetry from heap stats
pub fn update_memory_telemetry() {
    let stats = crate::heap::get_heap_stats();
    let mut telem = MEMORY_TELEMETRY.lock();

    // Calculate free memory percentage
    let heap_size: usize = 100 * 1024; // 100 KiB (from heap.rs HEAP_SIZE)
    let used = stats.current_allocated();
    let free = heap_size.saturating_sub(used);
    telem.free_memory_percent = ((free * 100) / heap_size).min(100) as u32;

    // Calculate allocation rate (allocations per second)
    let now_ns = crate::graph::cycles_to_ns(crate::graph::now_cycles());
    if telem.last_update_ns > 0 {
        let delta_ns = now_ns.saturating_sub(telem.last_update_ns);
        if delta_ns > 0 {
            let delta_allocs = stats.total_allocations().saturating_sub(telem.prev_alloc_count);
            // Convert to per-second rate
            let rate = (delta_allocs as u64 * 1_000_000_000) / delta_ns;
            telem.allocation_rate = rate.min(1000) as u32; // Cap at 1000 allocs/sec
        }
    }

    // Estimate fragmentation (simple heuristic: peak vs current usage)
    if stats.peak_allocated() > 0 {
        let utilization = (stats.current_allocated() * 100) / stats.peak_allocated();
        // Lower utilization with many allocs/deallocs = higher fragmentation
        let churn = stats.total_deallocations().saturating_sub(stats.total_allocations() / 2);
        let frag_estimate = if churn > 10 {
            100u32.saturating_sub(utilization as u32)
        } else {
            0
        };
        telem.fragmentation_level = frag_estimate.min(100);
    }

    // Track recent allocation failures
    telem.recent_failures = stats.allocation_failures().min(10) as u32;

    // Update tracking state
    telem.last_update_ns = now_ns;
    telem.prev_alloc_count = stats.total_allocations();
}

/// Predict memory health and compaction need
/// Returns: (confidence, oom_risk, compact_needed)
pub fn predict_memory_health() -> (u16, bool, bool) {
    // Update telemetry before prediction
    update_memory_telemetry();

    let telem = MEMORY_TELEMETRY.lock();

    // Convert telemetry to Q8.8 inputs for neural network
    // Milli-units (0-1000) -> Q8.8 (0-256 for 100%)
    let inputs_q88 = [
        ((telem.free_memory_percent * 256 / 100).min(256)) as i16,      // Free memory %
        ((telem.allocation_rate * 256 / 1000).min(256)) as i16,         // Allocation rate (max 1000/sec)
        ((telem.fragmentation_level * 256 / 100).min(256)) as i16,      // Fragmentation %
        ((telem.recent_failures * 256 / 10).min(256)) as i16,           // Recent failures (max 10)
    ];

    drop(telem);

    // Run inference on MEMORY_AGENT
    let mut agent = MEMORY_AGENT.lock();
    let out_len = agent.infer(&inputs_q88);

    if out_len < 2 {
        drop(agent);
        return (0, false, false); // Not enough outputs
    }

    let out0 = agent.last_out[0]; // Memory health (Q8.8)
    let out1 = agent.last_out[1]; // Compaction need (Q8.8)
    drop(agent);

    // Convert Q8.8 to milli-units (0-1000)
    let health_milli = ((out0 as i32) * 1000 / 256).clamp(-1000, 1000);
    let compact_milli = ((out1 as i32) * 1000 / 256).clamp(-1000, 1000);

    // Compute confidence (average absolute value of outputs)
    let confidence = ((health_milli.abs() + compact_milli.abs()) / 2).min(1000) as u16;

    // Threshold for decisions
    let oom_risk = health_milli < -300;  // Negative output = unhealthy
    let compact_needed = compact_milli > 300;  // Positive output = compact needed

    // Record predictions for tracking accuracy
    // Memory pressure prediction (convert health to pressure: lower health = higher pressure)
    let pressure_value = ((-health_milli + 1000) / 2).clamp(0, 1000) as i16; // 0-1000 scale
    let _pressure_pred_id = crate::prediction_tracker::record_prediction(
        crate::prediction_tracker::PredictionType::MemoryPressure,
        pressure_value,
        confidence as i16,
    );

    // Compaction needed prediction
    let compact_value = if compact_needed { 1 } else { 0 };
    let _compact_pred_id = crate::prediction_tracker::record_prediction(
        crate::prediction_tracker::PredictionType::MemoryCompactionNeeded,
        compact_value,
        confidence as i16,
    );

    // Publish messages to agent bus for cross-subsystem coordination
    if confidence >= 300 {  // Only publish if confidence is sufficient (30%+)
        let timestamp = crate::agent_bus::get_timestamp_us();
        let telem = MEMORY_TELEMETRY.lock();

        if oom_risk || telem.free_memory_percent < 20 {
            // High memory pressure
            let pressure_level = (100 - telem.free_memory_percent).min(100) as u8;
            crate::agent_bus::publish_message(crate::agent_bus::AgentMessage::MemoryPressure {
                level: pressure_level,
                fragmentation: telem.fragmentation_level.min(100) as u8,
                confidence,
                timestamp_us: timestamp,
            });
        } else if compact_needed {
            // Compaction needed
            let urgency = ((compact_milli - 300) * 100 / 700).min(100) as u8;
            crate::agent_bus::publish_message(crate::agent_bus::AgentMessage::MemoryCompactionNeeded {
                urgency,
                confidence,
                timestamp_us: timestamp,
            });
        } else if telem.free_memory_percent >= 50 {
            // Memory is healthy
            crate::agent_bus::publish_message(crate::agent_bus::AgentMessage::MemoryHealthy {
                headroom_percent: telem.free_memory_percent.min(100) as u8,
                timestamp_us: timestamp,
            });
        }
    }

    (confidence, oom_risk, compact_needed)
}

/// Print memory agent status
pub fn print_memory_agent_status() {
    let telem = MEMORY_TELEMETRY.lock();
    unsafe { crate::uart_print(b"[MEM AGENT] Telemetry:\n"); }
    unsafe { crate::uart_print(b"  Free Memory: "); }
    crate::shell::print_number_simple(telem.free_memory_percent as u64);
    unsafe { crate::uart_print(b"%\n"); }
    unsafe { crate::uart_print(b"  Allocation Rate: "); }
    crate::shell::print_number_simple(telem.allocation_rate as u64);
    unsafe { crate::uart_print(b" /sec\n"); }
    unsafe { crate::uart_print(b"  Fragmentation: "); }
    crate::shell::print_number_simple(telem.fragmentation_level as u64);
    unsafe { crate::uart_print(b"%\n"); }
    unsafe { crate::uart_print(b"  Recent Failures: "); }
    crate::shell::print_number_simple(telem.recent_failures as u64);
    unsafe { crate::uart_print(b"\n"); }

    drop(telem);

    // Run prediction and show results
    let (conf, oom_risk, compact_needed) = predict_memory_health();

    unsafe { crate::uart_print(b"[MEM AGENT] Prediction:\n"); }
    unsafe { crate::uart_print(b"  Confidence: "); }
    crate::shell::print_number_simple(conf as u64);
    unsafe { crate::uart_print(b"/1000\n"); }
    unsafe { crate::uart_print(b"  OOM Risk: "); }
    if oom_risk {
        unsafe { crate::uart_print(b"YES\n"); }
    } else {
        unsafe { crate::uart_print(b"NO\n"); }
    }
    unsafe { crate::uart_print(b"  Compaction Needed: "); }
    if compact_needed {
        unsafe { crate::uart_print(b"YES\n"); }
    } else {
        unsafe { crate::uart_print(b"NO\n"); }
    }
}

/// Check memory health and emit autonomous warnings if issues detected
/// Call this periodically or after significant allocations
pub fn check_autonomous_memory_warnings() {
    let (conf, oom_risk, compact_needed) = predict_memory_health();

    // Only emit warnings if confidence is sufficient (>= 300/1000 = 30%)
    const MIN_CONFIDENCE: u16 = 300;

    if conf >= MIN_CONFIDENCE {
        if oom_risk {
            unsafe { crate::uart_print(b"\n[MEMORY AGENT] AUTONOMOUS WARNING: OOM RISK DETECTED (conf="); }
            crate::shell::print_number_simple(conf as u64);
            unsafe { crate::uart_print(b"/1000)\n"); }

            // Print current telemetry for debugging
            let telem = MEMORY_TELEMETRY.lock();
            unsafe { crate::uart_print(b"  Free Memory: "); }
            crate::shell::print_number_simple(telem.free_memory_percent as u64);
            unsafe { crate::uart_print(b"%\n"); }
            unsafe { crate::uart_print(b"  Alloc Rate: "); }
            crate::shell::print_number_simple(telem.allocation_rate as u64);
            unsafe { crate::uart_print(b"/sec\n"); }
            unsafe { crate::uart_print(b"  Recent Failures: "); }
            crate::shell::print_number_simple(telem.recent_failures as u64);
            unsafe { crate::uart_print(b"\n"); }

            metric_kv("memory_oom_warning", conf as usize);
        }

        if compact_needed {
            unsafe { crate::uart_print(b"\n[MEMORY AGENT] AUTONOMOUS WARNING: COMPACTION RECOMMENDED (conf="); }
            crate::shell::print_number_simple(conf as u64);
            unsafe { crate::uart_print(b"/1000)\n"); }

            let telem = MEMORY_TELEMETRY.lock();
            unsafe { crate::uart_print(b"  Fragmentation: "); }
            crate::shell::print_number_simple(telem.fragmentation_level as u64);
            unsafe { crate::uart_print(b"%\n"); }

            metric_kv("memory_compact_warning", conf as usize);
        }
    }
}

// --- Cross-Agent Coordination ---

/// Check agent bus and coordinate actions based on other agents' messages
/// This should be called periodically (e.g., every 100ms or on key events)
pub fn process_agent_coordination() {
    // Get recent messages from the last second (1,000,000 microseconds)
    let timestamp = crate::agent_bus::get_timestamp_us();
    let one_second_ago = timestamp.saturating_sub(1_000_000);
    let messages = crate::agent_bus::get_messages_since(one_second_ago);

    if messages.is_empty() {
        return; // No recent activity
    }

    // Track coordination actions taken
    let mut memory_pressure_detected = false;
    let mut scheduling_load_high = false;
    let mut rapid_commands_detected = false;

    // Scan for critical messages
    for msg in messages.iter() {
        match msg {
            crate::agent_bus::AgentMessage::MemoryPressure { level, confidence, .. } => {
                if *level > 70 && *confidence >= 300 {
                    memory_pressure_detected = true;
                }
            }
            crate::agent_bus::AgentMessage::SchedulingLoadHigh { deadline_misses, confidence, .. } => {
                if *deadline_misses > 3 && *confidence >= 300 {
                    scheduling_load_high = true;
                }
            }
            crate::agent_bus::AgentMessage::CommandRapidStream { commands_per_sec, .. } => {
                if *commands_per_sec > 15 {
                    rapid_commands_detected = true;
                }
            }
            _ => {}
        }
    }

    // Take coordinated actions

    // 1. Memory pressure → Scheduling should lower non-critical operators
    if memory_pressure_detected {
        unsafe { crate::uart_print(b"[COORDINATION] Memory pressure detected, adjusting scheduling\n"); }
        metric_kv("coord_memory_pressure_action", 1);
        // In a full implementation, this would call:
        // crate::graph::lower_noncritical_operator_priorities();
    }

    // 2. High scheduling load → Memory should be conservative
    if scheduling_load_high {
        unsafe { crate::uart_print(b"[COORDINATION] High scheduling load, memory entering conservative mode\n"); }
        metric_kv("coord_scheduling_load_action", 1);
        // Could reduce memory allocation aggressiveness or trigger early GC
    }

    // 3. Rapid commands → All agents enter defensive mode
    if rapid_commands_detected {
        unsafe { crate::uart_print(b"[COORDINATION] Rapid command stream detected, defensive mode\n"); }
        metric_kv("coord_rapid_commands_action", 1);
        // Could throttle predictions, increase confidence thresholds
    }

    // 4. Combined stress: all three conditions
    if memory_pressure_detected && scheduling_load_high && rapid_commands_detected {
        unsafe { crate::uart_print(b"[COORDINATION] CRITICAL: Multi-subsystem stress detected!\n"); }
        metric_kv("coord_multi_stress", 1);
        // Emergency coordination: shed load, reject new work, log alert
    }
}

/// Scheduling agent checks for memory pressure and adjusts priorities
pub fn scheduling_check_memory_coordination() -> bool {
    let timestamp = crate::agent_bus::get_timestamp_us();
    let recent_window = timestamp.saturating_sub(500_000); // Last 500ms
    let messages = crate::agent_bus::get_messages_since(recent_window);

    for msg in messages.iter() {
        if let crate::agent_bus::AgentMessage::MemoryPressure { level, confidence, .. } = msg {
            if *level > 70 && *confidence >= 400 {
                // High confidence memory pressure
                unsafe { crate::uart_print(b"[SCHED] Detected memory pressure, lowering non-critical priority\n"); }
                metric_kv("sched_memory_coordination", 1);
                return true; // Signal to lower priorities
            }
        }
    }
    false
}

/// Memory agent checks for scheduling stress and becomes conservative
pub fn memory_check_scheduling_coordination() -> bool {
    let timestamp = crate::agent_bus::get_timestamp_us();
    let recent_window = timestamp.saturating_sub(500_000); // Last 500ms
    let messages = crate::agent_bus::get_messages_since(recent_window);

    for msg in messages.iter() {
        if let crate::agent_bus::AgentMessage::SchedulingLoadHigh { deadline_misses, confidence, .. } = msg {
            if *deadline_misses > 5 && *confidence >= 400 {
                // High confidence scheduling stress
                unsafe { crate::uart_print(b"[MEM] Detected scheduling stress, entering conservative mode\n"); }
                metric_kv("mem_scheduling_coordination", 1);
                return true; // Signal to be conservative
            }
        }
    }
    false
}

/// Command agent checks for system stress and adjusts predictions
pub fn command_check_system_stress() -> bool {
    let timestamp = crate::agent_bus::get_timestamp_us();
    let recent_window = timestamp.saturating_sub(1_000_000); // Last 1 second
    let messages = crate::agent_bus::get_messages_since(recent_window);

    let mut stress_indicators = 0;

    for msg in messages.iter() {
        match msg {
            crate::agent_bus::AgentMessage::MemoryPressure { level, .. } if *level > 80 => {
                stress_indicators += 1;
            }
            crate::agent_bus::AgentMessage::SchedulingLoadHigh { deadline_misses, .. } if *deadline_misses > 5 => {
                stress_indicators += 1;
            }
            _ => {}
        }
    }

    if stress_indicators >= 2 {
        unsafe { crate::uart_print(b"[CMD] Detected system stress, throttling predictions\n"); }
        metric_kv("cmd_system_stress_coordination", 1);
        return true;
    }
    false
}

/// Get coordination statistics
pub fn get_coordination_stats() -> (usize, usize, usize) {
    let timestamp = crate::agent_bus::get_timestamp_us();
    let recent_window = timestamp.saturating_sub(5_000_000); // Last 5 seconds
    let messages = crate::agent_bus::get_messages_since(recent_window);

    let mut memory_events = 0;
    let mut scheduling_events = 0;
    let mut command_events = 0;

    for msg in messages.iter() {
        match msg {
            crate::agent_bus::AgentMessage::MemoryPressure { .. }
            | crate::agent_bus::AgentMessage::MemoryCompactionNeeded { .. }
            | crate::agent_bus::AgentMessage::MemoryHealthy { .. } => {
                memory_events += 1;
            }
            crate::agent_bus::AgentMessage::SchedulingLoadHigh { .. }
            | crate::agent_bus::AgentMessage::SchedulingLoadLow { .. }
            | crate::agent_bus::AgentMessage::SchedulingCriticalOperatorLatency { .. } => {
                scheduling_events += 1;
            }
            crate::agent_bus::AgentMessage::CommandHeavyPredicted { .. }
            | crate::agent_bus::AgentMessage::CommandRapidStream { .. }
            | crate::agent_bus::AgentMessage::CommandLowAccuracy { .. }
            | crate::agent_bus::AgentMessage::CommandQuiet { .. } => {
                command_events += 1;
            }
        }
    }

    (memory_events, scheduling_events, command_events)
}
