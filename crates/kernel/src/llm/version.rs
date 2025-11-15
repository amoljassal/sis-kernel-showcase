//! # Adapter Version Control
//!
//! Git-like versioning for LoRA adapters with lineage tracking and rollback.
//!
//! ## Overview
//!
//! This module provides version control for LoRA adapters, similar to Git:
//!
//! - **Commits**: Save adapter states with metadata
//! - **History**: Track parent-child relationships
//! - **Rollback**: Restore previous adapter versions
//! - **Tags**: Mark stable/production versions
//! - **Diff**: Compare adapter versions
//! - **Garbage Collection**: Clean up old versions
//!
//! ## Example Version History
//!
//! ```text
//! v1 (baseline)
//!   ├─ v2 (trained on warehouse A failures)
//!   │   ├─ v3 (fine-tuned for low-light conditions)
//!   │   └─ v4 (adapted to new product types)
//!   └─ v5 (branched: trained on factory floor data)
//!       └─ v6 (merged improvements from v3)
//! ```

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, Ordering};

/// Unique version ID
pub type VersionId = u32;

/// SHA-256 hash of adapter weights
pub type Hash = [u8; 32];

/// Version metadata
#[derive(Debug, Clone)]
pub struct VersionMetadata {
    /// Timestamp (nanoseconds since epoch)
    pub timestamp: u64,
    /// Number of training examples used
    pub training_examples: usize,
    /// Training duration in milliseconds
    pub training_duration_ms: u64,
    /// Final training loss
    pub final_loss: f32,
    /// Accuracy improvement over parent
    pub accuracy_improvement: f32,
    /// Environment tag (e.g., "warehouse_A", "factory_floor_2")
    pub environment_tag: String,
    /// Human-readable description
    pub description: String,
}

impl Default for VersionMetadata {
    fn default() -> Self {
        Self {
            timestamp: 0,
            training_examples: 0,
            training_duration_ms: 0,
            final_loss: 0.0,
            accuracy_improvement: 0.0,
            environment_tag: String::new(),
            description: String::new(),
        }
    }
}

/// Adapter version
#[derive(Debug, Clone)]
pub struct AdapterVersion {
    /// Unique version ID (incremental)
    pub version_id: VersionId,
    /// Parent version (None for initial version)
    pub parent_version: Option<VersionId>,
    /// Version metadata
    pub metadata: VersionMetadata,
    /// Content hash (SHA-256 of adapter weights)
    pub hash: Hash,
    /// Storage path (simplified - would be actual file path)
    pub storage_path: String,
}

impl AdapterVersion {
    /// Create a new adapter version
    pub fn new(
        version_id: VersionId,
        parent_version: Option<VersionId>,
        description: String,
    ) -> Self {
        Self {
            version_id,
            parent_version,
            metadata: VersionMetadata {
                timestamp: crate::time::get_timestamp_us() * 1000, // Convert μs to ns
                description,
                ..Default::default()
            },
            hash: [0u8; 32], // Simplified - would compute actual hash
            storage_path: alloc::format!("v{}_adapter.bin", version_id),
        }
    }

    /// Set training metadata
    pub fn with_training_metadata(
        mut self,
        examples: usize,
        duration_ms: u64,
        loss: f32,
        accuracy_improvement: f32,
    ) -> Self {
        self.metadata.training_examples = examples;
        self.metadata.training_duration_ms = duration_ms;
        self.metadata.final_loss = loss;
        self.metadata.accuracy_improvement = accuracy_improvement;
        self
    }

    /// Set environment tag
    pub fn with_environment(mut self, tag: String) -> Self {
        self.metadata.environment_tag = tag;
        self
    }
}

/// Difference between two versions
#[derive(Debug, Clone)]
pub struct VersionDiff {
    pub version_a: VersionId,
    pub version_b: VersionId,
    pub accuracy_delta: f32,
    pub param_changes: usize, // How many weights changed
    pub time_delta_hours: u64,
}

/// Error during version control operations
#[derive(Debug, Clone)]
pub enum VersionError {
    /// Version not found
    NotFound(VersionId),
    /// Invalid operation
    InvalidOperation(String),
    /// Storage error
    StorageError(String),
}

/// Adapter version control system
pub struct AdapterVersionControl {
    /// Current active version
    current_version: AtomicU32,
    /// Next version ID to assign
    next_version_id: AtomicU32,
    /// Version history (in a real implementation, would use Mutex)
    /// For this demo, we'll use a simplified approach
    versions_count: AtomicU32,
    /// Total commits
    total_commits: AtomicU32,
    /// Total rollbacks
    total_rollbacks: AtomicU32,
}

impl AdapterVersionControl {
    /// Create a new version control system
    pub const fn new() -> Self {
        Self {
            current_version: AtomicU32::new(0),
            next_version_id: AtomicU32::new(1),
            versions_count: AtomicU32::new(0),
            total_commits: AtomicU32::new(0),
            total_rollbacks: AtomicU32::new(0),
        }
    }

    /// Commit current adapter as new version
    ///
    /// In a real implementation, this would:
    /// 1. Serialize adapter weights
    /// 2. Compute SHA-256 hash
    /// 3. Save to storage
    /// 4. Update version index
    pub fn commit(&self, description: &str) -> Result<VersionId, VersionError> {
        let current = self.current_version.load(Ordering::Relaxed);
        let version_id = self.next_version_id.fetch_add(1, Ordering::Relaxed);

        // Create new version
        let parent = if current > 0 { Some(current) } else { None };
        let _version = AdapterVersion::new(version_id, parent, description.to_string());

        // In a real implementation, would save to storage here

        self.current_version.store(version_id, Ordering::Relaxed);
        self.versions_count.fetch_add(1, Ordering::Relaxed);
        self.total_commits.fetch_add(1, Ordering::Relaxed);

        Ok(version_id)
    }

