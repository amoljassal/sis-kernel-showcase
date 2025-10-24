//! Kernel-resident LLM operator/service (feature: `llm`).
//!
//! Phase 0/1 skeleton:
//! - Defines buffer schemas for TEXT/TOKENS
//! - Provides a stub `infer` implementation with deterministic, bounded work
//! - Emits basic METRICs (latency, tokens)
//! - Exposes simple load/stats helpers used by shell commands
//!
//! This module avoids heavy deps and large models. It is intended to validate
//! control paths, scheduling hooks, and observability before wiring real backends.

#![allow(dead_code)]

extern crate alloc;

use alloc::string::String;
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::Mutex;

use crate::trace::metric_kv;
use crate::model;

/// Typed schema identifiers for text/tokens carried via tensor headers (future wiring).
pub const SCHEMA_TEXT: u32 = 1001;
pub const SCHEMA_TOKENS: u32 = 1002;

/// Simple LLM service configuration
#[derive(Clone, Copy)]
pub struct LlmConfig {
    /// Worst-case execution time budget (cycles) for sizing CBS budgets later
    pub wcet_cycles: u64,
    /// Default maximum tokens per inference
    pub default_max_tokens: usize,
    /// Period (ns) for budgeting; 0 disables period accounting
    pub period_ns: u64,
    /// Max tokens allowed per period; 0 = unlimited
    pub max_tokens_per_period: usize,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self { wcet_cycles: 25_000, default_max_tokens: 64, period_ns: 0, max_tokens_per_period: 0 }
    }
}

/// Runtime counters for observability
struct LlmState {
    cfg: LlmConfig,
    current_model: Option<ModelMeta>,
    queue_depth: usize,
    queue_depth_max: usize,
    total_tokens: usize,
    deadline_miss_count: usize,
    rejects: usize,
    last_latency_us: usize,
    // Period accounting
    period_window_start_ns: u64,
    period_tokens_issued: usize,
}

impl LlmState {
    pub const fn new() -> Self {
        Self {
            cfg: LlmConfig { wcet_cycles: 25_000, default_max_tokens: 64, period_ns: 0, max_tokens_per_period: 0 },
            current_model: None,
            queue_depth: 0,
            queue_depth_max: 0,
            total_tokens: 0,
            deadline_miss_count: 0,
            rejects: 0,
            last_latency_us: 0,
            period_window_start_ns: 0,
            period_tokens_issued: 0,
        }
    }
}

static STATE: Mutex<LlmState> = Mutex::new(LlmState::new());
static INFER_ID: AtomicUsize = AtomicUsize::new(1);

// --- Simple control-plane polling state (last inference only) ---
pub struct LastInferState {
    pub infer_id: usize,
    pub tokens: heapless::Vec<heapless::String<32>, 128>,
    pub read_idx: usize,
    pub done: bool,
}

impl LastInferState {
    pub const fn new() -> Self {
        Self { infer_id: 0, tokens: heapless::Vec::new(), read_idx: 0, done: true }
    }
}

static LAST_INFER: Mutex<LastInferState> = Mutex::new(LastInferState::new());

#[derive(Clone)]
pub struct InferState {
    pub infer_id: usize,
    pub tokens: heapless::Vec<heapless::String<32>, 128>,
    pub read_idx: usize,
    pub done: bool,
    pub ts_ns: u64,
    pub model_id: Option<u32>,
    pub prompt_len: usize,
}

impl InferState {
    pub fn new(id: usize, toks: heapless::Vec<heapless::String<32>, 128>, ts_ns: u64, model_id: Option<u32>, prompt_len: usize) -> Self {
        Self { infer_id: id, tokens: toks, read_idx: 0, done: true, ts_ns, model_id, prompt_len }
    }
}

static INFER_TABLE: Mutex<heapless::Vec<InferState, 32>> = Mutex::new(heapless::Vec::new());
static MODEL_SEC: Mutex<model::ModelSecurityManager<4, 64>> = Mutex::new(model::ModelSecurityManager::new());

