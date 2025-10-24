//! Control plane message parsing and graph wiring (V0 binary framing).
//! Frame header: magic 'C'(0x43), ver u8(0), cmd u8, flags u8, len u32 LE, payload[len].
//! Commands:
//!  0x01 CreateGraph {}
//!  0x02 AddChannel { capacity_le_u16 }
//!  0x03 AddOperator { op_id_le_u32, in_ch_le_u16(0xFFFF=none), out_ch_le_u16(0xFFFF=none), priority_u8, stage_u8 }
//!  0x04 StartGraph { steps_le_u32 }
//!  0x05 AddOperatorTyped { op_id_le_u32, in_ch_le_u16(0xFFFF=none), out_ch_le_u16(0xFFFF=none), priority_u8, stage_u8, in_schema_le_u32, out_schema_le_u32 }
//!  0x06 EnableDeterministic { wcet_ns_le_u64, period_ns_le_u64, deadline_ns_le_u64 }
//!  0x10 LlmLoad { wcet_cycles_le_u64 } (feature: `llm`)
//!  0x11 LlmInferStart { max_tokens_le_u16, prompt_utf8[...] } (feature: `llm`)
//!  0x12 LlmInferPoll { infer_id_le_u32 } (feature: `llm`, reserved for streaming)
//!  0x13 LlmCancel { infer_id_le_u32 } (feature: `llm`)

use crate::graph::{GraphApi, OperatorSpec, Stage};
#[cfg(feature = "llm")]
use crate::llm;
use crate::trace::metric_kv;
use core::sync::atomic::{AtomicU64, Ordering};
#[cfg(feature = "virtio-console")]
use heapless::String as HString;

/// Maximum allowed control payload length (bytes)
pub const MAX_CTRL_LEN: usize = 64;

/// Simple 64-bit capability token for control-plane authorization.
/// Frames must include this token as the first 8 bytes of payload.
static CONTROL_TOKEN: AtomicU64 = AtomicU64::new(0x53535F4354524C21); // legacy dev token
static CONTROL_TOKEN_ADMIN: AtomicU64 = AtomicU64::new(0x53535F4354524C21);
static CONTROL_TOKEN_SUBMIT: AtomicU64 = AtomicU64::new(0x53535F4354524C21);

static mut CTRL_GRAPH: Option<GraphApi> = None;

pub enum CtrlError {
    BadFrame,
    Unsupported,
    NoGraph,
    Oversize,
    AuthFailed,
}

fn read_token(payload: &[u8]) -> Option<(u64, &[u8])> {
    if payload.len() < 8 { return None; }
    let t = u64::from_le_bytes([
        payload[0], payload[1], payload[2], payload[3],
        payload[4], payload[5], payload[6], payload[7]
    ]);
    Some((t, &payload[8..]))
}

#[inline(always)]
fn check_len(len: usize) -> Result<(), CtrlError> {
    if len > MAX_CTRL_LEN { return Err(CtrlError::Oversize); }
    Ok(())
}

#[inline(always)]
fn check_token(tok: u64) -> Result<(), CtrlError> {
    let expect = CONTROL_TOKEN.load(Ordering::Relaxed);
    if tok != expect { return Err(CtrlError::AuthFailed); }
    Ok(())
}

// Minimal decimal writer into a heapless::String
#[cfg(feature = "virtio-console")]
fn write_decimal_u64(buf: &mut HString<192>, mut v: u64) -> Result<(), ()> {
    let mut tmp = [0u8; 20];
    let mut i = 0;
    if v == 0 {
        buf.push('0').map_err(|_| ())?; return Ok(());
    }
    while v > 0 && i < tmp.len() {
        tmp[i] = b'0' + (v % 10) as u8;
        v /= 10; i += 1;
    }
    while i > 0 { i -= 1; buf.push(tmp[i] as char).map_err(|_| ())?; }
    Ok(())
}