    /// Rollback to previous version
    pub fn rollback(&self, version_id: VersionId) -> Result<(), VersionError> {
        // In a real implementation, would:
        // 1. Check if version exists
        // 2. Load adapter from storage
        // 3. Apply adapter weights
        // 4. Update current version

        if version_id == 0 {
            return Err(VersionError::NotFound(version_id));
        }

        self.current_version.store(version_id, Ordering::Relaxed);
        self.total_rollbacks.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }

    /// Get current version ID
    pub fn current_version(&self) -> VersionId {
        self.current_version.load(Ordering::Relaxed)
    }

    /// Get version history
    ///
    /// In a real implementation, this would load from storage
    pub fn history(&self) -> Vec<AdapterVersion> {
        let mut versions = Vec::new();

        let count = self.versions_count.load(Ordering::Relaxed);
        for i in 1..=count {
            let parent = if i > 1 { Some(i - 1) } else { None };
            versions.push(AdapterVersion::new(
                i,
                parent,
                alloc::format!("Version {}", i),
            ));
        }

        versions
    }

    /// Compare two versions
    pub fn diff(&self, v1: VersionId, v2: VersionId) -> Result<VersionDiff, VersionError> {
        // In a real implementation, would:
        // 1. Load both versions from storage
        // 2. Compare adapter weights
        // 3. Compute differences

        Ok(VersionDiff {
            version_a: v1,
            version_b: v2,
            accuracy_delta: 0.02, // Placeholder
            param_changes: 150,   // Placeholder
            time_delta_hours: 24, // Placeholder
        })
    }

    /// Tag a version
    ///
    /// In a real implementation, this would save tag metadata to storage
    pub fn tag(&self, version_id: VersionId, tag: &str) -> Result<(), VersionError> {
        // Validate version exists
        if version_id == 0 || version_id > self.next_version_id.load(Ordering::Relaxed) {
            return Err(VersionError::NotFound(version_id));
        }

        // In a real implementation, would save tag to storage
        // For now, just validate
        Ok(())
    }

    /// Garbage collect old versions (keep last N)
    pub fn gc(&self, keep_count: usize) -> Result<usize, VersionError> {
        let total = self.versions_count.load(Ordering::Relaxed) as usize;

        if total <= keep_count {
            return Ok(0); // Nothing to collect
        }

        let to_remove = total - keep_count;

        // In a real implementation, would:
        // 1. Identify versions to remove (excluding tagged ones)
        // 2. Delete from storage
        // 3. Update version index

        Ok(to_remove)
    }

    /// Get version control statistics
    pub fn get_stats(&self) -> VersionStats {
        VersionStats {
            total_versions: self.versions_count.load(Ordering::Relaxed),
            current_version: self.current_version.load(Ordering::Relaxed),
            total_commits: self.total_commits.load(Ordering::Relaxed),
            total_rollbacks: self.total_rollbacks.load(Ordering::Relaxed),
        }
    }
}

/// Statistics about version control
#[derive(Debug, Clone, Copy)]
pub struct VersionStats {
    pub total_versions: u32,
    pub current_version: u32,
    pub total_commits: u32,
    pub total_rollbacks: u32,
}

impl Default for AdapterVersionControl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_commit() {
        let vc = AdapterVersionControl::new();

        let v1 = vc.commit("Initial version").unwrap();
        assert_eq!(v1, 1);

        let v2 = vc.commit("Improved version").unwrap();
        assert_eq!(v2, 2);

        assert_eq!(vc.current_version(), 2);
    }

    #[test]
    fn test_version_rollback() {
        let vc = AdapterVersionControl::new();

        let v1 = vc.commit("Version 1").unwrap();
        let _v2 = vc.commit("Version 2").unwrap();
        let _v3 = vc.commit("Version 3").unwrap();

        vc.rollback(v1).unwrap();
        assert_eq!(vc.current_version(), v1);
    }

    #[test]
    fn test_version_history() {
        let vc = AdapterVersionControl::new();

        vc.commit("v1").unwrap();
        vc.commit("v2").unwrap();
        vc.commit("v3").unwrap();

        let history = vc.history();
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].version_id, 1);
        assert_eq!(history[1].parent_version, Some(1));
        assert_eq!(history[2].parent_version, Some(2));
    }

    #[test]
    fn test_version_diff() {
        let vc = AdapterVersionControl::new();

        let v1 = vc.commit("v1").unwrap();
        let v2 = vc.commit("v2").unwrap();

        let diff = vc.diff(v1, v2).unwrap();
        assert_eq!(diff.version_a, v1);
        assert_eq!(diff.version_b, v2);
    }

    #[test]
    fn test_version_tag() {
        let vc = AdapterVersionControl::new();

        let v1 = vc.commit("v1").unwrap();
        vc.tag(v1, "stable").unwrap();

        // Tagging non-existent version should fail
        assert!(vc.tag(999, "invalid").is_err());
    }

    #[test]
    fn test_garbage_collection() {
        let vc = AdapterVersionControl::new();

        for i in 1..=10 {
            vc.commit(&alloc::format!("v{}", i)).unwrap();
        }

        let removed = vc.gc(5).unwrap();
        assert_eq!(removed, 5);
    }
}