fn record_infer_state(
    id: usize,
    toks: heapless::Vec<heapless::String<32>, 128>,
    model_id: Option<u32>,
    prompt_len: usize,
) {
    // Update last-infer for backward compatibility
    {
        let mut li = LAST_INFER.lock();
        li.infer_id = id;
        li.tokens.clear();
        for t in toks.iter() { let _ = li.tokens.push(t.clone()); }
        li.read_idx = 0;
        li.done = true;
    }
    // Insert into table, evict oldest if full
    let mut tab = INFER_TABLE.lock();
    if tab.len() == tab.capacity() { let _ = tab.remove(0); }
    // Capture timestamp
    let ts_ns = crate::graph::cycles_to_ns(crate::graph::now_cycles());
    let _ = tab.push(InferState::new(id, toks, ts_ns, model_id, prompt_len));
}

// --- Audit log (ring buffer) ---
#[derive(Copy, Clone)]
struct AuditEntry {
    op: u8,                // 1=load,2=budget,3=infer,4=stream
    prompt_len: usize,
    tokens: usize,
    wcet_cycles: u64,
    period_ns: u64,
    status: u8,            // bit0=ok, bit1=reject, bit2=deadline_miss
    ts_ns: u64,
}

struct AuditRing<const N: usize> {
    buf: [AuditEntry; N],
    idx: usize,
    filled: bool,
}

impl<const N: usize> AuditRing<N> {
    const fn new(z: AuditEntry) -> Self { Self { buf: [z; N], idx: 0, filled: false } }
    fn push(&mut self, e: AuditEntry) {
        self.buf[self.idx] = e;
        self.idx = (self.idx + 1) % N;
        if self.idx == 0 { self.filled = true; }
    }
}

const AUDIT_ZERO: AuditEntry = AuditEntry { op:0, prompt_len:0, tokens:0, wcet_cycles:0, period_ns:0, status:0, ts_ns:0 };
static AUDIT: Mutex<AuditRing<32>> = Mutex::new(AuditRing::new(AUDIT_ZERO));

pub fn audit(op: u8, prompt_len: usize, tokens: usize, wcet_cycles: u64, period_ns: u64, status: u8) {
    let ts_ns = crate::graph::cycles_to_ns(crate::graph::now_cycles());
    let mut a = AUDIT.lock();
    a.push(AuditEntry { op, prompt_len, tokens, wcet_cycles, period_ns, status, ts_ns });
}

pub fn audit_print() {
    let a = AUDIT.lock();
    let total = if a.filled { a.buf.len() } else { a.idx };
    let mut n = 0usize;
    while n < total {
        let pos = if a.idx == 0 { total - 1 - n } else { (a.idx + a.buf.len() - 1 - n) % a.buf.len() };
        let e = a.buf[pos];
        unsafe {
            crate::uart_print(b"[LLM][AUD] op=");
            crate::shell::print_number_simple(e.op as u64);
            crate::uart_print(b" prompt_len=");
            crate::shell::print_number_simple(e.prompt_len as u64);
            crate::uart_print(b" tokens=");
            crate::shell::print_number_simple(e.tokens as u64);
            crate::uart_print(b" wcet_cycles=");
            crate::shell::print_number_simple(e.wcet_cycles as u64);
            crate::uart_print(b" period_ns=");
            crate::shell::print_number_simple(e.period_ns as u64);
            crate::uart_print(b" status=");
            crate::shell::print_number_simple(e.status as u64);
            crate::uart_print(b" ts_ns=");
            crate::shell::print_number_simple(e.ts_ns as u64);
            crate::uart_print(b"\n");
        }
        n += 1;
    }
}

/// Print audit log as JSON array to UART
pub fn audit_print_json() {
    let a = AUDIT.lock();
    let total = if a.filled { a.buf.len() } else { a.idx };
    unsafe { crate::uart_print(b"["); }
    let mut first = true;
    let mut n = 0usize;
    while n < total {
        let pos = if a.idx == 0 { total - 1 - n } else { (a.idx + a.buf.len() - 1 - n) % a.buf.len() };
        let e = a.buf[pos];
        if !first { unsafe { crate::uart_print(b", "); } } else { first = false; }
        // Print object
        unsafe {
            crate::uart_print(b"{\"op\":"); crate::shell::print_number_simple(e.op as u64);
            crate::uart_print(b",\"prompt_len\":"); crate::shell::print_number_simple(e.prompt_len as u64);
            crate::uart_print(b",\"tokens\":"); crate::shell::print_number_simple(e.tokens as u64);
            crate::uart_print(b",\"wcet_cycles\":"); crate::shell::print_number_simple(e.wcet_cycles as u64);
            crate::uart_print(b",\"period_ns\":"); crate::shell::print_number_simple(e.period_ns as u64);
            crate::uart_print(b",\"status\":"); crate::shell::print_number_simple(e.status as u64);
            crate::uart_print(b",\"ts_ns\":"); crate::shell::print_number_simple(e.ts_ns as u64);
            crate::uart_print(b"}");
        }
        n += 1;
    }
    unsafe { crate::uart_print(b"]\n"); }
}

