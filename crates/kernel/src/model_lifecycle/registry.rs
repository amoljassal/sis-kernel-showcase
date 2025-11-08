//! Model Registry - Manages model metadata and lifecycle state
//!
//! The registry maintains model metadata in /models/registry.json (ext4-backed)
//! and provides atomic operations for model management.

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use crate::lib::error::{Result, Errno};
use crate::vfs::{self, OpenFlags};
use crate::time;

/// Model metadata stored in registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub version: String,
    pub hash: [u8; 32],              // SHA-256
    pub signature: alloc::vec::Vec<u8>,         // Ed25519 (64 bytes)
    pub status: ModelStatus,
    pub loaded_at: u64,              // UNIX timestamp
    pub health: Option<HealthMetrics>,
}

/// Model status in registry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelStatus {
    Active,
    Shadow,
    Rollback,
    Failed,
}

/// Health check metrics for model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub inference_latency_p99_us: u64,
    pub memory_footprint_bytes: usize,
    pub test_accuracy: f32,
}

/// Model registry managing all models
#[derive(Debug)]
pub struct ModelRegistry {
    registry_path: &'static str,
    models: Vec<ModelMetadata>,
    active: Option<String>,
    shadow: Option<String>,
    rollback: Option<String>,
}

impl ModelRegistry {
    pub const REGISTRY_PATH: &'static str = "/models/registry.json";

    /// Create new model registry
    pub const fn new() -> Self {
        Self {
            registry_path: Self::REGISTRY_PATH,
            models: Vec::new(),
            active: None,
            shadow: None,
            rollback: None,
        }
    }

    /// Load registry from ext4
    pub fn load(&mut self) -> Result<()> {
        // Try to open registry file; if not present, return Ok with empty registry
        if let Ok(file) = vfs::open(self.registry_path, OpenFlags::O_RDONLY) {
            let size = file.size().unwrap_or(0) as usize;
            let mut buf = alloc::vec::Vec::with_capacity(size.max(1));
            buf.resize(size, 0);
            let _ = file.read(&mut buf[..])?;

            // Deserialize
            #[derive(Serialize, Deserialize)]
            struct SerializedRegistry {
                models: alloc::vec::Vec<ModelMetadata>,
                active: Option<String>,
                shadow: Option<String>,
                rollback: Option<String>,
            }
            if let Ok(sr) = serde_json::from_slice::<SerializedRegistry>(&buf) {
                self.models = sr.models;
                self.active = sr.active;
                self.shadow = sr.shadow;
                self.rollback = sr.rollback;
            }
        }
        Ok(())
    }

    /// Save registry to ext4 (journaled)
    pub fn save(&self) -> Result<()> {
        // Ensure directory exists
        let _ = vfs::mkdir("/models", 0o755);

        // Serialize
        #[derive(Serialize)]
        struct SerializedRegistry<'a> {
            models: &'a [ModelMetadata],
            active: &'a Option<String>,
            shadow: &'a Option<String>,
            rollback: &'a Option<String>,
        }
        let sr = SerializedRegistry {
            models: &self.models,
            active: &self.active,
            shadow: &self.shadow,
            rollback: &self.rollback,
        };
        let json = serde_json::to_vec(&sr).map_err(|_| Errno::EINVAL)?;

        // Create or truncate file and write
        let file = match vfs::open(self.registry_path, OpenFlags::O_WRONLY | OpenFlags::O_TRUNC) {
            Ok(f) => f,
            Err(_) => vfs::create(self.registry_path, 0o644, OpenFlags::O_WRONLY | OpenFlags::O_CREAT | OpenFlags::O_TRUNC)?,
        };
        let _ = file.write(&json[..])?;

