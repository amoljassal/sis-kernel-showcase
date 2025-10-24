//! Signed model package infrastructure (Phase 2):
//! - SHA-256 + Ed25519 signature verification
//! - Model permissions (LOAD/EXECUTE/INSPECT/EXPORT/ATTEST)
//! - Measurement logging for audit trail
//! - Secure model loading with capability enforcement

use crate::cap::{Capability, CapRights, CapObjectKind};
use crate::trace::metric_kv;
use alloc::vec::Vec;

/// Model package header with cryptographic verification
#[derive(Clone)]
pub struct ModelPackage {
    pub id: u32,
    pub version: u32,
    pub size_bytes: u32,
    pub sha256_hash: [u8; 32],
    pub ed25519_signature: [u8; 64],
    pub permissions: ModelPermissions,
}

bitflags::bitflags! {
    #[derive(Clone)]
    pub struct ModelPermissions: u32 {
        const LOAD = 0b0000_0001;        // Load model into memory
        const EXECUTE = 0b0000_0010;     // Execute model inference
        const INSPECT = 0b0000_0100;     // Inspect model structure/weights
        const EXPORT = 0b0000_1000;      // Export model data
        const ATTEST = 0b0001_0000;      // Generate attestation reports
    }
}

/// Model execution constraints for deterministic scheduling
#[derive(Copy, Clone)]
pub struct ModelConstraints {
    pub memory_cap_bytes: u64,     // Maximum memory allocation
    pub compute_budget_ns: u64,    // Maximum execution time
    pub allowed_ops: u32,          // Bitmask of allowed operations
}

/// Audit log entry for model operations
#[derive(Clone)]
pub struct ModelAuditEntry {
    pub timestamp_ns: u64,
    pub model_id: u32,
    pub operation: ModelOperation,
    pub result: ModelResult,
    pub context_id: u32,
}

#[derive(Copy, Clone, Debug)]
pub enum ModelOperation {
    Load,
    Execute,
    Inspect,
    Export,
    Attest,
}

#[derive(Copy, Clone, Debug)]
pub enum ModelResult {
    Success,
    PermissionDenied,
    ConstraintViolation,
    SignatureFailure,
    HashMismatch,
}

/// Model security manager with audit logging
pub struct ModelSecurityManager<const MAX_MODELS: usize, const MAX_AUDIT_ENTRIES: usize> {
    loaded_models: [Option<ModelPackage>; MAX_MODELS],
    model_count: usize,
    audit_log: [Option<ModelAuditEntry>; MAX_AUDIT_ENTRIES],
    audit_count: usize,
    load_success_count: u32,
    load_failure_count: u32,
}

impl<const MAX_MODELS: usize, const MAX_AUDIT_ENTRIES: usize> ModelSecurityManager<MAX_MODELS, MAX_AUDIT_ENTRIES> {
    pub const fn new() -> Self {
        Self {
            loaded_models: [const { None }; MAX_MODELS],
            model_count: 0,
            audit_log: [const { None }; MAX_AUDIT_ENTRIES],
            audit_count: 0,
            load_success_count: 0,
            load_failure_count: 0,
        }
    }

    /// Verify model signature and hash
    fn verify_model(&self, package: &ModelPackage, data: &[u8]) -> bool {
        // Verify SHA-256 hash
        let computed_hash = self.sha256_hash(data);
        if computed_hash != package.sha256_hash {
            return false;
        }

        // In a real implementation, would verify Ed25519 signature
        // For Phase 2 demo, we'll simulate verification
        self.verify_ed25519_signature(&package.sha256_hash, &package.ed25519_signature)
    }

