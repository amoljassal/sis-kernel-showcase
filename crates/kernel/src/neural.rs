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
    infer_count: usize,
    teach_count: usize,
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
    pub fn infer(&mut self, input_q88: &[i16]) -> usize {
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
    op: u8,            // 1=infer, 2=teach
    in_len: u8,
    out_len: u8,
    ts_ns: u64,
    latency_us: u32,
    inputs_q88: [i16; MAX_IN],
    targets_q88: [i16; MAX_OUT], // only set for teach
    outputs_q88: [i16; MAX_OUT], // only set for infer
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

static NN_AUDIT: Mutex<NnAuditRing<32>> = Mutex::new(NnAuditRing::new());
// Learning mode state
struct LearnState { on: bool, limit: usize }
static LEARN: Mutex<LearnState> = Mutex::new(LearnState { on: false, limit: 1 });

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
    let mut items: heapless::Vec<(heapless::Vec<i32, MAX_IN>, heapless::Vec<i32, MAX_OUT>), 16> = heapless::Vec::new();
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
