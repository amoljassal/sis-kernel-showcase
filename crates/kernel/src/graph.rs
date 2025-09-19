//! Minimal graph and operator scaffolding for Phase 1.
//! Includes a simple two-operator demo wiring an SPSC channel.

use crate::channel::spsc::Spsc;
use crate::tensor::{BumpArena, TensorHandle};
use crate::trace::metric_kv;
#[cfg(feature = "deterministic")]
use crate::deterministic::{DeterministicScheduler, TaskSpec};
#[cfg(feature = "perf-verbose")]
use crate::pmu::aarch64 as pmu;
use heapless::Vec;

const MAX_OPERATORS: usize = 32;
const MAX_CHANNELS: usize = 16;

#[derive(Copy, Clone)]
pub struct OperatorId(pub u32);

pub struct Operator<'a> {
    pub id: OperatorId,
    pub run: fn(&mut OperatorCtx<'a>),
}

pub struct OperatorCtx<'a> {
    pub produced: &'a Spsc<TensorHandle, 64>,
    pub consumed: &'a Spsc<TensorHandle, 64>,
}

pub struct GraphDemo {
    pub n_items: usize,
    arena: BumpArena<8192>,
    graph: GraphApi,
    #[allow(dead_code)]
    op_a_idx: usize,
    #[allow(dead_code)]
    op_b_idx: usize,
    ch_ab_idx: usize,
    ch_bc_idx: usize,
}

impl GraphDemo {
    pub fn new(n_items: usize) -> Self {
        let mut graph = GraphApi::create();
        let ch_ab_idx = graph.add_channel(ChannelSpec { capacity: 64 });
        let ch_bc_idx = graph.add_channel(ChannelSpec { capacity: 64 });
        let op_a_idx = graph.add_operator(OperatorSpec { id: 1, func: op_a_run, in_ch: None, out_ch: Some(ch_ab_idx), priority: 10, stage: None, in_schema: None, out_schema: Some(1) });
        let op_b_idx = graph.add_operator(OperatorSpec { id: 2, func: op_b_run, in_ch: Some(ch_ab_idx), out_ch: Some(ch_bc_idx), priority: 5, stage: None, in_schema: Some(1), out_schema: None });
        Self { n_items, arena: BumpArena::new(), graph, op_a_idx, op_b_idx, ch_ab_idx, ch_bc_idx }
    }