#[inline(always)]
fn token_split(tok: u64) -> (u8, u64) {
    let rights = (tok >> 56) as u8;
    let secret = tok & 0x00FF_FFFF_FFFF_FFFF;
    (rights, secret)
}

#[inline(always)]
fn check_embedded_rights(tok: u64, need_admin: bool, need_submit: bool) -> Result<(), CtrlError> {
    let (rights, secret) = token_split(tok);
    // Secret must match CONTROL_TOKEN (lower 56 bits)
    let expect = CONTROL_TOKEN.load(Ordering::Relaxed) & 0x00FF_FFFF_FFFF_FFFF;
    if secret != expect { return Err(CtrlError::AuthFailed); }
    // Rights bitmask: bit0=ADMIN, bit1=SUBMIT
    let admin_ok = rights & 0x01 != 0;
    let submit_ok = rights & 0x02 != 0;
    if need_admin { if admin_ok { return Ok(()); } else { return Err(CtrlError::AuthFailed); } }
    if need_submit { if submit_ok || admin_ok { return Ok(()); } else { return Err(CtrlError::AuthFailed); } }
    Ok(())
}

#[inline(always)]
fn check_token_admin(tok: u64) -> Result<(), CtrlError> {
    let expect = CONTROL_TOKEN_ADMIN.load(Ordering::Relaxed);
    if tok != expect { return Err(CtrlError::AuthFailed); }
    Ok(())
}

#[inline(always)]
fn check_token_submit_or_admin(tok: u64) -> Result<(), CtrlError> {
    let a = CONTROL_TOKEN_ADMIN.load(Ordering::Relaxed);
    let s = CONTROL_TOKEN_SUBMIT.load(Ordering::Relaxed);
    if tok == a || tok == s { Ok(()) } else { Err(CtrlError::AuthFailed) }
}

