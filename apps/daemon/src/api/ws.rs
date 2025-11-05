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
    State(supervisor): State<Arc<QemuSupervisor>>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, supervisor))
}

/// Handle WebSocket connection
async fn handle_socket(mut socket: WebSocket, supervisor: Arc<QemuSupervisor>) {
    info!("New WebSocket client connected");

    // Subscribe to events
    let mut rx = supervisor.subscribe();

    // Event streaming loop
    loop {
        tokio::select! {
            // Receive event from supervisor
            event = rx.recv() => {
                match event {
                    Ok(event) => {
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
                    Err(e) => {
                        error!("Event channel error: {}", e);
                        break;
                    }
                }
            }

            // Receive message from client (for keepalive/ping)
            msg = socket.recv() => {
                match msg {
                    Some(Ok(msg)) => {
                        if msg.is_close() {
                            debug!("Client closed connection");
                            break;
                        }
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