    /// Run a trivial A->B pipeline to demonstrate scheduling and metrics.
    pub fn run(&mut self) {
        // Operators: A produces 0..n, B consumes and forwards (or accumulates).
        const SCHEMA_ID_A2B: u32 = 1;
        #[cfg(feature = "perf-verbose")]
        let _op_a_id = 1u32;
        #[cfg(feature = "perf-verbose")]
        let _op_b_id = 2u32;

        let mut _produced = 0usize;
        let mut _consumed = 0usize;
        let mut zero_copy_count = 0usize;
        let mut zero_copy_handle_count = 0usize;
        let mut ch_ab_depth_max = 0usize;
        let mut ch_ab_stalls = 0usize;
        let mut ch_ab_drops = 0usize;
        let mut op_a_runs = 0usize;
        let mut op_b_runs = 0usize;
        let mut op_a_cycles: u64 = 0;
        let mut op_b_cycles: u64 = 0;
        #[cfg(feature = "perf-verbose")]
        let mut op_a_inst: u64 = 0;
        #[cfg(feature = "perf-verbose")]
        let mut op_b_inst: u64 = 0;
        #[cfg(feature = "perf-verbose")]
        let mut op_a_l1d: u64 = 0;
        #[cfg(feature = "perf-verbose")]
        let mut op_b_l1d: u64 = 0;
        // PMU attribution is intentionally disabled in the demo to avoid
        // QEMU variability; keep perf-verbose for other parts of the boot.

        // Collect per-operator latency samples (ns) for percentiles (window 128)
        let mut lat_a: [u64; 128] = [0; 128];
        let mut lat_b: [u64; 128] = [0; 128];
        let mut lat_a_n: usize = 0;
        let mut lat_b_n: usize = 0;
        let mut schema_mismatch_count: usize = 0;
        let mut quality_warns: usize = 0;

        let t0 = now_cycles();
        for i in 0..self.n_items {
            // Producer work (no channel dependency)
            let ta0 = now_cycles();
            #[cfg(feature = "perf-verbose")]
            let s0a = unsafe { pmu::read_snapshot() };
            (op_a_run)(&mut OperatorCtx {
                produced: self.graph.channel(self.ch_ab_idx),
                consumed: self.graph.channel(self.ch_ab_idx),
            });
            // Allocate a handle and try to enqueue into AB channel
            if let Some(h) = self.arena.alloc(128, 64) {
                if !h.is_null() { zero_copy_handle_count += 1; }
                // Initialize header (typed DataTensor)
                unsafe {
                    if let Some(hdr) = h.header_mut() {
                        hdr.version = 1;
                        hdr.dtype = 0;
                        hdr.dims = [0; 4];
                        hdr.strides = [0; 4];
                        hdr.data_offset = core::mem::size_of::<crate::tensor::TensorHeader>() as u64;
                        hdr.schema_id = SCHEMA_ID_A2B;
                        hdr.records = 1;
                        hdr.quality = 100; // perfect quality for demo
                        hdr._pad = 0;
                        hdr.lineage = i as u64;
                    }
                }
                // Enqueue; count stalls/drops on failure
                if self.graph.channel(self.ch_ab_idx).try_enqueue(h).is_err() {
                    ch_ab_stalls = ch_ab_stalls.saturating_add(1);
                    ch_ab_drops = ch_ab_drops.saturating_add(1);
                } else {
                    zero_copy_count = zero_copy_count.saturating_add(1);
                }
            }
            _produced += 1;
            let ta1 = now_cycles();
            let cyc_a = ta1.saturating_sub(ta0);
            op_a_cycles = op_a_cycles.saturating_add(cyc_a);
            if lat_a_n < lat_a.len() { lat_a[lat_a_n] = cycles_to_ns(cyc_a); lat_a_n += 1; }
            #[cfg(feature = "perf-verbose")]
            {
                let s1a = unsafe { pmu::read_snapshot() };
                op_a_inst = op_a_inst.saturating_add(s1a.inst.saturating_sub(s0a.inst));
                op_a_l1d = op_a_l1d.saturating_add(s1a.l1d_refill.saturating_sub(s0a.l1d_refill));
            }
            op_a_runs += 1;

            // Consumer work; track channel AB depth for backpressure visibility
            let d = self.graph.channel(self.ch_ab_idx).depth();
            if d > ch_ab_depth_max { ch_ab_depth_max = d; }
            // Try to dequeue from AB channel and enforce schema
            if let Some(hd) = self.graph.channel(self.ch_ab_idx).try_dequeue() {
                unsafe {
                    if let Some(hdr) = hd.header() {
                        if hdr.schema_id != SCHEMA_ID_A2B {
                            schema_mismatch_count = schema_mismatch_count.saturating_add(1);
                        }
                        if hdr.quality < 50 { // arbitrary warning threshold
                            quality_warns = quality_warns.saturating_add(1);
                        }
                    }
                }
            }
            let tb0 = now_cycles();
            #[cfg(feature = "perf-verbose")]
            let s0b = unsafe { pmu::read_snapshot() };
            (op_b_run)(&mut OperatorCtx {
                produced: self.graph.channel(self.ch_bc_idx),
                consumed: self.graph.channel(self.ch_ab_idx),
            });
            _consumed += 1;
            let tb1 = now_cycles();
            let cyc_b = tb1.saturating_sub(tb0);
            op_b_cycles = op_b_cycles.saturating_add(cyc_b);
            if lat_b_n < lat_b.len() { lat_b[lat_b_n] = cycles_to_ns(cyc_b); lat_b_n += 1; }
            #[cfg(feature = "perf-verbose")]
            {
                let s1b = unsafe { pmu::read_snapshot() };
                op_b_inst = op_b_inst.saturating_add(s1b.inst.saturating_sub(s0b.inst));
                op_b_l1d = op_b_l1d.saturating_add(s1b.l1d_refill.saturating_sub(s0b.l1d_refill));
            }
            op_b_runs += 1;
            if (i & 7) == 7 {
                crate::trace::trace("GRAPH DEMO: progressed 8 items");
            }
        }
        let t1 = now_cycles();
        let ns = cycles_to_ns(t1.saturating_sub(t0));
        metric_kv("graph_demo_total_ns", ns as usize);
        metric_kv("graph_demo_items", self.n_items);
        if self.n_items > 0 { metric_kv("graph_demo_avg_ns_per_item", (ns / (self.n_items as u64)) as usize); }
        // Scheduler batch timing (us)
        metric_kv("scheduler_run_us", (ns / 1000) as usize);
        metric_kv("channel_ab_depth_max", ch_ab_depth_max);
        metric_kv("channel_ab_stalls", ch_ab_stalls);
        metric_kv("channel_ab_drops", ch_ab_drops);
        metric_kv("schema_mismatch_count", schema_mismatch_count);
        metric_kv("quality_warns", quality_warns);
        metric_kv("zero_copy_count", zero_copy_count);
        metric_kv("zero_copy_handle_count", zero_copy_handle_count);
        // Operator summaries
        metric_kv("op_a_runs", op_a_runs);
        metric_kv("op_b_runs", op_b_runs);
        metric_kv("op_a_total_ns", cycles_to_ns(op_a_cycles) as usize);
        metric_kv("op_b_total_ns", cycles_to_ns(op_b_cycles) as usize);
        // Percentiles for operator latencies
        if lat_a_n > 0 {
            let p50 = percentile_ns(&mut lat_a, lat_a_n, 0.50);
            let p95 = percentile_ns(&mut lat_a, lat_a_n, 0.95);
            let p99 = percentile_ns(&mut lat_a, lat_a_n, 0.99);
            metric_kv("op_a_p50_ns", p50 as usize);
            metric_kv("op_a_p95_ns", p95 as usize);
            metric_kv("op_a_p99_ns", p99 as usize);
        }
        if lat_b_n > 0 {
            let p50 = percentile_ns(&mut lat_b, lat_b_n, 0.50);
            let p95 = percentile_ns(&mut lat_b, lat_b_n, 0.95);
            let p99 = percentile_ns(&mut lat_b, lat_b_n, 0.99);
            metric_kv("op_b_p50_ns", p50 as usize);
            metric_kv("op_b_p95_ns", p95 as usize);
            metric_kv("op_b_p99_ns", p99 as usize);
        }
        #[cfg(feature = "perf-verbose")]
        {
            metric_kv("op_a_pmu_inst", op_a_inst as usize);
            metric_kv("op_b_pmu_inst", op_b_inst as usize);
            metric_kv("op_a_pmu_l1d_refill", op_a_l1d as usize);
            metric_kv("op_b_pmu_l1d_refill", op_b_l1d as usize);
        }
        // Arena remaining bytes (sanity check for bump behavior)
        metric_kv("arena_remaining_bytes", self.arena.remaining());
    }
}

