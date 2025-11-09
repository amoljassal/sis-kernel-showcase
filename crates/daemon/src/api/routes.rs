//! API routing

use super::{
    autonomy_handlers, conflicts_handlers, crash_handlers, deployment_handlers, drift_handlers,
    graph_handlers, handlers, llm_handlers, logs_handlers, memory_handlers, metrics_handlers,
    middleware, orchestrator_handlers, replay_handlers, sched_handlers, shell_handlers,
    versions_handlers, ws,
};
use crate::qemu::{QemuSupervisor, ReplayManager};
use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::health,
        handlers::get_config,
        handlers::qemu_run,
        handlers::qemu_stop,
        handlers::qemu_status,
        shell_handlers::shell_exec,
        shell_handlers::shell_selfcheck,
        shell_handlers::shell_selfcheck_cancel,
        replay_handlers::replay_start,
        replay_handlers::replay_stop,
        replay_handlers::replay_status,
        metrics_handlers::list_streams,
        metrics_handlers::query_series,
        autonomy_handlers::autonomy_on,
        autonomy_handlers::autonomy_off,
        autonomy_handlers::autonomy_reset,
        autonomy_handlers::autonomy_set_interval,
        autonomy_handlers::autonomy_set_threshold,
        autonomy_handlers::autonomy_status,
        autonomy_handlers::autonomy_audit,
        autonomy_handlers::autonomy_explain,
        autonomy_handlers::autonomy_preview,
        autonomy_handlers::autonomy_whatif,
        memory_handlers::mem_get_approvals,
        memory_handlers::mem_approval_toggle,
        memory_handlers::mem_approve,
        memory_handlers::mem_reject,
        graph_handlers::graph_create,
        graph_handlers::graph_add_channel,
        graph_handlers::graph_add_operator,
        graph_handlers::graph_start,
        graph_handlers::graph_predict,
        graph_handlers::graph_feedback,
        graph_handlers::graph_state,
        graph_handlers::graph_export,
        sched_handlers::sched_workloads,
        sched_handlers::sched_set_priority,
        sched_handlers::sched_set_affinity,
        sched_handlers::sched_set_feature,
        sched_handlers::sched_circuit_breaker_status,
        sched_handlers::sched_circuit_breaker_reset,
        llm_handlers::llm_load,
        llm_handlers::llm_infer,
        llm_handlers::llm_audit,
        llm_handlers::llm_status,
        logs_handlers::logs_tail,
        logs_handlers::runs_start,
        logs_handlers::runs_stop,
        logs_handlers::runs_list,
        logs_handlers::runs_export,
        crash_handlers::crash_ingest,
        crash_handlers::crash_list,
        crash_handlers::incident_create,
        crash_handlers::incident_list,
        // Phase 2: Orchestrator
        orchestrator_handlers::get_orchestrator_stats,
        orchestrator_handlers::get_orchestrator_decisions,
        orchestrator_handlers::get_orchestrator_agents,
        // Phase 2: Conflicts
        conflicts_handlers::get_conflict_stats,
        conflicts_handlers::get_conflict_history,
        conflicts_handlers::get_priority_table,
        // Phase 2: Deployment
        deployment_handlers::get_deployment_status,
        deployment_handlers::get_deployment_history,
        deployment_handlers::advance_deployment,
        deployment_handlers::rollback_deployment,
        deployment_handlers::update_deployment_config,
        // Phase 2: Drift
        drift_handlers::get_drift_status,
        drift_handlers::get_drift_history,
        drift_handlers::trigger_retrain,
        drift_handlers::reset_baseline,
        // Phase 2: Versions
        versions_handlers::get_version_list,
        versions_handlers::commit_version,
        versions_handlers::rollback_version,
        versions_handlers::get_version_diff,
        versions_handlers::tag_version,
        versions_handlers::garbage_collect_versions,
    ),
    components(
        schemas(
            crate::config::DaemonConfig,
            crate::qemu::QemuConfig,
            crate::qemu::QemuStatus,
            crate::qemu::QemuState,
            crate::qemu::ReplayStatus,
            crate::qemu::ReplayState,
            crate::qemu::ShellCommandRequest,
            crate::qemu::ShellCommandResponse,
            crate::qemu::SelfCheckResponse,
            crate::qemu::TestResultEntry,
            crate::metrics::MetricPoint,
            crate::metrics::SeriesStats,
            crate::metrics::store::SeriesMetadata,
            crate::metrics::store::QueryResult,
            handlers::ErrorResponse,
            handlers::SuccessResponse,
            handlers::HealthResponse,
            replay_handlers::ReplayRequest,
            replay_handlers::ReplayResponse,
            autonomy_handlers::AutonomyStatus,
            autonomy_handlers::AutonomyDecision,
            autonomy_handlers::ExplainResponse,
            autonomy_handlers::AttentionWeight,
            autonomy_handlers::PreviewRequest,
            autonomy_handlers::PreviewResponse,
            autonomy_handlers::WhatIfRequest,
            autonomy_handlers::WhatIfResponse,
            memory_handlers::MemoryApprovalStatus,
            memory_handlers::PendingOperation,
            memory_handlers::ApproveRequest,
            memory_handlers::RejectRequest,
            memory_handlers::ApprovalToggleRequest,
            graph_handlers::CreateGraphResponse,
            graph_handlers::AddChannelRequest,
            graph_handlers::AddChannelResponse,
            graph_handlers::AddOperatorRequest,
            graph_handlers::AddOperatorResponse,
            graph_handlers::StartGraphRequest,
            graph_handlers::StartGraphResponse,
            graph_handlers::PredictRequest,
            graph_handlers::PredictResponse,
            graph_handlers::FeedbackRequest,
            graph_handlers::FeedbackResponse,
            graph_handlers::GraphState,
            graph_handlers::GraphOperator,
            graph_handlers::GraphChannel,
            graph_handlers::GraphStats,
            graph_handlers::ExportGraphRequest,
            graph_handlers::ExportGraphResponse,
            sched_handlers::Workload,
            sched_handlers::SetPriorityRequest,
            sched_handlers::SetAffinityRequest,
            sched_handlers::SetFeatureRequest,
            sched_handlers::SchedResponse,
            sched_handlers::CircuitBreakerState,
            llm_handlers::LoadModelRequest,
            llm_handlers::LoadModelResponse,
            llm_handlers::InferRequest,
            llm_handlers::InferResponse,
            llm_handlers::AuditEntry,
            llm_handlers::LlmStatus,
            logs_handlers::LogEntry,
            logs_handlers::RunProfile,
            logs_handlers::StartRunRequest,
            logs_handlers::StartRunResponse,
            logs_handlers::StopRunResponse,
            logs_handlers::RunHistoryEntry,
            crash_handlers::CrashLog,
            crash_handlers::IngestCrashRequest,
            crash_handlers::IngestCrashResponse,
            crash_handlers::CrashListResponse,
            crash_handlers::Incident,
            crash_handlers::CreateIncidentRequest,
            crash_handlers::CreateIncidentResponse,
            crash_handlers::IncidentListResponse,
            // Phase 2: Orchestrator
            orchestrator_handlers::OrchestrationStats,
            orchestrator_handlers::CoordinatedDecision,
            orchestrator_handlers::AgentInfo,
            orchestrator_handlers::AgentLastDecision,
            orchestrator_handlers::AgentStats,
            orchestrator_handlers::DecisionsResponse,
            orchestrator_handlers::AgentsResponse,
            // Phase 2: Conflicts
            conflicts_handlers::ConflictStats,
            conflicts_handlers::ConflictAgent,
            conflicts_handlers::ConflictResolution,
            conflicts_handlers::Conflict,
            conflicts_handlers::ConflictsResponse,
            conflicts_handlers::PriorityEntry,
            conflicts_handlers::PriorityTableResponse,
            // Phase 2: Deployment
            deployment_handlers::CurrentPhase,
            deployment_handlers::DeploymentStatus,
            deployment_handlers::MetricsSnapshot,
            deployment_handlers::PhaseTransition,
            deployment_handlers::DeploymentHistoryResponse,
            deployment_handlers::AdvanceRequest,
            deployment_handlers::RollbackRequest,
            deployment_handlers::DeploymentConfigRequest,
            deployment_handlers::PhaseChangeResponse,
            deployment_handlers::ConfigUpdateResponse,
            deployment_handlers::DeploymentConfig,
            // Phase 2: Drift
            drift_handlers::DriftStatus,
            drift_handlers::DriftSample,
            drift_handlers::DriftHistoryResponse,
            drift_handlers::RetrainRequest,
            drift_handlers::RetrainResponse,
            drift_handlers::ResetBaselineResponse,
            // Phase 2: Versions
            versions_handlers::VersionMetadata,
            versions_handlers::AdapterVersion,
            versions_handlers::VersionListResponse,
            versions_handlers::VersionDiff,
            versions_handlers::CommitRequest,
            versions_handlers::CommitResponse,
            versions_handlers::VersionRollbackRequest,
            versions_handlers::VersionRollbackResponse,
            versions_handlers::TagRequest,
            versions_handlers::TagResponse,
            versions_handlers::GarbageCollectRequest,
            versions_handlers::GarbageCollectResponse,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "config", description = "Configuration endpoints"),
        (name = "qemu", description = "QEMU control endpoints"),
        (name = "shell", description = "Shell command execution"),
        (name = "replay", description = "Replay log files for offline testing"),
        (name = "metrics", description = "Metrics collection and querying"),
        (name = "autonomy", description = "Autonomy control and decision management"),
        (name = "memory", description = "Memory approval management"),
        (name = "graph", description = "Graph control and operator management"),
        (name = "scheduling", description = "Workload scheduling and feature management"),
        (name = "llm", description = "LLM model loading and inference (feature-gated)"),
        (name = "logs", description = "Log tailing and filtering"),
        (name = "runs", description = "Run history and troubleshooting"),
        (name = "crashes", description = "Crash capture and querying"),
        (name = "incidents", description = "Incident tracking and management"),
        (name = "orchestrator", description = "Phase 2: Multi-agent orchestration"),
        (name = "conflicts", description = "Phase 2: Conflict resolution"),
        (name = "deployment", description = "Phase 2: Deployment management"),
        (name = "drift", description = "Phase 2: Model drift detection"),
        (name = "versions", description = "Phase 2: Adapter version control")
    ),
    info(
        title = "SIS Kernel Control Daemon (sisctl)",
        version = "0.1.0",
        description = "REST API for managing SIS kernel QEMU instances",
    )
)]
struct ApiDoc;

