//! # Phase 2: AI Governance & Multi-Agent Coordination
//!
//! This module implements production-grade AI governance for the SIS kernel,
//! coordinating multiple AI agents (from Phase 1) to work together harmoniously.
//!
//! ## Key Components
//!
//! - **Orchestrator** (`orchestrator.rs`): Central coordinator for all AI agents
//! - **Conflict Resolution** (`conflict.rs`): Priority-based conflict resolution
//! - **Deployment Manager** (`deployment.rs`): Enhanced phase management with auto-transitions
//!
//! ## Architecture
//!
//! ```text
//! ┌────────────────────────────────────────────────────┐
//! │       AI Governance Layer (Phase 2)                │
//! │  ┌──────────────┐  ┌──────────────┐               │
//! │  │ Orchestrator │  │   Conflict   │               │
//! │  │              │  │   Resolver   │               │
//! │  └──────────────┘  └──────────────┘               │
//! └────────────────────────────────────────────────────┘
//!          │                 │
//! ┌────────┼─────────────────┼──────────────────────────┐
//! │        │  Phase 1 AI Components (Existing)         │
//! │  ┌─────▼──────┐  ┌──────▼──────┐                  │
//! │  │   Crash    │  │ Transformer │                  │
//! │  │ Predictor  │  │  Scheduler  │                  │
//! │  └────────────┘  └─────────────┘                  │
//! └─────────────────────────────────────────────────────┘
//! ```
//!
//! ## EU AI Act Compliance
//!
//! - **Article 13**: All decisions are transparent and explainable
//! - **Article 14**: Human can override all AI decisions
//! - **Article 16**: Complete audit trail of all coordination decisions

pub mod orchestrator;
pub mod conflict;
pub mod deployment;

pub use orchestrator::{AgentOrchestrator, CoordinatedDecision, OrchestrationStats};
pub use conflict::{ConflictResolver, Conflict, Resolution};
pub use deployment::{DeploymentManager, PhaseId, PhaseTransition};
