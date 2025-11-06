//! API middleware

use axum::{
    body::Body,
    extract::Request,
    http::{header, HeaderValue},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

/// X-Request-Id header name
pub const X_REQUEST_ID: &str = "X-Request-Id";

/// Middleware to handle X-Request-Id
///
/// - Accepts inbound X-Request-Id or generates UUIDv4 if absent
/// - Attaches to tracing span
/// - Echoes in response header
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    // Get or generate request ID
    let request_id = request
        .headers()
        .get(X_REQUEST_ID)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Attach to current span
    tracing::Span::current().record("request_id", &request_id);

    // Store in request extensions for downstream use
    request.extensions_mut().insert(request_id.clone());

    // Call next middleware/handler
    let mut response = next.run(request).await;

    // Echo request ID in response header
    if let Ok(header_value) = HeaderValue::from_str(&request_id) {
        response
            .headers_mut()
            .insert(header::HeaderName::from_static("x-request-id"), header_value);
    }

    response
}
