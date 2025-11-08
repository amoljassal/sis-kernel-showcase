//! Incident Bundle Export
//!
//! Exports decision traces, system snapshot, and model info
//! as JSON bundles for forensic investigation.

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Serialize, Deserialize};
use crate::lib::error::{Result, Errno};
use crate::vfs::{self, OpenFlags};
use super::decision::DecisionTrace;

/// Complete incident bundle for export
#[derive(Debug, Serialize, Deserialize)]
pub struct IncidentBundle {
    pub incident_id: String,
    pub exported_at: u64,
    pub traces: Vec<DecisionTrace>,
    pub model_info: ModelInfo,
    pub system_snapshot: SystemSnapshot,
    pub config: ConfigInfo,
    #[cfg(feature = "shadow-mode")]
    pub shadow_divergences: Vec<ShadowDivergence>,
}

/// Model information snapshot
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub version: String,
    pub hash: [u8; 32],
    pub loaded_at: u64,
}

/// System state snapshot
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub uptime_ms: u64,
    pub heap_stats: HeapStats,
    pub logs: Vec<String>,  // Last N log entries
}

/// Heap statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct HeapStats {
    pub allocated: usize,
    pub peak: usize,
}

/// Build configuration info
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigInfo {
    pub features: String,
    pub git_commit: String,
    pub build_timestamp: u64,
}

/// Shadow divergence item (for incident bundles)
#[cfg(feature = "shadow-mode")]
#[derive(Debug, Serialize, Deserialize)]
pub struct ShadowDivergence {
    pub timestamp_ms: u64,
    pub confidence_delta: u32,
    pub action_matches: bool,
    pub mode: alloc::string::String,
}

/// Incident bundle exporter
pub struct IncidentExporter {
    export_dir: &'static str,
    counter: spin::Mutex<u32>,
}

impl IncidentExporter {
    pub const EXPORT_DIR: &'static str = "/incidents";

    /// Create new exporter
    pub const fn new() -> Self {
        Self {
            export_dir: Self::EXPORT_DIR,
            counter: spin::Mutex::new(0),
        }
    }

    /// Export incident bundle for given trace IDs
    pub fn export_bundle(&self, trace_ids: &[u64]) -> Result<String> {
        // 1. Gather traces
        let traces = self.gather_traces(trace_ids)?;

        if traces.is_empty() {
            return Err(Errno::ENOENT);
        }

        // 2. Build bundle
        let bundle = IncidentBundle {
            incident_id: self.generate_incident_id(),
            exported_at: crate::time::get_uptime_ms() * 1000,
            traces,
            model_info: self.get_model_info(),
            system_snapshot: self.get_system_snapshot(),
            config: self.get_config(),
            #[cfg(feature = "shadow-mode")]
            shadow_divergences: alloc::vec::Vec::new(),
        };

        // 3. Serialize to JSON
        let json = serde_json::to_string_pretty(&bundle)
            .map_err(|_| Errno::EINVAL)?;

        // 4. Write to ext4
        let filename = alloc::format!("{}/{}.json",
            self.export_dir, bundle.incident_id);

        self.write_file(&filename, json.as_bytes())?;

        Ok(filename)
    }

    /// Export recent shadow divergences into a bundle (no traces required)
    #[cfg(feature = "shadow-mode")]
    pub fn export_shadow_divergences(&self, max: usize) -> Result<String> {
        use crate::shadow::divergence::DIVERGENCE_LOG;

        let recents = DIVERGENCE_LOG.lock().recent(max);
        let items: Vec<ShadowDivergence> = recents.iter().map(|e| ShadowDivergence {
            timestamp_ms: e.timestamp_ms,
            confidence_delta: e.confidence_delta,
            action_matches: e.action_matches,
            mode: alloc::format!("{:?}", e.mode),
        }).collect();

        let mut bundle = IncidentBundle {
            incident_id: self.generate_incident_id(),
            exported_at: crate::time::get_uptime_ms() * 1000,
            traces: Vec::new(),
            model_info: self.get_model_info(),
            system_snapshot: self.get_system_snapshot(),
            config: self.get_config(),
            shadow_divergences: items,
        };

        let json = serde_json::to_string_pretty(&bundle).map_err(|_| Errno::EINVAL)?;
        let filename = alloc::format!("{}/{}.json", self.export_dir, bundle.incident_id);
        self.write_file(&filename, json.as_bytes())?;
        Ok(filename)
    }

    /// Export all traces in buffer
    pub fn export_all(&self) -> Result<String> {
        use super::buffer::TRACE_BUFFER;

        let traces = TRACE_BUFFER.drain_all();
        let trace_ids: Vec<u64> = traces.iter().map(|t| t.trace_id).collect();

        self.export_bundle(&trace_ids)
    }

    // Private helper methods

    fn gather_traces(&self, trace_ids: &[u64]) -> Result<Vec<DecisionTrace>> {
        use super::buffer::TRACE_BUFFER;

        let traces: Vec<DecisionTrace> = trace_ids.iter()
            .filter_map(|id| TRACE_BUFFER.find_by_trace_id(*id))
            .collect();

        Ok(traces)
    }

    fn generate_incident_id(&self) -> String {
        let mut counter = self.counter.lock();
        let id = *counter;
        *counter += 1;

        let timestamp = crate::time::get_uptime_ms() / 1000;  // Seconds
        alloc::format!("INC-{}-{:03}", timestamp, id)
    }

    fn get_model_info(&self) -> ModelInfo {
        // TODO: Get actual model info from registry
        ModelInfo {
            version: String::from("unknown"),
            hash: [0u8; 32],
            loaded_at: 0,
        }
    }

    fn get_system_snapshot(&self) -> SystemSnapshot {
        SystemSnapshot {
            uptime_ms: crate::time::get_uptime_ms(),
            heap_stats: HeapStats {
                allocated: 0,  // TODO: Get from heap allocator
                peak: 0,
            },
            logs: Vec::new(),  // TODO: Get from kernel log buffer
        }
    }

    fn get_config(&self) -> ConfigInfo {
        let features = option_env!("FEATURES").unwrap_or("");
        let git_commit = option_env!("GIT_COMMIT").unwrap_or("unknown");
        let build_timestamp = option_env!("BUILD_TIMESTAMP")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        ConfigInfo {
            features: String::from(features),
            git_commit: String::from(git_commit),
            build_timestamp,
        }
    }

    fn write_file(&self, _path: &str, _data: &[u8]) -> Result<()> {
        // Ensure directory exists
        let _ = vfs::mkdir(Self::EXPORT_DIR, 0o755);
        let file = match vfs::open(_path, OpenFlags::O_WRONLY | OpenFlags::O_TRUNC) {
            Ok(f) => f,
            Err(_) => vfs::create(_path, 0o644, OpenFlags::O_WRONLY | OpenFlags::O_CREAT | OpenFlags::O_TRUNC)?,
        };
        let _ = file.write(_data)?;
        Ok(())
    }
}

/// Global incident exporter instance
pub static INCIDENT_EXPORTER: IncidentExporter = IncidentExporter::new();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incident_exporter() {
        let exporter = IncidentExporter::new();
        let id1 = exporter.generate_incident_id();
        let id2 = exporter.generate_incident_id();

        assert!(id1.starts_with("INC-"));
        assert_ne!(id1, id2);
    }
}