/// Very small, deterministic signature stub for model packages.
/// This is NOT cryptographic — it is a placeholder to exercise the control path and audit.
fn verify_signature_stub(model_id: u32, sig: u64) -> bool {
    // Expected signature: XOR of two fixed salts with model_id in the low bits.
    // Chosen to be stable and obviously non-cryptographic.
    const SALT_A: u64 = 0xA5A5_A5A5_A5A5_A5A5;
    const SALT_B: u64 = 0x5349_534C_4D4F_444C; // b"SISLMODL"
    let expected = SALT_A ^ SALT_B ^ (model_id as u64);
    sig == expected
}

/// Supported quantization schemes (placeholder)
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Quantization { Q4_0, Q4_1, Int8, FP16, FP32 }

/// Basic model packaging metadata (placeholder)
#[derive(Clone)]
pub struct ModelMeta {
    pub id: u32,
    pub name: Option<String>,
    pub ctx_len: u32,
    pub vocab_size: u32,
    pub quant: Quantization,
    pub revision: Option<u32>,
    pub size_bytes: usize,
}

fn validate_model_meta(meta: &ModelMeta) -> bool {
    // Policy bounds (tunable):
    const MAX_CTX: u32 = 8192;
    const MIN_CTX: u32 = 1;
    const MIN_VOCAB: u32 = 1000;
    const MAX_VOCAB: u32 = 500_000;
    const MAX_SIZE: usize = 64 * 1024 * 1024; // 64 MiB placeholder cap

    if meta.ctx_len < MIN_CTX || meta.ctx_len > MAX_CTX { return false; }
    if meta.vocab_size < MIN_VOCAB || meta.vocab_size > MAX_VOCAB { return false; }
    if meta.size_bytes > MAX_SIZE { return false; }
    true
}

/// Load/activate a model package (skeleton). Returns true on success.
pub fn load_model(wcet_cycles: Option<u64>) -> bool {
    let mut st = STATE.lock();
    if let Some(c) = wcet_cycles { st.cfg.wcet_cycles = c; }
    // In a later phase, verify signatures and reserve arenas
    audit(1, 0, 0, st.cfg.wcet_cycles, st.cfg.period_ns, 0b001);
    true
}

/// Load model with optional metadata and signature verification (stubbed).
/// On signature failure, audits reject and returns false.
pub fn load_model_meta(model_id: Option<u32>, wcet_cycles: Option<u64>, signature: Option<u64>) -> bool {
    let mut st = STATE.lock();
    if let Some(c) = wcet_cycles { st.cfg.wcet_cycles = c; }
    if let (Some(id), Some(sig)) = (model_id, signature) {
        if !verify_signature_stub(id, sig) {
            // audit reject (op=1)
            audit(1, 0, 0, st.cfg.wcet_cycles, st.cfg.period_ns, 0b010);
            return false;
        }
    }
    audit(1, 0, 0, st.cfg.wcet_cycles, st.cfg.period_ns, 0b001);
    true
}