        // Append history entry
        let history_path = "/models/registry.log";
        let mut entry = alloc::format!(
            "ts={} active={:?} shadow={:?} rollback={:?} node={}\n",
            time::get_uptime_ms(),
            self.active,
            self.shadow,
            self.rollback,
            option_env!("NODE_ID").unwrap_or(option_env!("TARGET").unwrap_or("unknown"))
        );
        let hist = match vfs::open(history_path, OpenFlags::O_WRONLY) {
            Ok(f) => f,
            Err(_) => vfs::create(history_path, 0o644, OpenFlags::O_WRONLY | OpenFlags::O_CREAT)?,
        };
        // Seek to end (simple: read size and set offset)
        let _ = hist.lseek(0, 2); // SEEK_END
        let _ = hist.write(entry.as_bytes());
        Ok(())
    }

    /// List all models
    pub fn list(&self) -> &[ModelMetadata] {
        &self.models
    }

    /// Get active model
    pub fn active(&self) -> Option<&ModelMetadata> {
        self.active.as_ref().and_then(|v| {
            self.models.iter().find(|m| &m.version == v)
        })
    }

    /// Get shadow model
    pub fn shadow(&self) -> Option<&ModelMetadata> {
        self.shadow.as_ref().and_then(|v| {
            self.models.iter().find(|m| &m.version == v)
        })
    }

    /// Get rollback model
    pub fn rollback(&self) -> Option<&ModelMetadata> {
        self.rollback.as_ref().and_then(|v| {
            self.models.iter().find(|m| &m.version == v)
        })
    }

    /// Set active model version
    pub fn set_active(&mut self, version: &str) {
        self.active = Some(String::from(version));

        // Update model status
        for model in &mut self.models {
            if model.version == version {
                model.status = ModelStatus::Active;
            } else if model.status == ModelStatus::Active {
                model.status = ModelStatus::Rollback;
            }
        }
    }

    /// Set shadow model version
    pub fn set_shadow(&mut self, version: &str) {
        self.shadow = Some(String::from(version));

        // Update model status
        for model in &mut self.models {
            if model.version == version {
                model.status = ModelStatus::Shadow;
            }
        }
    }

    /// Set rollback model version
    pub fn set_rollback(&mut self, version: &str) {
        self.rollback = Some(String::from(version));

        // Update model status
        for model in &mut self.models {
            if model.version == version {
                model.status = ModelStatus::Rollback;
            }
        }
    }

    /// Add model to registry
    pub fn add_model(&mut self, metadata: ModelMetadata) {
        // Remove existing entry for this version
        self.models.retain(|m| m.version != metadata.version);

        // Add new entry
        self.models.push(metadata);
    }

    /// Remove model from registry
    pub fn remove_model(&mut self, version: &str) -> Result<()> {
        if self.active.as_ref().map(|v| v == version).unwrap_or(false) {
            return Err(Errno::EBUSY);  // Cannot remove active model
        }

        self.models.retain(|m| m.version != version);

        if self.shadow.as_ref().map(|v| v == version).unwrap_or(false) {
            self.shadow = None;
        }
        if self.rollback.as_ref().map(|v| v == version).unwrap_or(false) {
            self.rollback = None;
        }

        Ok(())
    }

    /// Update health metrics for a model
    pub fn update_health(&mut self, version: &str, health: HealthMetrics) -> Result<()> {
        for model in &mut self.models {
            if model.version == version {
                model.health = Some(health);
                return Ok(());
            }
        }
        Err(Errno::ENOENT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_basic() {
        let mut registry = ModelRegistry::new();

        let metadata = ModelMetadata {
            version: String::from("v1.0.0"),
            hash: [0u8; 32],
            signature: alloc::vec::Vec::new(),
            status: ModelStatus::Active,
            loaded_at: 0,
            health: None,
        };

        registry.add_model(metadata);
        assert_eq!(registry.list().len(), 1);
    }

    #[test]
    fn test_registry_active_model() {
        let mut registry = ModelRegistry::new();

        let metadata = ModelMetadata {
            version: String::from("v1.0.0"),
            hash: [0u8; 32],
            signature: alloc::vec::Vec::new(),
            status: ModelStatus::Active,
            loaded_at: 0,
            health: None,
        };

        registry.add_model(metadata);
        registry.set_active("v1.0.0");

        assert!(registry.active().is_some());
        assert_eq!(registry.active().unwrap().version, "v1.0.0");
    }
}