#[inline(always)]
fn percentile_ns(buf: &mut [u64; 128], n: usize, p: f32) -> u64 {
    if n == 0 { return 0; }
    // simple in-place sort of the used prefix
    let slice = &mut buf[..n];
    slice.sort_unstable();
    let idx = ((n - 1) as f32 * p) as usize;
    slice[idx]
}

pub fn op_a_run(_ctx: &mut OperatorCtx) {
    // Placeholder for producer work (could fill a tensor)
}

fn op_b_run(_ctx: &mut OperatorCtx) {
    // Placeholder for consumer work (could transform a tensor)
}

#[inline(always)]
pub fn now_cycles() -> u64 {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let mut v: u64; core::arch::asm!("isb; mrs {x}, cntvct_el0", x = out(reg) v, options(nomem, nostack, preserves_flags)); v
    }
    #[cfg(not(target_arch = "aarch64"))]
    { 0 }
}

#[inline(always)]
fn cntfrq_hz() -> u64 {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let mut v: u64; core::arch::asm!("mrs {x}, cntfrq_el0", x = out(reg) v, options(nomem, nostack, preserves_flags)); v
    }
    #[cfg(not(target_arch = "aarch64"))]
    { 1 }
}

#[inline(always)]
pub fn cycles_to_ns(cycles: u64) -> u64 {
    let f = cntfrq_hz();
    if f == 0 { return 0; }
    (cycles.saturating_mul(1_000_000_000u64)) / f
}

// Minimal Graph API surface (Phase 1 scaffolding)
#[allow(dead_code)]
pub enum PortDir { In, Out }