/// Load model with richer metadata and signature, enforce basic policy caps.
pub fn load_model_with_meta(meta: Option<ModelMeta>, wcet_cycles: Option<u64>, signature: Option<u64>) -> bool {
    let mut st = STATE.lock();
    if let Some(c) = wcet_cycles { st.cfg.wcet_cycles = c; }
    if let Some(ref m) = meta {
        // If a signature was provided, verify it against model id
        if let Some(sig) = signature {
            if !verify_signature_stub(m.id, sig) {
                audit(1, 0, 0, st.cfg.wcet_cycles, st.cfg.period_ns, 0b010);
                return false;
            }
        }
        // Enforce basic packaging caps
        if !validate_model_meta(m) {
            audit(1, 0, 0, st.cfg.wcet_cycles, st.cfg.period_ns, 0b010);
            return false;
        }
        st.current_model = Some(m.clone());
        audit(1, 0, 0, st.cfg.wcet_cycles, st.cfg.period_ns, 0b001);
        return true;
    }
    // Fallback to minimal path
    audit(1, 0, 0, st.cfg.wcet_cycles, st.cfg.period_ns, 0b001);
    true
}

/// Inference result summary
pub struct LlmResult {
    pub infer_id: usize,
    pub tokens_emitted: usize,
    pub output: String,
    pub latency_us: usize,
}

/// Submit a prompt for inference (stubbed). Bounded CPU work per token.
pub fn infer(prompt: &str, max_tokens: Option<usize>) -> LlmResult {
    let id = INFER_ID.fetch_add(1, Ordering::Relaxed);
    let mut st = STATE.lock();
    st.queue_depth = st.queue_depth.saturating_add(1);
    if st.queue_depth > st.queue_depth_max { st.queue_depth_max = st.queue_depth; }

    // Period window maintenance and quota check
    let now_ns = crate::graph::cycles_to_ns(crate::graph::now_cycles());
    if st.cfg.period_ns > 0 {
        if st.period_window_start_ns == 0 || now_ns.saturating_sub(st.period_window_start_ns) >= st.cfg.period_ns {
            st.period_window_start_ns = now_ns;
            st.period_tokens_issued = 0;
        }
        let req = max_tokens.unwrap_or(st.cfg.default_max_tokens);
        if st.cfg.max_tokens_per_period > 0 && st.period_tokens_issued.saturating_add(req) > st.cfg.max_tokens_per_period {
            st.rejects = st.rejects.saturating_add(1);
            metric_kv("llm_rejects", st.rejects);
            st.queue_depth = st.queue_depth.saturating_sub(1);
            audit(3, prompt.len(), 0, st.cfg.wcet_cycles, st.cfg.period_ns, 0b010);
            return LlmResult { infer_id: id, tokens_emitted: 0, output: String::new(), latency_us: 0 };
        }
    }

    // Start timing
    let t0 = crate::graph::now_cycles();

    // Very small, deterministic generation: split words, cap by max_tokens
    let cap = max_tokens.unwrap_or(st.cfg.default_max_tokens);
    let mut out = String::new();
    let mut tokens = 0usize;
    let bytes = prompt.as_bytes();
    let mut i = 0;
    // Capture tokens for control-plane polling
    let mut captured: heapless::Vec<heapless::String<32>, 128> = heapless::Vec::new();
    while i < bytes.len() && tokens < cap {
        // find next space (simple token)
        let mut j = i;
        while j < bytes.len() && bytes[j] > b' ' { j += 1; }
        if j > i {
            // echo token back with a simple transformation (deterministic)
            // to prove end-to-end flow without heavy compute
            if !out.is_empty() { out.push(' '); }
            out.push_str("⟨");
            for &b in &bytes[i..j] {
                // map ASCII letters to lowercase; keep other bytes as '?'
                let c = if (b'A'..=b'Z').contains(&b) { (b as u8 + 32) as char }
                        else if (b'a'..=b'z').contains(&b) || (b'0'..=b'9').contains(&b) { b as char }
                        else { '?' };
                out.push(c);
            }
            out.push_str("⟩");
            tokens += 1;
            // capture ascii-safe token for polling
            let mut t = heapless::String::<32>::new();
            for &b in &bytes[i..j] {
                let c = if (b'A'..=b'Z').contains(&b) { (b as u8 + 32) as char }
                        else if (b'a'..=b'z').contains(&b) || (b'0'..=b'9').contains(&b) { b as char }
                        else { '?' };
                let _ = t.push(c);
            }
            let _ = captured.push(t);
        }
        // skip spaces
        while j < bytes.len() && bytes[j] <= b' ' { j += 1; }
        i = j;
    }

    let t1 = crate::graph::now_cycles();
    let ns = crate::graph::cycles_to_ns(t1.saturating_sub(t0));
    let us = (ns / 1000) as usize;

    // Deadline check against wcet_cycles (converted to ns)
    let mut frq: u64; unsafe { core::arch::asm!("mrs {x}, cntfrq_el0", x = out(reg) frq); }
    if frq > 0 {
        let wcet_ns = (st.cfg.wcet_cycles.saturating_mul(1_000_000_000)) / frq;
        if ns > wcet_ns { st.deadline_miss_count = st.deadline_miss_count.saturating_add(1); }
    }

    // Update counters and emit metrics
    st.total_tokens = st.total_tokens.saturating_add(tokens);
    st.last_latency_us = us;
    metric_kv("llm_infer_us", us);
    metric_kv("llm_tokens_out", tokens);
    metric_kv("llm_queue_depth_max", st.queue_depth_max);
    metric_kv("llm_deadline_miss_count", st.deadline_miss_count);
    metric_kv("llm_rejects", st.rejects);

    // Done with this request
    st.queue_depth = st.queue_depth.saturating_sub(1);
    if st.cfg.period_ns > 0 { st.period_tokens_issued = st.period_tokens_issued.saturating_add(tokens); }

    // Deterministic scheduler accounting (optional)
    #[cfg(feature = "deterministic")]
    {
        let expected_ns = if frq > 0 { (st.cfg.wcet_cycles.saturating_mul(1_000_000_000)) / frq } else { ns };
        crate::deterministic::llm_on_infer_complete(ns, expected_ns);
    }

    // Audit ok/deadline status
    let status = 0b001 | if frq > 0 { let wcet_ns = (st.cfg.wcet_cycles.saturating_mul(1_000_000_000))/frq; if ns > wcet_ns { 0b100 } else { 0 } } else { 0 };
    let wcet_cycles = st.cfg.wcet_cycles;
    let period_ns = st.cfg.period_ns;
    let cur_model_id = st.current_model.as_ref().map(|m| m.id);
    // Release lock before recording to avoid deadlock
    drop(st);
    audit(3, prompt.len(), tokens, wcet_cycles, period_ns, status);

    // Publish state (last + table) without locking STATE again
    record_infer_state(id, captured, cur_model_id, prompt.len());

    LlmResult { infer_id: id, tokens_emitted: tokens, output: out, latency_us: us }
}