pub fn handle_frame(frame: &[u8]) -> Result<(), CtrlError> {
    if frame.len() < 8 { return Err(CtrlError::BadFrame); }
    if frame[0] != 0x43 { return Err(CtrlError::BadFrame); } // 'C'
    let ver = frame[1];
    let cmd = frame[2];
    let _flags = frame[3];
    let len = u32::from_le_bytes([frame[4], frame[5], frame[6], frame[7]]) as usize;
    if ver != 0 { return Err(CtrlError::Unsupported); }
    if frame.len() < 8 + len { return Err(CtrlError::BadFrame); }
    check_len(len)?;
    let payload = &frame[8..8+len];

    match cmd {
        0x01 => { // CreateGraph
            let (tok, _p) = read_token(payload).ok_or(CtrlError::BadFrame)?;
            check_token(tok)?;
            unsafe { CTRL_GRAPH = Some(GraphApi::create()); }
            ctrl_print(b"CTRL: graph created\n");
            // Emit basic graph stats metrics (ops/channels)
            if let Some((ops, chans)) = current_graph_counts() {
                metric_kv("graph_stats_ops", ops);
                metric_kv("graph_stats_channels", chans);
            }
            Ok(())
        }
        0x02 => { // AddChannel
            if payload.len() < (8+2) { return Err(CtrlError::BadFrame); }
            let (tok, p) = read_token(payload).ok_or(CtrlError::BadFrame)?;
            check_token(tok)?;
            let cap = u16::from_le_bytes([p[0], p[1]]) as usize;
            unsafe {
                if let Some(ref mut g) = CTRL_GRAPH {
                    let _ = g.add_channel(crate::graph::ChannelSpec { capacity: cap });
                    ctrl_print(b"CTRL: channel added\n");
                    if let Some((ops, chans)) = current_graph_counts() {
                        metric_kv("graph_stats_ops", ops);
                        metric_kv("graph_stats_channels", chans);
                    }
                    Ok(())
                } else { Err(CtrlError::NoGraph) }
            }
        }
        0x03 => { // AddOperator
            ctrl_print(b"CTRL: begin add-operator\n");
            if payload.len() < (8+4+2+2+1+1) { ctrl_print(b"CTRL: bad frame len\n"); return Err(CtrlError::BadFrame); }
            let (tok, p) = read_token(payload).ok_or(CtrlError::BadFrame)?;
            check_token(tok)?;
            let op_id = u32::from_le_bytes([p[0],p[1],p[2],p[3]]);
            let in_ch = u16::from_le_bytes([p[4],p[5]]);
            let out_ch = u16::from_le_bytes([p[6],p[7]]);
            let prio = p[8];
            let stage_u8 = p[9];
            let stage = match stage_u8 { 0=>Some(Stage::AcquireData),1=>Some(Stage::CleanData),2=>Some(Stage::ExploreData),3=>Some(Stage::ModelData),4=>Some(Stage::ExplainResults), _=>None };
            unsafe {
                if let Some(ref mut g) = CTRL_GRAPH {
                    // Defensive index checks
                    if in_ch != 0xFFFF && (in_ch as usize) >= g.counts().1 { ctrl_print(b"CTRL: in_ch OOR\n"); }
                    if out_ch != 0xFFFF && (out_ch as usize) >= g.counts().1 { ctrl_print(b"CTRL: out_ch OOR\n"); }
                    let spec = OperatorSpec {
                        id: op_id,
                        func: crate::graph::op_a_run,
                        in_ch: if in_ch==0xFFFF { None } else { Some(in_ch as usize) },
                        out_ch: if out_ch==0xFFFF { None } else { Some(out_ch as usize) },
                        priority: prio,
                        stage,
                        in_schema: None,
                        out_schema: None,
                    };
                    ctrl_print(b"CTRL: inserting operator\n");
                    let _idx = g.add_operator(spec);
                    ctrl_print(b"CTRL: operator added\n");
                    if let Some((ops, chans)) = current_graph_counts() {
                        metric_kv("graph_stats_ops", ops);
                        metric_kv("graph_stats_channels", chans);
                    }
                    Ok(())
                } else { Err(CtrlError::NoGraph) }
            }
        }
        0x04 => { // StartGraph (run steps)
            if payload.len() < (8+4) { return Err(CtrlError::BadFrame); }
            let (tok, p) = read_token(payload).ok_or(CtrlError::BadFrame)?;
            check_token(tok)?;
            let steps = u32::from_le_bytes([p[0],p[1],p[2],p[3]]) as usize;
            unsafe {
                if let Some(ref mut g) = CTRL_GRAPH {
                    crate::trace::trace("graph_start");
                    let t0 = crate::graph::now_cycles();
                    g.run_steps(steps);
                    let t1 = crate::graph::now_cycles();
                    let ns = crate::graph::cycles_to_ns(t1.saturating_sub(t0));
                    metric_kv("scheduler_run_us", (ns / 1000) as usize);
                    crate::trace::trace("graph_end");
                    ctrl_print(b"CTRL: ran steps\n");
                    Ok(())
                } else { Err(CtrlError::NoGraph) }
            }
        }
        0x05 => { // AddOperatorTyped
            ctrl_print(b"CTRL: begin add-operator (typed)\n");
            if payload.len() < (8+4+2+2+1+1+4+4) { return Err(CtrlError::BadFrame); }
            let (tok, p) = read_token(payload).ok_or(CtrlError::BadFrame)?;
            check_token(tok)?;
            let op_id = u32::from_le_bytes([p[0],p[1],p[2],p[3]]);
            let in_ch = u16::from_le_bytes([p[4],p[5]]);
            let out_ch = u16::from_le_bytes([p[6],p[7]]);
            let prio = p[8];
            let stage_u8 = p[9];
            let in_schema = u32::from_le_bytes([p[10],p[11],p[12],p[13]]);
            let out_schema = u32::from_le_bytes([p[14],p[15],p[16],p[17]]);
            let stage = match stage_u8 { 0=>Some(Stage::AcquireData),1=>Some(Stage::CleanData),2=>Some(Stage::ExploreData),3=>Some(Stage::ModelData),4=>Some(Stage::ExplainResults), _=>None };
            unsafe {
                if let Some(ref mut g) = CTRL_GRAPH {
                    // Defensive: ensure channel indices are in range when present
                    if in_ch != 0xFFFF && (in_ch as usize) >= g.counts().1 { ctrl_print(b"CTRL: typed in_ch OOR\n"); }
                    if out_ch != 0xFFFF && (out_ch as usize) >= g.counts().1 { ctrl_print(b"CTRL: typed out_ch OOR\n"); }
                    let spec = OperatorSpec {
                        id: op_id,
                        func: crate::graph::op_a_run,
                        in_ch: if in_ch==0xFFFF { None } else { Some(in_ch as usize) },
                        out_ch: if out_ch==0xFFFF { None } else { Some(out_ch as usize) },
                        priority: prio,
                        stage,
                        in_schema: if in_schema == 0 { None } else { Some(in_schema) },
                        out_schema: if out_schema == 0 { None } else { Some(out_schema) },
                    };
                    ctrl_print(b"CTRL: inserting operator (typed)\n");
                    let has_schema = spec.in_schema.is_some() || spec.out_schema.is_some();
                    let ok = g.add_operator_strict(spec);
                    if !ok {
                        ctrl_print(b"CTRL: operator rejected (schema mismatch)\n");
                        return Ok(());
                    }
                    if has_schema {
                        ctrl_print(b"CTRL: operator added (schema verified)\n");
                    } else {
                        ctrl_print(b"CTRL: operator added (untyped)\n");
                    }
                    if let Some((ops, chans)) = current_graph_counts() {
                        metric_kv("graph_stats_ops", ops);
                        metric_kv("graph_stats_channels", chans);
                    }
                    Ok(())
                } else { Err(CtrlError::NoGraph) }
            }
        }
        0x06 => { // EnableDeterministic (graph-level)
            if payload.len() < (8+8+8+8) { return Err(CtrlError::BadFrame); }
            let (tok, p) = read_token(payload).ok_or(CtrlError::BadFrame)?;
            check_token(tok)?;
            let _wcet = u64::from_le_bytes([p[0],p[1],p[2],p[3],p[4],p[5],p[6],p[7]]);
            let _period = u64::from_le_bytes([p[8],p[9],p[10],p[11],p[12],p[13],p[14],p[15]]);
            let _deadline = u64::from_le_bytes([p[16],p[17],p[18],p[19],p[20],p[21],p[22],p[23]]);
            unsafe {
                if let Some(ref mut _g) = CTRL_GRAPH {
                    #[cfg(feature = "deterministic")]
                    {
                        let ok = _g.enable_deterministic(_wcet, _period, _deadline);
                        if ok { ctrl_print(b"CTRL: deterministic enabled\n"); } else { ctrl_print(b"CTRL: deterministic admit rejected\n"); }
                        return Ok(());
                    }
                    #[cfg(not(feature = "deterministic"))]
                    {
                        ctrl_print(b"CTRL: deterministic feature not enabled\n");
                        return Ok(());
                    }
                } else { Err(CtrlError::NoGraph) }
            }
        }
        0x10 => { // LlmLoad
            let (tok, p) = read_token(payload).ok_or(CtrlError::BadFrame)?;
            // Explicit allow/deny with audit
            #[cfg(feature = "llm")]
            {
                // Accept either explicit admin token, or embedded admin rights
                let mut ok = check_token_admin(tok).is_ok();
                if !ok { ok = check_embedded_rights(tok, true, false).is_ok(); }
                if !ok { llm::audit(1, 0, 0, 0, 0, 0b010); return Err(CtrlError::AuthFailed); }
            }
            #[cfg(not(feature = "llm"))]
            {
                check_token(tok)?;
            }
            #[cfg(feature = "llm")]
            {
                if p.len() >= 8 {
                    let wcet = u64::from_le_bytes([p[0],p[1],p[2],p[3],p[4],p[5],p[6],p[7]]);
                    let ok = crate::llm::load_model(Some(wcet));
                    llm::audit(1, 0, 0, wcet, 0, if ok { 0b001 } else { 0b010 });
                    if ok {
                        ctrl_print(b"CTRL: llm load ok (wcet configured)\n");
                    } else {
                        ctrl_print(b"CTRL: llm load fail\n");
                    }
                    return Ok(());
                } else {
                    let _ = crate::llm::load_model(None);
                    llm::audit(1, 0, 0, 0, 0, 0b001);
                    ctrl_print(b"CTRL: llm load ok (default wcet)\n");
                    return Ok(());
                }
            }
            #[cfg(not(feature = "llm"))]
            {
                ctrl_print(b"CTRL: llm feature not enabled\n");
                Ok(())
            }
        }
        0x11 => { // LlmInferStart
            if payload.len() < (8+2) { return Err(CtrlError::BadFrame); }
            let (tok, p) = read_token(payload).ok_or(CtrlError::BadFrame)?;
            #[cfg(feature = "llm")]
            {
                // Accept explicit submit/admin tokens, or embedded submit/admin rights
                let mut ok = check_token_submit_or_admin(tok).is_ok();
                if !ok { ok = check_embedded_rights(tok, false, true).is_ok(); }
                if !ok { llm::audit(3, 0, 0, 0, 0, 0b010); return Err(CtrlError::AuthFailed); }
            }
            #[cfg(not(feature = "llm"))]
            {
                check_token(tok)?;
            }
            #[cfg(feature = "llm")]
            {
                let max_t = u16::from_le_bytes([p[0], p[1]]) as usize;
                // Remaining bytes are utf8 prompt (bounded by MAX_CTRL_LEN)
                let prompt_bytes = &p[2..];
                let prompt = core::str::from_utf8(prompt_bytes).unwrap_or("");
                let res = crate::llm::infer(prompt, Some(max_t));
                llm::audit(3, prompt_bytes.len(), max_t, 0, 0, 0b001);
                ctrl_print(b"CTRL: llm infer submitted\n");
                unsafe {
                    crate::uart_print(b"[LLM][CTL] id=");
                    crate::shell::print_number_simple(res.infer_id as u64);
                    crate::uart_print(b" tokens=");
                    crate::shell::print_number_simple(res.tokens_emitted as u64);
                    crate::uart_print(b" us=");
                    crate::shell::print_number_simple(res.latency_us as u64);
                    crate::uart_print(b"\n");
                }
                return Ok(());
            }
            #[cfg(not(feature = "llm"))]
            {
                ctrl_print(b"CTRL: llm feature not enabled\n");
                Ok(())
            }
        }
        0x12 => { // LlmInferPoll (id + optional max)
            let (tok, p) = read_token(payload).ok_or(CtrlError::BadFrame)?;
            check_token(tok)?;
            // Expected: infer_id (u32 LE), optional max (u16 LE)
            let (infer_id, max) = if p.len() >= 4 {
                let id = u32::from_le_bytes([p[0],p[1],p[2],p[3]]) as usize;
                let m = if p.len() >= 6 { u16::from_le_bytes([p[4],p[5]]) as usize } else { 4 };
                (id, m)
            } else {
                (0usize, 4usize)
            };
            #[cfg(feature = "llm")]
            {
                let (id, n, done, items) = if infer_id != 0 { crate::llm::ctl_poll_id(infer_id, max) } else { crate::llm::ctl_poll(max) };
                let (plen, model_id) = crate::llm::ctl_peek_meta(id);
                ctrl_print(b"CTRL: llm poll\n");
                #[cfg(feature = "virtio-console")]
                {
                    // Compose a compact token response line
                    let mut line = heapless::String::<192>::new();
                    let _ = line.push_str("OK TOK id=");
                    let _ = write_decimal_u64(&mut line, id as u64);
                    let _ = line.push_str(" n=");
                    let _ = write_decimal_u64(&mut line, n as u64);
                    let _ = line.push_str(" done=");
                    let _ = write_decimal_u64(&mut line, done as u64);
                    let _ = line.push_str(" plen=");
                    let _ = write_decimal_u64(&mut line, plen as u64);
                    let _ = line.push_str(" model=");
                    match model_id {
                        Some(mid) => { let _ = write_decimal_u64(&mut line, mid as u64); }
                        None => { let _ = line.push_str("none"); }
                    }
                    if items.len() > 0 {
                        let _ = line.push_str(" items=");
                        let _ = line.push_str(items.as_str());
                    }
                    let _ = line.push('\n');
                    // Send over virtio-console data TX
                    let drv = crate::virtio_console::get_virtio_console_driver();
                    let _ = drv.write_data(line.as_bytes());
                }
                unsafe {
                    crate::uart_print(b"[LLM][CTL-POLL] id=");
                    crate::shell::print_number_simple(id as u64);
                    crate::uart_print(b" n=");
                    crate::shell::print_number_simple(n as u64);
                    crate::uart_print(b" done=");
                    crate::shell::print_number_simple(done as u64);
                    crate::uart_print(b" plen=");
                    crate::shell::print_number_simple(plen as u64);
                    crate::uart_print(b" model=");
                    match model_id {
                        Some(mid) => crate::shell::print_number_simple(mid as u64),
                        None => crate::uart_print(b"none"),
                    }
                    crate::uart_print(b" items=");
                    crate::uart_print(items.as_bytes());
                    crate::uart_print(b"\n");
                }
            }
            Ok(())
        }
        0x13 => { // LlmCancel (by id)
            let (tok, p) = read_token(payload).ok_or(CtrlError::BadFrame)?;
            check_token(tok)?;
            let id = if p.len() >= 4 { u32::from_le_bytes([p[0],p[1],p[2],p[3]]) as usize } else { 0 };
            #[cfg(feature = "llm")]
            {
                if id != 0 { crate::llm::ctl_cancel_id(id); } else { crate::llm::ctl_cancel(); }
            }
            ctrl_print(b"CTRL: llm cancel\n");
            Ok(())
        }
        _ => Err(CtrlError::Unsupported),
    }
}