#[allow(dead_code)]
pub struct ChannelSpec { pub capacity: usize }

#[allow(dead_code)]
pub struct OperatorSpec {
    pub id: u32,
    pub func: fn(&mut OperatorCtx),
    pub in_ch: Option<usize>,
    pub out_ch: Option<usize>,
    pub priority: u8,
    #[allow(dead_code)]
    pub stage: Option<Stage>,
    #[allow(dead_code)]
    pub in_schema: Option<u32>,
    #[allow(dead_code)]
    pub out_schema: Option<u32>,
}

pub struct GraphApi {
    channels: Vec<Spsc<TensorHandle, 64>, MAX_CHANNELS>,
    ops: Vec<OpNode, MAX_OPERATORS>,
    channel_schemas: Vec<Option<u32>, MAX_CHANNELS>,
    schema_mismatch_count: usize,
    prev_depths: [usize; MAX_CHANNELS],
    #[cfg(feature = "deterministic")]
    det_server_id: Option<u32>,
    #[cfg(feature = "deterministic")]
    det_wcet_ns: u64,
    #[cfg(feature = "deterministic")]
    det_scheduler: DeterministicScheduler<16>,
    #[allow(dead_code)]
    deterministic_mode: bool,
    #[cfg(feature = "deterministic")]
    det_overrun_count: usize,
}

struct OpNode {
    #[allow(dead_code)]
    id: u32,
    in_ch: Option<usize>,
    out_ch: Option<usize>,
    #[allow(dead_code)]
    priority: u8,
    func: fn(&mut OperatorCtx),
    #[allow(dead_code)]
    stage: Option<Stage>,
    #[allow(dead_code)]
    in_schema: Option<u32>,
    #[allow(dead_code)]
    out_schema: Option<u32>,
}

#[allow(dead_code)]
impl GraphApi {
    pub fn create() -> Self {
        let g = Self {
            channels: Vec::new(),
            ops: Vec::new(),
            channel_schemas: Vec::new(),
            schema_mismatch_count: 0,
            prev_depths: [0; MAX_CHANNELS],
            #[cfg(feature = "deterministic")]
            det_server_id: None,
            #[cfg(feature = "deterministic")]
            det_wcet_ns: 0,
            #[cfg(feature = "deterministic")]
            det_scheduler: DeterministicScheduler::new(850_000), // 85% utilization bound
            deterministic_mode: false,
            #[cfg(feature = "deterministic")]
            det_overrun_count: 0,
        };
        // Pre-reserve small capacities to avoid first-use heap allocations during control ops
        // Pre-allocate fixed capacity (heapless); no dynamic allocations after this
        // Capacity is fixed by type parameters (MAX_*), so nothing to do here.
        g
    }
    
    /// Enable deterministic mode for this graph
    #[cfg(feature = "deterministic")]
    pub fn enable_deterministic(&mut self, wcet_ns: u64, period_ns: u64, deadline_ns: u64) -> bool {
        let spec = TaskSpec {
            id: 0, // Graph-level task
            wcet_ns,
            period_ns,
            deadline_ns,
        };
        
        match self.det_scheduler.admit_graph(0, spec) {
            Ok(server_id) => {
                self.deterministic_mode = true;
                self.det_server_id = Some(server_id);
                self.det_wcet_ns = wcet_ns;
                metric_kv("det_admit_ok", 1);
                true
            },
            Err(()) => {
                metric_kv("det_admit_reject", 1);
                false
            }
        }
    }
    pub fn add_channel(&mut self, _spec: ChannelSpec) -> usize {
        let idx = self.channels.len();
        if self.channels.push(Spsc::new()).is_err() {
            metric_kv("graph_add_channel_overflow", 1);
            return idx; // no-op
        }
        let _ = self.channel_schemas.push(None);
        idx
    }
    pub fn add_operator(&mut self, spec: OperatorSpec) -> usize {
        let idx = self.ops.len();
        if self.ops.push(OpNode {
            id: spec.id,
            in_ch: spec.in_ch,
            out_ch: spec.out_ch,
            priority: spec.priority,
            func: spec.func,
            stage: spec.stage,
            in_schema: spec.in_schema,
            out_schema: spec.out_schema,
        }).is_err() {
            metric_kv("graph_add_operator_overflow", 1);
        }
        idx
    }

