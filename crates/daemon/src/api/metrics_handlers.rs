//! Metrics API handlers

use crate::metrics::store::{QueryResult, SeriesMetadata};
use crate::qemu::QemuSupervisor;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use utoipa::IntoParams;

use super::handlers::ErrorResponse;

/// Query parameters for streams endpoint
#[derive(Debug, Deserialize, IntoParams)]
pub struct StreamsQuery {
    /// Filter by name prefix (optional)
    #[serde(default)]
    pub prefix: Option<String>,
}

/// Query parameters for metrics query endpoint
#[derive(Debug, Deserialize, IntoParams)]
pub struct MetricsQuery {
    /// Metric name
    pub name: String,

    /// Start timestamp (ms)
    #[serde(default)]
    pub from: Option<i64>,

    /// End timestamp (ms)
    #[serde(default)]
    pub to: Option<i64>,

    /// Maximum number of points to return (100-5000, default 1000)
    #[serde(default = "default_max_points", rename = "maxPoints")]
    pub max_points: usize,
}

fn default_max_points() -> usize {
    1000
}

/// List all metric series with metadata
#[utoipa::path(
    get,
    path = "/api/v1/metrics/streams",
    params(StreamsQuery),
    responses(
        (status = 200, description = "List of metric series", body = Vec<SeriesMetadata>),
    ),
    tag = "metrics"
)]
pub async fn list_streams(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
    Query(query): Query<StreamsQuery>,
) -> Json<Vec<SeriesMetadata>> {
    let (supervisor, _) = &state;
    let metrics = supervisor.metrics();
    let mut series = metrics.list_series().await;

    // Filter by prefix if specified
    if let Some(prefix) = query.prefix {
        series.retain(|s| s.name.starts_with(&prefix));
    }

    Json(series)
}

/// Query a specific metric series
#[utoipa::path(
    get,
    path = "/api/v1/metrics/query",
    params(MetricsQuery),
    responses(
        (status = 200, description = "Metric data points", body = QueryResult),
        (status = 400, description = "Invalid query parameters", body = ErrorResponse),
        (status = 404, description = "Series not found", body = ErrorResponse),
    ),
    tag = "metrics"
)]
pub async fn query_series(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
    Query(mut query): Query<MetricsQuery>,
) -> Result<Json<QueryResult>, (StatusCode, Json<ErrorResponse>)> {
    let (supervisor, _) = &state;
    let metrics = supervisor.metrics();

    // Validate and sanitize maxPoints
    if query.max_points < 100 {
        query.max_points = 100;
    } else if query.max_points > 5000 {
        query.max_points = 5000;
    }

    // Default time range if not specified (last 5 minutes)
    let now_ms = chrono::Utc::now().timestamp_millis();
    let from = query.from.unwrap_or(now_ms - 5 * 60 * 1000);
    let to = query.to.unwrap_or(now_ms);

    // Validate time range
    if from >= to {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::with_type(
                StatusCode::BAD_REQUEST,
                "Invalid time range: 'from' must be less than 'to'".to_string(),
                Some("/errors/query-bad-range".to_string()),
            )),
        ));
    }

    // Query the series
    match metrics.query(&query.name, from, to, query.max_points).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("not found") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse::with_type(
                        StatusCode::NOT_FOUND,
                        format!("Metric series not found: {}", query.name),
                        Some("/errors/query-series-unknown".to_string()),
                    )),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::with_type(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        error_msg,
                        Some("/errors/internal".to_string()),
                    )),
                ))
            }
        }
    }
}