fn ctrl_print(msg: &[u8]) { unsafe { crate::uart_print(msg); } }

/// Expose current graph counts for diagnostics (ops, channels)
pub fn current_graph_counts() -> Option<(usize, usize)> {
    unsafe {
        if let Some(ref g) = CTRL_GRAPH {
            Some(g.counts())
        } else {
            None
        }
    }
}

/// Export the current graph structure to UART
pub fn export_graph_text() -> Result<(), CtrlError> {
    unsafe {
        if let Some(ref g) = CTRL_GRAPH {
            g.export_text();
            Ok(())
        } else {
            Err(CtrlError::NoGraph)
        }
    }
}

pub fn export_graph_json() -> Result<(), CtrlError> {
    unsafe {
        if let Some(ref g) = CTRL_GRAPH {
            g.export_json();
            Ok(())
        } else {
            Err(CtrlError::NoGraph)
        }
    }
}

/// Directly add an operator (used by shell to avoid rare frame-path stalls)
pub fn add_operator_direct(
    op_id: u32,
    in_ch: Option<u16>,
    out_ch: Option<u16>,
    prio: u8,
    stage_u8: u8,
    in_schema: Option<u32>,
    out_schema: Option<u32>,
) -> Result<(), CtrlError> {
    let stage = match stage_u8 { 0=>Some(Stage::AcquireData),1=>Some(Stage::CleanData),2=>Some(Stage::ExploreData),3=>Some(Stage::ModelData),4=>Some(Stage::ExplainResults), _=>None };
    unsafe {
        if let Some(ref mut g) = CTRL_GRAPH {
            let spec = OperatorSpec {
                id: op_id,
                func: crate::graph::op_a_run,
                in_ch: in_ch.map(|v| v as usize),
                out_ch: out_ch.map(|v| v as usize),
                priority: prio,
                stage,
                in_schema,
                out_schema,
            };
            ctrl_print(b"CTRL: begin add-operator (direct)\n");
            let ok = g.add_operator_strict(spec);
            if !ok {
                ctrl_print(b"CTRL: operator rejected (schema mismatch)\n");
                return Ok(());
            }
            if in_schema.is_some() || out_schema.is_some() {
                ctrl_print(b"CTRL: operator added (schema verified)\n");
            } else {
                ctrl_print(b"CTRL: operator added (untyped)\n");
            }
            if let Some((ops, chans)) = current_graph_counts() {
                metric_kv("graph_stats_ops", ops);
                metric_kv("graph_stats_channels", chans);
            }
            Ok(())
        } else { Err(CtrlError::NoGraph) }
    }
}