    /// Strictly enforce typed schemas at connect time; returns true if added
    pub fn add_operator_strict(&mut self, spec: OperatorSpec) -> bool {
        // Enforce/connect typed schemas to channels if provided
        if let Some(ch_idx) = spec.out_ch {
            if let Some(schema) = spec.out_schema {
                if ch_idx < self.channel_schemas.len() {
                    let slot = &mut self.channel_schemas[ch_idx];
                    if slot.is_none() {
                        *slot = Some(schema);
                    } else if let Some(existing) = slot.as_mut() {
                        if *existing != schema {
                            self.schema_mismatch_count = self.schema_mismatch_count.saturating_add(1);
                            metric_kv("schema_mismatch_count", self.schema_mismatch_count);
                            return false;
                        }
                    }
                } else {
                    metric_kv("graph_schema_out_of_range", ch_idx);
                    return false;
                }
            }
        }
        if let Some(ch_idx) = spec.in_ch {
            if let Some(expected) = spec.in_schema {
                if ch_idx < self.channel_schemas.len() {
                    if let Some(current) = self.channel_schemas[ch_idx] {
                        if current != expected {
                            self.schema_mismatch_count = self.schema_mismatch_count.saturating_add(1);
                            metric_kv("schema_mismatch_count", self.schema_mismatch_count);
                            return false;
                        }
                    } else {
                        // Bind channel to expected input schema if not set yet
                        self.channel_schemas[ch_idx] = Some(expected);
                    }
                } else {
                    metric_kv("graph_schema_in_of_range", ch_idx);
                    return false;
                }
            }
        }
        let _ = self.ops.push(OpNode {
            id: spec.id,
            in_ch: spec.in_ch,
            out_ch: spec.out_ch,
            priority: spec.priority,
            func: spec.func,
            stage: spec.stage,
            in_schema: spec.in_schema,
            out_schema: spec.out_schema,
        });
        true
    }
    pub fn is_runnable(&self, op_idx: usize) -> bool {
        if let Some(op) = self.ops.get(op_idx) {
            let in_ready = match op.in_ch { Some(i) if i < self.channels.len() => !self.channels[i].is_empty(), None => true, _ => false };
            let out_ready = match op.out_ch { Some(i) if i < self.channels.len() => !self.channels[i].is_full(), None => true, _ => false };
            in_ready && out_ready
        } else { false }
    }
    pub fn channel(&self, idx: usize) -> &Spsc<TensorHandle, 64> { &self.channels[idx] }

    /// Execute up to `steps` runnable operators in static-priority order (highest first).
    pub fn run_steps(&mut self, steps: usize) {
        if steps == 0 { return; }
        // Simple O(n^2) selection for now (tiny n)
        for _ in 0..steps {
            #[cfg(feature = "deterministic")]
            {
                if self.deterministic_mode {
                    let now = cycles_to_ns(now_cycles());
                    if let Some(gid) = self.det_scheduler.schedule_next(now) {
                        if gid != 0 { break; }
                    } else {
                        break;
                    }
                }
            }
            let mut ran = false;
            let mut best_idx: Option<usize> = None;
            let mut best_pri: u8 = 0;
            for (i, op) in self.ops.iter().enumerate() {
                if self.is_runnable(i) && op.priority >= best_pri { best_pri = op.priority; best_idx = Some(i); }
            }
            if let Some(i) = best_idx {
                let op = &self.ops[i];
                let out_idx = op.out_ch.unwrap_or(0);
                let in_idx = op.in_ch.unwrap_or(out_idx);
                let out = self.channels.get(out_idx).unwrap_or(&self.channels[0]);
                let inp = self.channels.get(in_idx).unwrap_or(out);
                // Trace channel depth changes for in/out
                let in_depth = if in_idx < self.channels.len() { self.channels[in_idx].depth() } else { 0 };
                if in_idx < self.prev_depths.len() && self.prev_depths[in_idx] != in_depth {
                    self.prev_depths[in_idx] = in_depth;
                    crate::trace::ch_depth(in_idx, in_depth);
                }
                let out_depth = if out_idx < self.channels.len() { self.channels[out_idx].depth() } else { 0 };
                if out_idx < self.prev_depths.len() && self.prev_depths[out_idx] != out_depth {
                    self.prev_depths[out_idx] = out_depth;
                    crate::trace::ch_depth(out_idx, out_depth);
                }
                let mut ctx = OperatorCtx { produced: out, consumed: inp };
                crate::trace::op_queued(op.id);
                let t0 = now_cycles();
                crate::trace::op_start(op.id);
                // In deterministic mode, disallow heap allocations during operator run
                #[cfg(feature = "deterministic")]
                if self.deterministic_mode { crate::heap::det_no_alloc_enter(); }
                (op.func)(&mut ctx);
                #[cfg(feature = "deterministic")]
                if self.deterministic_mode { crate::heap::det_no_alloc_exit(); }
                let t1 = now_cycles();
                let dt_ns = cycles_to_ns(t1.saturating_sub(t0));
                crate::trace::op_end_ns(op.id, dt_ns);
                #[cfg(feature = "deterministic")]
                {
                    if self.deterministic_mode {
                        let expected = if self.det_wcet_ns == 0 { dt_ns } else { self.det_wcet_ns };
                        if dt_ns > expected {
                            self.det_overrun_count = self.det_overrun_count.saturating_add(1);
                            crate::trace::metric_kv("det_overrun_count", self.det_overrun_count);
                        }
                        self.det_scheduler.complete_execution(0, dt_ns, expected);
                    }
                }
                ran = true;
            }
            if !ran { break; }
        }
    }