/// Return a snapshot of stats for shell printing
pub fn stats() -> (usize, usize, usize, usize) {
    let st = STATE.lock();
    (st.queue_depth_max, st.total_tokens, st.deadline_miss_count, st.last_latency_us)
}

/// Configure LLM budgets (wcet_cycles, period_ns, max_tokens_per_period)
pub fn configure_budget(wcet_cycles: Option<u64>, period_ns: Option<u64>, max_tokens_per_period: Option<usize>) {
    let mut st = STATE.lock();
    if let Some(w) = wcet_cycles { st.cfg.wcet_cycles = w; }
    if let Some(p) = period_ns { st.cfg.period_ns = p; st.period_window_start_ns = 0; st.period_tokens_issued = 0; }
    if let Some(m) = max_tokens_per_period { st.cfg.max_tokens_per_period = m; }
    #[cfg(feature = "deterministic")]
    {
        // Try to configure LLM server in the scheduler using provided budgets
        let mut frq: u64; unsafe { core::arch::asm!("mrs {x}, cntfrq_el0", x = out(reg) frq); }
        if frq > 0 {
            let wcet = st.cfg.wcet_cycles.saturating_mul(1_000_000_000) / frq;
            let deadline = if st.cfg.period_ns > 0 { st.cfg.period_ns } else { wcet };
            let _ = crate::deterministic::llm_configure_server(wcet, st.cfg.period_ns, deadline);
        }
    }
}