/// Rotate the control-plane capability token
pub fn set_control_token(new_tok: u64) {
    CONTROL_TOKEN.store(new_tok, Ordering::Relaxed);
}

/// Read the current control-plane capability token
pub fn get_control_token() -> u64 {
    CONTROL_TOKEN.load(Ordering::Relaxed)
}

/// Rotate admin and submit tokens (optional)
pub fn set_admin_token(tok: u64) { CONTROL_TOKEN_ADMIN.store(tok, Ordering::Relaxed); }
pub fn set_submit_token(tok: u64) { CONTROL_TOKEN_SUBMIT.store(tok, Ordering::Relaxed); }
pub fn get_admin_token() -> u64 { CONTROL_TOKEN_ADMIN.load(Ordering::Relaxed) }
pub fn get_submit_token() -> u64 { CONTROL_TOKEN_SUBMIT.load(Ordering::Relaxed) }

/// Enable deterministic mode on current graph (direct)
#[cfg(feature = "deterministic")]
pub fn det_enable_direct(wcet: u64, period: u64, deadline: u64) -> Result<bool, CtrlError> {
    unsafe {
        if let Some(ref mut g) = CTRL_GRAPH {
            Ok(g.enable_deterministic(wcet, period, deadline))
        } else { Err(CtrlError::NoGraph) }
    }
}