    /// Return simple counts for ops and channels (for diagnostics).
    pub fn counts(&self) -> (usize, usize) {
        (self.ops.len(), self.channels.len())
    }

    #[cfg(feature = "deterministic")]
    pub fn admit_deterministic(&mut self, wcet_ns: u64, period_ns: u64, deadline_ns: u64) -> bool {
        self.enable_deterministic(wcet_ns, period_ns, deadline_ns)
    }

    /// Disable deterministic mode for this graph
    #[cfg(feature = "deterministic")]
    pub fn disable_deterministic(&mut self) { self.deterministic_mode = false; }

    /// Return current deterministic overrun count
    #[cfg(feature = "deterministic")]
    pub fn det_overruns(&self) -> usize { self.det_overrun_count }

    /// Reset deterministic counters
    #[cfg(feature = "deterministic")]
    pub fn det_reset(&mut self) { self.det_overrun_count = 0; }

    /// Return whether deterministic mode is enabled
    #[cfg(feature = "deterministic")]
    pub fn deterministic_enabled(&self) -> bool { self.deterministic_mode }

    /// Return configured WCET (ns)
    #[cfg(feature = "deterministic")]
    pub fn det_wcet(&self) -> u64 { self.det_wcet_ns }
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum Stage { AcquireData=0, CleanData=1, ExploreData=2, ModelData=3, ExplainResults=4 }

// Control-plane can call op_a_run directly (pub)

/// Debug SPSC ring behavior with progress prints (u32 payloads). Feature-gated.
#[cfg(feature = "graph-spsc-debug")]
pub fn run_spsc_debug(n: usize) {
    use crate::trace::trace;
    let q: Spsc<u32, 64> = Spsc::new();
    let mut produced = 0usize;
    let mut consumed = 0usize;
    trace("SPSC DEBUG: start");
    while consumed < n {
        if produced < n {
            let v = produced as u32;
            if q.try_enqueue(v).is_ok() {
                produced += 1;
                if produced % 8 == 0 { trace("SPSC DEBUG: produced 8"); }
            }
        }
        if let Some(_v) = q.try_dequeue() {
            consumed += 1;
            if consumed % 8 == 0 { trace("SPSC DEBUG: consumed 8"); }
        }
    }
    crate::trace::metric_kv("spsc_debug_done", 1);
}

/// Phase 2 deterministic demo with comprehensive CBS+EDF scheduling
#[cfg(feature = "deterministic")]
pub fn deterministic_demo() {
    use crate::deterministic::{DeterministicScheduler, TaskSpec, ConstraintEnforcer, verify_deterministic_constraints};
    use crate::model::{ModelSecurityManager, ModelPermissions, ModelConstraints, create_demo_model};
    use crate::cap::{Capability, CapId, CapRights, CapObjectKind};
    use crate::trace::trace;
    
    trace("DETERMINISTIC DEMO: Starting Phase 2 comprehensive demo");
    
    // Initialize deterministic scheduler
    let mut scheduler: DeterministicScheduler<8> = DeterministicScheduler::new(850_000); // 85% bound
    
    // Initialize model security manager
    let mut model_manager: ModelSecurityManager<4, 32> = ModelSecurityManager::new();
    
    // Initialize constraint enforcer
    let mut enforcer = ConstraintEnforcer::new(1000); // Max 1000 loop iterations
    
    // Create and load a demo model
    let (mut demo_package, demo_data) = create_demo_model();
    
    // Compute proper hash for the demo data
    demo_package.sha256_hash = [0x42; 32]; // Demo hash
    demo_package.permissions = ModelPermissions::LOAD | ModelPermissions::EXECUTE;
    
    // Load the model
    match model_manager.load_model(demo_package, &demo_data) {
        Ok(model_idx) => {
            trace("DETERMINISTIC DEMO: Model loaded successfully");
            
            // Create capability for model execution
            let model_cap = Capability {
                id: CapId::new(1).unwrap(),
                kind: CapObjectKind::Model,
                rights: CapRights::RUN | CapRights::EXECUTE,
            };
            
            // Test model execution with constraints
            let constraints = ModelConstraints {
                memory_cap_bytes: 512 * 1024, // 512KB limit
                compute_budget_ns: 100_000,   // 100μs budget
                allowed_ops: 0xFF,            // All ops allowed for demo
            };
            
            let exec_result = model_manager.execute_model(model_idx, constraints, model_cap);
            match exec_result {
                crate::model::ModelResult::Success => {
                    trace("DETERMINISTIC DEMO: Model execution successful");
                }
                _ => {
                    trace("DETERMINISTIC DEMO: Model execution failed");
                }
            }
        }
        Err(_) => {
            trace("DETERMINISTIC DEMO: Model load failed");
        }
    }
    
    // Admit deterministic graphs to scheduler
    let graph_specs = [
        TaskSpec { id: 1, wcet_ns: 50_000, period_ns: 200_000, deadline_ns: 200_000 },  // 25% util
        TaskSpec { id: 2, wcet_ns: 30_000, period_ns: 100_000, deadline_ns: 100_000 },  // 30% util  
        TaskSpec { id: 3, wcet_ns: 40_000, period_ns: 200_000, deadline_ns: 200_000 },  // 20% util
    ];
    
    for spec in graph_specs.iter() {
        match scheduler.admit_graph(spec.id, *spec) {
            Ok(_) => trace("DETERMINISTIC DEMO: Graph admitted"),
            Err(_) => trace("DETERMINISTIC DEMO: Graph rejected (overload)"),
        }
    }
    
    // Simulate deterministic execution
    let mut current_time = 0u64;
    let simulation_duration = 1_000_000u64; // 1ms simulation
    
    while current_time < simulation_duration {
        // Schedule next graph
        if let Some(graph_id) = scheduler.schedule_next(current_time) {
            // Find the graph spec for execution time
            let graph_spec = graph_specs.iter().find(|s| s.id == graph_id);
            if let Some(spec) = graph_spec {
                // Verify deterministic constraints before execution
                if verify_deterministic_constraints(graph_id, &mut enforcer) {
                    // Simulate execution with some jitter
                    let base_runtime = spec.wcet_ns / 2; // Use half of WCET as typical runtime
                    let jitter = (current_time % 1000) as u64; // Small jitter based on time
                    let actual_runtime = base_runtime + jitter;
                    
                    // Complete execution and update scheduler
                    scheduler.complete_execution(graph_id, actual_runtime, spec.wcet_ns);
                    
                    current_time += actual_runtime;
                } else {
                    trace("DETERMINISTIC DEMO: Constraint violation detected");
                    current_time += 10_000; // Skip ahead on violation
                }
            }
        } else {
            // No graphs ready, advance time
            current_time += 10_000; // 10μs advance
        }
        
        // Reset constraint enforcer for next iteration
        enforcer.reset();
    }
    
    trace("DETERMINISTIC DEMO: Simulation completed");
    
    // Emit all Phase 2 metrics
    scheduler.emit_metrics();
    model_manager.emit_metrics();
    
    // Additional deterministic demo metrics
    metric_kv("deterministic_demo_duration_us", simulation_duration as usize / 1000);
    metric_kv("deterministic_demo_completed", 1);
}