/// Streaming inference: emits token chunks and prints partial output.
/// Returns final result summary.
pub fn infer_stream(prompt: &str, max_tokens: Option<usize>, chunk_tokens: usize) -> LlmResult {
    let id = INFER_ID.fetch_add(1, Ordering::Relaxed);
    let mut st = STATE.lock();
    st.queue_depth = st.queue_depth.saturating_add(1);
    if st.queue_depth > st.queue_depth_max { st.queue_depth_max = st.queue_depth; }

    let now_ns = crate::graph::cycles_to_ns(crate::graph::now_cycles());
    if st.cfg.period_ns > 0 {
        if st.period_window_start_ns == 0 || now_ns.saturating_sub(st.period_window_start_ns) >= st.cfg.period_ns {
            st.period_window_start_ns = now_ns;
            st.period_tokens_issued = 0;
        }
    }

    // Start timing
    let t0 = crate::graph::now_cycles();

    let cap = max_tokens.unwrap_or(st.cfg.default_max_tokens);
    let bytes = prompt.as_bytes();
    let mut i = 0usize;
    let mut tokens = 0usize;
    let mut out = String::new();
    let mut chunks = 0usize;
    let mut chunk_buf = String::new();
    // Capture tokens for control-plane polling and session tracking
    let mut captured: heapless::Vec<heapless::String<32>, 128> = heapless::Vec::new();

    while i < bytes.len() && tokens < cap {
        let mut j = i;
        while j < bytes.len() && bytes[j] > b' ' { j += 1; }
        if j > i {
            // quota check per token (period)
            if st.cfg.period_ns > 0 && st.cfg.max_tokens_per_period > 0 {
                if st.period_tokens_issued.saturating_add(1) > st.cfg.max_tokens_per_period {
                    st.rejects = st.rejects.saturating_add(1);
                    break;
                }
                st.period_tokens_issued += 1;
            }
            // build token
            if !out.is_empty() { out.push(' '); }
            if !chunk_buf.is_empty() { chunk_buf.push(' '); }
            out.push_str("⟨"); chunk_buf.push_str("⟨");
            let mut tok_s = heapless::String::<32>::new();
            for &b in &bytes[i..j] {
                let c = if (b'A'..=b'Z').contains(&b) { (b as u8 + 32) as char }
                        else if (b'a'..=b'z').contains(&b) || (b'0'..=b'9').contains(&b) { b as char }
                        else { '?' };
                out.push(c);
                chunk_buf.push(c);
                let _ = tok_s.push(c);
            }
            let _ = captured.push(tok_s);
            out.push_str("⟩"); chunk_buf.push_str("⟩");
            tokens += 1;

            // flush chunk
            if tokens % chunk_tokens == 0 || tokens == cap {
                chunks += 1;
                unsafe {
                    crate::uart_print(b"[LLM][STREAM] chunk: ");
                    crate::uart_print(chunk_buf.as_bytes());
                    crate::uart_print(b"\n");
                }
                metric_kv("llm_stream_chunk_tokens", chunk_tokens.min(tokens));
                chunk_buf.clear();
            }
        }
        while j < bytes.len() && bytes[j] <= b' ' { j += 1; }
        i = j;
        if tokens >= cap { break; }
    }

    let t1 = crate::graph::now_cycles();
    let ns = crate::graph::cycles_to_ns(t1.saturating_sub(t0));
    let us = (ns / 1000) as usize;

    st.total_tokens = st.total_tokens.saturating_add(tokens);
    st.last_latency_us = us;
    metric_kv("llm_infer_us", us);
    metric_kv("llm_tokens_out", tokens);
    metric_kv("llm_queue_depth_max", st.queue_depth_max);
    metric_kv("llm_deadline_miss_count", st.deadline_miss_count);
    metric_kv("llm_rejects", st.rejects);
    metric_kv("llm_stream_chunks", chunks);

    st.queue_depth = st.queue_depth.saturating_sub(1);

    #[cfg(feature = "deterministic")]
    {
        let mut frq: u64; unsafe { core::arch::asm!("mrs {x}, cntfrq_el0", x = out(reg) frq); }
        let expected_ns = if frq > 0 { (st.cfg.wcet_cycles.saturating_mul(1_000_000_000)) / frq } else { ns };
        crate::deterministic::llm_on_infer_complete(ns, expected_ns);
    }

    audit(4, prompt.len(), tokens, st.cfg.wcet_cycles, st.cfg.period_ns, 0b001);

    // Publish state so streamed sessions are visible to ctl_poll/llmsummary
    let cur_model_id = st.current_model.as_ref().map(|m| m.id);
    drop(st);
    record_infer_state(id, captured, cur_model_id, prompt.len());

    LlmResult { infer_id: id, tokens_emitted: tokens, output: out, latency_us: us }
}

