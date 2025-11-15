//! Shadow divergence event log (ring buffer)

use spin::Mutex;

use super::agent::ShadowMode;

#[derive(Debug, Clone, Copy)]
pub struct DivergenceEvent {
    pub timestamp_ms: u64,
    pub confidence_delta: u32,
    pub action_matches: bool,
    pub mode: ShadowMode,
    pub trace_id: Option<u64>,
}

pub struct DivergenceLog {
    buf: heapless::Vec<DivergenceEvent, 256>,
}

impl DivergenceLog {
    const fn new() -> Self { Self { buf: heapless::Vec::new() } }

    fn push(&mut self, ev: DivergenceEvent) {
        if self.buf.len() == self.buf.capacity() {
            // Drop oldest
            let _ = self.buf.remove(0);
        }
        let _ = self.buf.push(ev);
    }

    pub fn recent(&self, max: usize) -> alloc::vec::Vec<DivergenceEvent> {
        let n = core::cmp::min(max, self.buf.len());
        let start = self.buf.len().saturating_sub(n);
        self.buf.iter().skip(start).take(n).cloned().collect()
    }

    pub fn len(&self) -> usize { self.buf.len() }
}

pub static DIVERGENCE_LOG: Mutex<DivergenceLog> = Mutex::new(DivergenceLog::new());

pub fn log_event(confidence_delta: u32, action_matches: bool, mode: ShadowMode) {
    log_event_with_trace(None, confidence_delta, action_matches, mode);
}

pub fn log_event_with_trace(trace_id: Option<u64>, confidence_delta: u32, action_matches: bool, mode: ShadowMode) {
    let ev = DivergenceEvent { timestamp_ms: crate::time::get_uptime_ms(), confidence_delta, action_matches, mode, trace_id };
    DIVERGENCE_LOG.lock().push(ev);
}