/// Create the API router
pub fn create_router(supervisor: Arc<QemuSupervisor>, replay_manager: Arc<ReplayManager>) -> Router {
    // Create OpenAPI documentation
    let openapi = ApiDoc::openapi();

    // Shared state tuple for all handlers
    let state = (supervisor, replay_manager);

    Router::new()
        // Health check
        .route("/health", get(handlers::health))
        // Configuration
        .route("/api/v1/config", get(handlers::get_config))
        // QEMU control endpoints
        .route("/api/v1/qemu/run", post(handlers::qemu_run))
        .route("/api/v1/qemu/stop", post(handlers::qemu_stop))
        .route("/api/v1/qemu/status", get(handlers::qemu_status))
        // Shell command endpoints
        .route("/api/v1/shell/exec", post(shell_handlers::shell_exec))
        .route(
            "/api/v1/shell/selfcheck",
            post(shell_handlers::shell_selfcheck),
        )
        .route(
            "/api/v1/shell/selfcheck/cancel",
            post(shell_handlers::shell_selfcheck_cancel),
        )
        // Replay endpoints for offline testing
        .route("/api/v1/replay", post(replay_handlers::replay_start))
        .route("/api/v1/replay/stop", post(replay_handlers::replay_stop))
        .route("/api/v1/replay/status", get(replay_handlers::replay_status))
        // Metrics endpoints
        .route("/api/v1/metrics/streams", get(metrics_handlers::list_streams))
        .route("/api/v1/metrics/query", get(metrics_handlers::query_series))
        // Autonomy endpoints
        .route("/api/v1/autonomy/on", post(autonomy_handlers::autonomy_on))
        .route("/api/v1/autonomy/off", post(autonomy_handlers::autonomy_off))
        .route("/api/v1/autonomy/reset", post(autonomy_handlers::autonomy_reset))
        .route("/api/v1/autonomy/interval", post(autonomy_handlers::autonomy_set_interval))
        .route("/api/v1/autonomy/conf-threshold", post(autonomy_handlers::autonomy_set_threshold))
        .route("/api/v1/autonomy/status", get(autonomy_handlers::autonomy_status))
        .route("/api/v1/autonomy/audit", get(autonomy_handlers::autonomy_audit))
        .route("/api/v1/autonomy/explain", get(autonomy_handlers::autonomy_explain))
        .route("/api/v1/autonomy/preview", post(autonomy_handlers::autonomy_preview))
        .route("/api/v1/autonomy/whatif", post(autonomy_handlers::autonomy_whatif))
        // Memory approval endpoints
        .route("/api/v1/mem/approvals", get(memory_handlers::mem_get_approvals))
        .route("/api/v1/mem/approval", post(memory_handlers::mem_approval_toggle))
        .route("/api/v1/mem/approve", post(memory_handlers::mem_approve))
        .route("/api/v1/mem/reject", post(memory_handlers::mem_reject))
        // Graph endpoints
        .route("/api/v1/graph/create", post(graph_handlers::graph_create))
        .route("/api/v1/graph/add-channel", post(graph_handlers::graph_add_channel))
        .route("/api/v1/graph/add-operator", post(graph_handlers::graph_add_operator))
        .route("/api/v1/graph/start", post(graph_handlers::graph_start))
        .route("/api/v1/graph/predict", post(graph_handlers::graph_predict))
        .route("/api/v1/graph/feedback", post(graph_handlers::graph_feedback))
        .route("/api/v1/graph/state", get(graph_handlers::graph_state))
        .route("/api/v1/graph/export", post(graph_handlers::graph_export))
        // Scheduling endpoints
        .route("/api/v1/sched/workloads", get(sched_handlers::sched_workloads))
        .route("/api/v1/sched/priorities", post(sched_handlers::sched_set_priority))
        .route("/api/v1/sched/affinity", post(sched_handlers::sched_set_affinity))
        .route("/api/v1/sched/feature", post(sched_handlers::sched_set_feature))
        .route("/api/v1/sched/circuit-breaker", get(sched_handlers::sched_circuit_breaker_status))
        .route("/api/v1/sched/circuit-breaker/reset", post(sched_handlers::sched_circuit_breaker_reset))
        // LLM endpoints
        .route("/api/v1/llm/load", post(llm_handlers::llm_load))
        .route("/api/v1/llm/infer", post(llm_handlers::llm_infer))
        .route("/api/v1/llm/audit", get(llm_handlers::llm_audit))
        .route("/api/v1/llm/status", get(llm_handlers::llm_status))
        // Logs and troubleshooting endpoints
        .route("/api/v1/logs/tail", get(logs_handlers::logs_tail))
        .route("/api/v1/runs/start", post(logs_handlers::runs_start))
        .route("/api/v1/runs/stop", post(logs_handlers::runs_stop))
        .route("/api/v1/runs/list", get(logs_handlers::runs_list))
        .route("/api/v1/runs/:runId/export", get(logs_handlers::runs_export))
        // Crash capture endpoints
        .route("/api/v1/crash", post(crash_handlers::crash_ingest))
        .route("/api/v1/crashes", get(crash_handlers::crash_list))
        // Incident tracking endpoints
        .route("/api/v1/incidents", post(crash_handlers::incident_create))
        .route("/api/v1/incidents", get(crash_handlers::incident_list))
        // Phase 2: Orchestrator endpoints
        .route("/api/v1/orchestrator/stats", get(orchestrator_handlers::get_orchestrator_stats))
        .route("/api/v1/orchestrator/decisions", get(orchestrator_handlers::get_orchestrator_decisions))
        .route("/api/v1/orchestrator/agents", get(orchestrator_handlers::get_orchestrator_agents))
        // Phase 2: Conflicts endpoints
        .route("/api/v1/conflicts/stats", get(conflicts_handlers::get_conflict_stats))
        .route("/api/v1/conflicts/history", get(conflicts_handlers::get_conflict_history))
        .route("/api/v1/conflicts/priority-table", get(conflicts_handlers::get_priority_table))
        // Phase 2: Deployment endpoints
        .route("/api/v1/deployment/status", get(deployment_handlers::get_deployment_status))
        .route("/api/v1/deployment/history", get(deployment_handlers::get_deployment_history))
        .route("/api/v1/deployment/advance", post(deployment_handlers::advance_deployment))
        .route("/api/v1/deployment/rollback", post(deployment_handlers::rollback_deployment))
        .route("/api/v1/deployment/config", post(deployment_handlers::update_deployment_config))
        // Phase 2: Drift endpoints
        .route("/api/v1/drift/status", get(drift_handlers::get_drift_status))
        .route("/api/v1/drift/history", get(drift_handlers::get_drift_history))
        .route("/api/v1/drift/retrain", post(drift_handlers::trigger_retrain))
        .route("/api/v1/drift/reset-baseline", post(drift_handlers::reset_baseline))
        // Phase 2: Versions endpoints
        .route("/api/v1/versions/list", get(versions_handlers::get_version_list))
        .route("/api/v1/versions/commit", post(versions_handlers::commit_version))
        .route("/api/v1/versions/rollback", post(versions_handlers::rollback_version))
        .route("/api/v1/versions/diff", get(versions_handlers::get_version_diff))
        .route("/api/v1/versions/tag", post(versions_handlers::tag_version))
        .route("/api/v1/versions/gc", post(versions_handlers::garbage_collect_versions))
        // WebSocket events
        .route("/events", get(ws::events_handler))
        // State
        .with_state(state)
        // Swagger UI
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi))
        // Request ID middleware
        .layer(axum_middleware::from_fn(middleware::request_id_middleware))
        // CORS for local development
        .layer(CorsLayer::permissive())
}
