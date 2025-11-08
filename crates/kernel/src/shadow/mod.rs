//! Phase 7: Shadow Agent and Canary Deployment
//!
//! This module provides shadow/canary deployment capabilities:
//! - Shadow agent runs parallel predictions
//! - Comparison logic detects divergence
//! - Automatic rollback on excessive divergence
//! - Canary modes (10%, 100% traffic)

#[cfg(feature = "shadow-mode")]
pub mod agent;

#[cfg(feature = "shadow-mode")]
pub mod compare;

#[cfg(feature = "shadow-mode")]
pub mod rollback;

#[cfg(feature = "shadow-mode")]
pub use agent::{ShadowAgent, ShadowMode, ShadowStats};

#[cfg(feature = "shadow-mode")]
pub use compare::ComparisonResult;

#[cfg(feature = "shadow-mode")]
pub use rollback::{RollbackTrigger, RollbackDecision};