/// Control-plane: poll available tokens from last inference.
/// Returns (infer_id, emitted_count, done_flag).
pub fn ctl_poll(max: usize) -> (usize, usize, bool, heapless::String<128>) {
    // Poll most recent infer from table if present
    let id_opt;
    {
        let tab = INFER_TABLE.lock();
        id_opt = tab.last().map(|s| s.infer_id);
    }
    if let Some(id) = id_opt { return ctl_poll_id(id, max); }
    // Fallback to last-infer only
    let mut li = LAST_INFER.lock();
    let id0 = li.infer_id;
    let mut emitted = 0usize;
    let mut items = heapless::String::<128>::new();
    while li.read_idx < li.tokens.len() && emitted < max {
        if emitted > 0 { let _ = items.push('|'); }
        if let Some(tok) = li.tokens.get(li.read_idx) {
            for ch in tok.chars() { if items.len() >= 120 { break; } let _ = items.push(ch); }
        }
        li.read_idx += 1; emitted += 1;
    }
    (id0, emitted, li.read_idx >= li.tokens.len(), items)
}

/// Poll by specific infer id (currently aliases last-infer; returns empty if id mismatches)
pub fn ctl_poll_id(infer_id: usize, max: usize) -> (usize, usize, bool, heapless::String<128>) {
    // Search table for id
    let idx_opt;
    {
        let tab = INFER_TABLE.lock();
        idx_opt = tab.iter().position(|s| s.infer_id == infer_id);
    }
    if let Some(pos) = idx_opt {
        let mut tab = INFER_TABLE.lock();
        let st = &mut tab[pos];
        let mut emitted = 0usize;
        let mut items = heapless::String::<128>::new();
        while st.read_idx < st.tokens.len() && emitted < max {
            if emitted > 0 { let _ = items.push('|'); }
            if let Some(tok) = st.tokens.get(st.read_idx) {
                for ch in tok.chars() { if items.len() >= 120 { break; } let _ = items.push(ch); }
            }
            st.read_idx += 1; emitted += 1;
        }
        return (infer_id, emitted, st.read_idx >= st.tokens.len(), items);
    }
    // fallback to last-infer if matches
    let li = LAST_INFER.lock();
    if li.infer_id == infer_id {
        drop(li);
        return ctl_poll(max);
    }
    (infer_id, 0, true, heapless::String::new())
}

/// Control-plane: cancel current inference (stub for compatibility).
pub fn ctl_cancel() {
    let mut li = LAST_INFER.lock();
    li.done = true;
}

pub fn ctl_cancel_id(infer_id: usize) {
    // Mark done in table if present, else mark last-infer
    let mut tab = INFER_TABLE.lock();
    if let Some(st) = tab.iter_mut().find(|s| s.infer_id == infer_id) { st.done = true; return; }
    drop(tab);
    let mut li = LAST_INFER.lock();
    if li.infer_id == infer_id { li.done = true; }
}

/// Print a summary of recent inference sessions (id, tokens, consumed, done, ts, model)
pub fn ctl_print_sessions() {
    let tab = INFER_TABLE.lock();
    unsafe { crate::uart_print(b"[LLM][SESSIONS] count="); }
    crate::shell::print_number_simple(tab.len() as u64);
    unsafe { crate::uart_print(b"\n"); }
    for s in tab.iter() {
        unsafe { crate::uart_print(b"[LLM][SESS] id="); }
        crate::shell::print_number_simple(s.infer_id as u64);
        unsafe { crate::uart_print(b" tokens="); }
        crate::shell::print_number_simple(s.tokens.len() as u64);
        unsafe { crate::uart_print(b" consumed="); }
        crate::shell::print_number_simple(s.read_idx as u64);
        unsafe { crate::uart_print(b" done="); }
        crate::shell::print_number_simple(s.done as u64);
        unsafe { crate::uart_print(b" ts_ns="); }
        crate::shell::print_number_simple(s.ts_ns as u64);
        unsafe { crate::uart_print(b" model="); }
        match s.model_id {
            Some(mid) => crate::shell::print_number_simple(mid as u64),
            None => unsafe { crate::uart_print(b"none"); },
        }
        unsafe { crate::uart_print(b"\n"); }
    }
}