/// Disable deterministic mode on current graph (direct)
#[cfg(feature = "deterministic")]
pub fn det_disable_direct() -> Result<(), CtrlError> {
    unsafe {
        if let Some(ref mut g) = CTRL_GRAPH {
            g.disable_deterministic();
            Ok(())
        } else { Err(CtrlError::NoGraph) }
    }
}

/// Get deterministic status and counters
#[cfg(feature = "deterministic")]
pub fn det_status_direct() -> Result<(bool, u64, usize), CtrlError> {
    unsafe {
        if let Some(ref g) = CTRL_GRAPH {
            let enabled = g.deterministic_enabled();
            let wcet = g.det_wcet();
            let overruns = g.det_overruns();
            Ok((enabled, wcet, overruns))
        } else { Err(CtrlError::NoGraph) }
    }
}

/// Reset deterministic counters
#[cfg(feature = "deterministic")]
pub fn det_reset_counters_direct() -> Result<(), CtrlError> {
    unsafe {
        if let Some(ref mut g) = CTRL_GRAPH {
            g.det_reset();
            Ok(())
        } else { Err(CtrlError::NoGraph) }
    }
}

/// Directly add a channel (used by shell to avoid frame-path stalls)
pub fn add_channel_direct(capacity: u16) -> Result<(), CtrlError> {
    unsafe {
        if let Some(ref mut g) = CTRL_GRAPH {
            let _ = g.add_channel(crate::graph::ChannelSpec { capacity: capacity as usize });
            ctrl_print(b"CTRL: channel added (direct)\n");
            if let Some((ops, chans)) = current_graph_counts() {
                metric_kv("graph_stats_ops", ops);
                metric_kv("graph_stats_channels", chans);
            }
            Ok(())
        } else {
            Err(CtrlError::NoGraph)
        }
    }
}
