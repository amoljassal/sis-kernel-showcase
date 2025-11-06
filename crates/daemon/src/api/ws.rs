//! WebSocket event streaming

use crate::qemu::{QemuEvent, QemuSupervisor};
use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    response::Response,
};
use std::sync::Arc;
use tracing::{debug, error, info};

/// WebSocket upgrade handler
pub async fn events_handler(
    ws: WebSocketUpgrade,
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
) -> Response {
    let (supervisor, _) = &state;
    let supervisor_clone = Arc::clone(supervisor);
    ws.on_upgrade(|socket| handle_socket(socket, supervisor_clone))
}

/// Handle WebSocket connection
async fn handle_socket(mut socket: WebSocket, supervisor: Arc<QemuSupervisor>) {
    info!("New WebSocket client connected");

    // Subscribe to events
    let mut rx = supervisor.subscribe();
    let mut dropped_count: usize = 0;

    // Event streaming loop
    loop {
        tokio::select! {
            // Receive event from supervisor
            event = rx.recv() => {
                match event {
                    Ok(event) => {
                        // If we had dropped events, send a notification first
                        if dropped_count > 0 {
                            let dropped_event = serde_json::json!({
                                "type": "backpressure",
                                "droppedCount": dropped_count,
                                "ts": chrono::Utc::now().timestamp_millis(),
                            });
                            if let Ok(json) = serde_json::to_string(&dropped_event) {
                                let _ = socket.send(axum::extract::ws::Message::Text(json)).await;
                            }
                            dropped_count = 0;
                        }

                        // Serialize event to JSON
                        match serde_json::to_string(&event) {
                            Ok(json) => {
                                // Send to client
                                if let Err(e) = socket.send(axum::extract::ws::Message::Text(json)).await {
                                    error!("Failed to send event to client: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                error!("Failed to serialize event: {}", e);
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        // Client is lagging, accumulate dropped count
                        dropped_count += n as usize;
                        debug!("Client lagging, dropped {} events (total: {})", n, dropped_count);
                        // Continue processing, don't break
                    }
                    Err(e) => {
                        error!("Event channel error: {}", e);
                        break;
                    }
                }
            }

            // Receive message from client (for keepalive/ping)
            msg = socket.recv() => {
                match msg {
                    Some(Ok(axum::extract::ws::Message::Close(_))) => {
                        debug!("Client closed connection");
                        break;
                    }
                    Some(Ok(_)) => {
                        // Ignore other messages (we only stream events)
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    None => {
                        debug!("WebSocket closed");
                        break;
                    }
                }
            }
        }
    }

    info!("WebSocket client disconnected");
}