/// Peek metadata for a given infer id: (prompt_len, model_id)
pub fn ctl_peek_meta(infer_id: usize) -> (usize, Option<u32>) {
    let tab = INFER_TABLE.lock();
    if let Some(st) = tab.iter().find(|s| s.infer_id == infer_id) {
        return (st.prompt_len, st.model_id);
    }
    (0, None)
}

// --- Demo model verification (uses simplified SHA-256 + Ed25519 stubs in model.rs) ---

fn demo_sha256_like(data: &[u8]) -> [u8; 32] {
    // Must match model::ModelSecurityManager::sha256_hash
    let mut hash = [0u8; 32];
    let mut checksum: u64 = 0;
    for b in data { checksum = checksum.wrapping_add(*b as u64); checksum = checksum.wrapping_mul(31); }
    for i in 0..4 {
        let bytes = (checksum.wrapping_add(i as u64 * 1000)).to_le_bytes();
        hash[i*8..(i+1)*8].copy_from_slice(&bytes);
    }
    hash
}

pub fn verify_demo_model() -> bool {
    let (mut pkg, data) = model::create_demo_model();
    // Compute hash to satisfy simplified verification
    pkg.sha256_hash = demo_sha256_like(&data);
    let mut sec = MODEL_SEC.lock();
    let ts0 = crate::graph::now_cycles();
    let ok = sec.load_model(pkg, &data).is_ok();
    let ts1 = crate::graph::now_cycles();
    let us = (crate::graph::cycles_to_ns(ts1.saturating_sub(ts0)) / 1000) as usize;
    metric_kv("model_verify_us", us);
    if ok { metric_kv("model_load_success", 1); } else { metric_kv("model_load_fail", 1); }
    ok
}

/// Load a model package from user-provided parameters (demo cryptography path).
/// - model_id: numeric id
/// - hash: expected simplified SHA-256-like hash (32 bytes)
/// - sig: Ed25519 signature bytes (64 bytes; demo verifies always true)
/// - size_bytes: size of the model data buffer (pattern-filled)
pub fn load_model_package(model_id: u32, hash: [u8; 32], sig: [u8; 64], size_bytes: usize) -> bool {
    // Construct deterministic data buffer (pattern based on model_id)
    let mut data = alloc::vec::Vec::with_capacity(size_bytes);
    let byte = (model_id & 0xFF) as u8;
    data.resize(size_bytes, byte);

    // Compute expected hash and compare with provided
    let computed = demo_sha256_like(&data);
    if computed != hash {
        // Hash mismatch => audit reject
        audit(1, 0, 0, STATE.lock().cfg.wcet_cycles, STATE.lock().cfg.period_ns, 0b010);
        metric_kv("model_hash_mismatch", 1);
        return false;
    }

    let pkg = model::ModelPackage {
        id: model_id,
        version: 1,
        size_bytes: size_bytes as u32,
        sha256_hash: hash,
        ed25519_signature: sig,
        permissions: model::ModelPermissions::LOAD | model::ModelPermissions::EXECUTE,
    };

    let mut sec = MODEL_SEC.lock();
    let ts0 = crate::graph::now_cycles();
    let ok = sec.load_model(pkg, &data).is_ok();
    let ts1 = crate::graph::now_cycles();
    let us = (crate::graph::cycles_to_ns(ts1.saturating_sub(ts0)) / 1000) as usize;
    metric_kv("model_verify_us", us);
    if ok {
        metric_kv("model_load_success", 1);
    } else {
        metric_kv("model_load_fail", 1);
    }
    ok
}

/// Compute the demo SHA-256-like hash for a deterministic buffer of given size and model id.
pub fn demo_hash_for(model_id: u32, size_bytes: usize) -> [u8; 32] {
    let mut data = alloc::vec::Vec::with_capacity(size_bytes);
    let byte = (model_id & 0xFF) as u8;
    data.resize(size_bytes, byte);
    demo_sha256_like(&data)
}