    /// SHA-256 hash implementation
    #[cfg(feature = "crypto-real")]
    fn sha256_hash(&self, data: &[u8]) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        let out = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&out[..]);
        hash
    }

    /// SHA-256 hash (demo implementation when crypto-real is off)
    #[cfg(not(feature = "crypto-real"))]
    fn sha256_hash(&self, data: &[u8]) -> [u8; 32] {
        // Demo checksum-based hash (keeps behavior consistent without real keys)
        let mut hash = [0u8; 32];
        let mut checksum: u64 = 0;
        for byte in data {
            checksum = checksum.wrapping_add(*byte as u64);
            checksum = checksum.wrapping_mul(31);
        }
        for i in 0..4 {
            let bytes = (checksum.wrapping_add(i as u64 * 1000)).to_le_bytes();
            hash[i*8..(i+1)*8].copy_from_slice(&bytes);
        }
        hash
    }

    /// Parse hex string to fixed-size byte array at runtime (no_std friendly)
    #[cfg(feature = "crypto-real")]
    fn parse_hex_fixed<const N: usize>(s: &str) -> Option<[u8; N]> {
        let hex = if let Some(rest) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) { rest } else { s };
        if hex.len() != N * 2 { return None; }
        let b = hex.as_bytes();
        let mut out = [0u8; N];
        let mut i = 0usize;
        while i < N { 
            let hi = b[i*2];
            let lo = b[i*2+1];
            let hn = match hi { b'0'..=b'9'=>hi-b'0', b'a'..=b'f'=>hi-b'a'+10, b'A'..=b'F'=>hi-b'A'+10, _=>0xFF };
            let ln = match lo { b'0'..=b'9'=>lo-b'0', b'a'..=b'f'=>lo-b'a'+10, b'A'..=b'F'=>lo-b'A'+10, _=>0xFF };
            if hn > 15 || ln > 15 { return None; }
            out[i] = (hn << 4) | ln; 
            i += 1; 
        }
        Some(out)
    }

    /// Read Ed25519 public key bytes from build-time env var `SIS_ED25519_PUBKEY`.
    /// Expects 64 hex chars (32 bytes), optionally with 0x prefix.
    #[cfg(feature = "crypto-real")]
    fn pubkey_from_env() -> Option<[u8; 32]> {
        if let Some(val) = option_env!("SIS_ED25519_PUBKEY") { Self::parse_hex_fixed::<32>(val) } else { None }
    }

    /// Ed25519 signature verification using a build-time configured public key (crypto-real)
    #[cfg(feature = "crypto-real")]
    fn verify_ed25519_signature(&self, hash: &[u8; 32], signature: &[u8; 64]) -> bool {
        use ed25519_dalek::{Signature, VerifyingKey};
        let pk = match Self::pubkey_from_env() { Some(pk) => pk, None => [0u8; 32] };
        match VerifyingKey::from_bytes(&pk) {
            Ok(vk) => {
                let sig = Signature::from_bytes(signature);
                vk.verify_strict(hash, &sig).is_ok()
            }
            _ => false,
        }
    }

    /// Ed25519 signature verification (demo accepts any signature when crypto-real is off)
    #[cfg(not(feature = "crypto-real"))]
    fn verify_ed25519_signature(&self, _hash: &[u8; 32], _signature: &[u8; 64]) -> bool { true }

    /// Load and verify a signed model package
    pub fn load_model(&mut self, package: ModelPackage, data: &[u8]) -> Result<u32, ModelResult> {
        let timestamp_ns = crate::graph::now_cycles(); // Use cycle counter as timestamp
        
        // Verify model signature and hash
        if !self.verify_model(&package, data) {
            self.log_audit(timestamp_ns, package.id, ModelOperation::Load, ModelResult::SignatureFailure);
            self.load_failure_count += 1;
            return Err(ModelResult::SignatureFailure);
        }

        // Check capacity
        if self.model_count >= MAX_MODELS {
            self.log_audit(timestamp_ns, package.id, ModelOperation::Load, ModelResult::ConstraintViolation);
            self.load_failure_count += 1;
            return Err(ModelResult::ConstraintViolation);
        }

        // Store model
        let model_idx = self.model_count;
        self.loaded_models[model_idx] = Some(package.clone());
        self.model_count += 1;

        // Log successful load
        self.log_audit(timestamp_ns, package.id, ModelOperation::Load, ModelResult::Success);
        self.load_success_count += 1;

        Ok(model_idx as u32)
    }

    /// Check model operation permissions
    pub fn check_permission(&self, model_idx: u32, operation: ModelOperation, capability: Capability) -> bool {
        if model_idx >= self.model_count as u32 {
            return false;
        }

        let model = match &self.loaded_models[model_idx as usize] {
            Some(m) => m,
            None => return false,
        };

        // Check capability type
        if !crate::cap::check(capability, CapRights::RUN, CapObjectKind::Model) {
            return false;
        }

        // Check model-specific permissions
        let required_permission = match operation {
            ModelOperation::Load => ModelPermissions::LOAD,
            ModelOperation::Execute => ModelPermissions::EXECUTE,
            ModelOperation::Inspect => ModelPermissions::INSPECT,
            ModelOperation::Export => ModelPermissions::EXPORT,
            ModelOperation::Attest => ModelPermissions::ATTEST,
        };

        model.permissions.contains(required_permission)
    }

    /// Execute model with constraint enforcement
    pub fn execute_model(&mut self, model_idx: u32, constraints: ModelConstraints, capability: Capability) -> ModelResult {
        let timestamp_ns = crate::graph::now_cycles();
        
        if model_idx >= self.model_count as u32 {
            return ModelResult::ConstraintViolation;
        }

        let model_id = self.loaded_models[model_idx as usize].as_ref().unwrap().id;

        // Check permissions
        if !self.check_permission(model_idx, ModelOperation::Execute, capability) {
            self.log_audit(timestamp_ns, model_id, ModelOperation::Execute, ModelResult::PermissionDenied);
            return ModelResult::PermissionDenied;
        }

        // Enforce constraints (simplified for demo)
        if constraints.memory_cap_bytes > 1024 * 1024 {  // 1MB limit for demo
            self.log_audit(timestamp_ns, model_id, ModelOperation::Execute, ModelResult::ConstraintViolation);
            return ModelResult::ConstraintViolation;
        }

        // Log successful execution
        self.log_audit(timestamp_ns, model_id, ModelOperation::Execute, ModelResult::Success);
        ModelResult::Success
    }

    /// Log audit entry
    fn log_audit(&mut self, timestamp_ns: u64, model_id: u32, operation: ModelOperation, result: ModelResult) {
        if self.audit_count < MAX_AUDIT_ENTRIES {
            self.audit_log[self.audit_count] = Some(ModelAuditEntry {
                timestamp_ns,
                model_id,
                operation,
                result,
                context_id: 0, // Could track calling context
            });
            self.audit_count += 1;
        }
    }

    /// Emit Phase 2 model security metrics
    pub fn emit_metrics(&self) {
        metric_kv("model_load_success", self.load_success_count as usize);
        metric_kv("model_load_fail", self.load_failure_count as usize);
        metric_kv("model_audit_entries", self.audit_count);
        metric_kv("models_loaded", self.model_count);
    }

    /// Get audit log statistics
    pub fn audit_stats(&self) -> (usize, usize, usize, usize) {
        let mut success_count = 0;
        let mut denied_count = 0;
        let mut violation_count = 0;
        let mut signature_fail_count = 0;

        for i in 0..self.audit_count {
            if let Some(entry) = &self.audit_log[i] {
                match entry.result {
                    ModelResult::Success => success_count += 1,
                    ModelResult::PermissionDenied => denied_count += 1,
                    ModelResult::ConstraintViolation => violation_count += 1,
                    ModelResult::SignatureFailure => signature_fail_count += 1,
                    _ => {}
                }
            }
        }

        (success_count, denied_count, violation_count, signature_fail_count)
    }
}

/// Create a demo model package for testing
pub fn create_demo_model() -> (ModelPackage, Vec<u8>) {
    let demo_data = b"demo_model_weights_data_placeholder";
    
    // Create demo package
    let package = ModelPackage {
        id: 1,
        version: 1,
        size_bytes: demo_data.len() as u32,
        sha256_hash: [0; 32], // Will be computed
        ed25519_signature: [0; 64], // Demo signature
        permissions: ModelPermissions::LOAD | ModelPermissions::EXECUTE,
    };

    (package, demo_data.to_vec())
}
