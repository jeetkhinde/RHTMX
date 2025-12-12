//! WebSocket handler for real-time sync

use std::collections::HashSet;
use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use futures::{SinkExt, StreamExt};
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

use super::message::SyncMessage;
use crate::document::DocumentChange;
use crate::engine::MergeEngine;

/// WebSocket connection state
pub struct WebSocketState {
    pub engine: Arc<MergeEngine>,
    pub broadcast_rx: broadcast::Receiver<DocumentChange>,
}

impl WebSocketState {
    pub fn new(engine: Arc<MergeEngine>, broadcast_rx: broadcast::Receiver<DocumentChange>) -> Self {
        Self { engine, broadcast_rx }
    }
}

/// Axum WebSocket handler
pub async fn ws_handler(
    State(state): State<Arc<WebSocketState>>,
    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(|socket| handle_connection(socket, state))
}

/// Handle a WebSocket connection
async fn handle_connection(socket: WebSocket, state: Arc<WebSocketState>) {
    let (mut sender, mut receiver) = socket.split();

    // Connection state
    let connection_id = Uuid::new_v4().to_string();
    let mut subscribed_entities: HashSet<String> = HashSet::new();

    // Channel for sending responses
    let (response_tx, mut response_rx) = mpsc::channel::<SyncMessage>(100);

    // Subscribe to broadcast changes
    let mut broadcast_rx = state.engine.subscribe();

    tracing::info!("WebSocket connected: {}", connection_id);

    // Spawn task to send messages
    let send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                // Send direct responses
                Some(msg) = response_rx.recv() => {
                    let json = match serde_json::to_string(&msg) {
                        Ok(j) => j,
                        Err(e) => {
                            tracing::error!("Failed to serialize message: {}", e);
                            continue;
                        }
                    };

                    if sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }

                // Forward broadcast changes
                Ok(change) = broadcast_rx.recv() => {
                    let msg = SyncMessage::Change { change };
                    let json = match serde_json::to_string(&msg) {
                        Ok(j) => j,
                        Err(e) => {
                            tracing::error!("Failed to serialize change: {}", e);
                            continue;
                        }
                    };

                    if sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    // Handle incoming messages
    while let Some(result) = receiver.next().await {
        let msg = match result {
            Ok(Message::Text(text)) => text,
            Ok(Message::Binary(data)) => {
                // Handle binary messages (compressed or Automerge data)
                match String::from_utf8(data) {
                    Ok(s) => s,
                    Err(_) => {
                        tracing::warn!("Received non-UTF8 binary message");
                        continue;
                    }
                }
            }
            Ok(Message::Close(_)) => {
                tracing::info!("WebSocket closed: {}", connection_id);
                break;
            }
            Ok(_) => continue,
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
        };

        // Parse message
        let sync_msg: SyncMessage = match serde_json::from_str(&msg) {
            Ok(m) => m,
            Err(e) => {
                let _ = response_tx
                    .send(SyncMessage::error(format!("Invalid message: {}", e)))
                    .await;
                continue;
            }
        };

        // Handle message
        match handle_message(
            sync_msg,
            &state.engine,
            &mut subscribed_entities,
            &response_tx,
        )
        .await
        {
            Ok(()) => {}
            Err(e) => {
                tracing::error!("Error handling message: {}", e);
                let _ = response_tx.send(SyncMessage::error(e.to_string())).await;
            }
        }
    }

    // Clean up
    send_task.abort();
    tracing::info!("WebSocket disconnected: {}", connection_id);
}

/// Handle an incoming sync message
async fn handle_message(
    msg: SyncMessage,
    engine: &Arc<MergeEngine>,
    subscribed: &mut HashSet<String>,
    response_tx: &mpsc::Sender<SyncMessage>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match msg {
        SyncMessage::Subscribe { entities, sync_state } => {
            // Add to subscriptions
            for entity in &entities {
                subscribed.insert(entity.clone());
            }

            // Send confirmation
            response_tx
                .send(SyncMessage::Subscribed {
                    entities: entities.clone(),
                })
                .await?;

            // Send initial sync for each entity
            for entity in &entities {
                let heads = sync_state
                    .as_ref()
                    .and_then(|s| s.get(entity))
                    .cloned()
                    .unwrap_or_default();

                // Parse heads
                let change_heads: Vec<automerge::ChangeHash> = heads
                    .iter()
                    .filter_map(|h| h.parse().ok())
                    .collect();

                // Get changes since heads
                let update = engine.get_changes_since(entity, &change_heads)?;
                let new_heads: Vec<String> = engine
                    .get_heads(entity)?
                    .iter()
                    .map(|h| h.to_string())
                    .collect();

                let count = engine.count(entity).await?;

                response_tx
                    .send(SyncMessage::SyncResponse {
                        entity: entity.clone(),
                        update: BASE64.encode(&update),
                        heads: new_heads,
                        count,
                    })
                    .await?;
            }
        }

        SyncMessage::Unsubscribe { entities } => {
            for entity in entities {
                subscribed.remove(&entity);
            }
        }

        SyncMessage::SyncRequest { entity, heads } => {
            let change_heads: Vec<automerge::ChangeHash> = heads
                .iter()
                .filter_map(|h| h.parse().ok())
                .collect();

            let update = engine.get_changes_since(&entity, &change_heads)?;
            let new_heads: Vec<String> = engine
                .get_heads(&entity)?
                .iter()
                .map(|h| h.to_string())
                .collect();

            let count = engine.count(&entity).await?;

            response_tx
                .send(SyncMessage::SyncResponse {
                    entity,
                    update: BASE64.encode(&update),
                    heads: new_heads,
                    count,
                })
                .await?;
        }

        SyncMessage::Create {
            request_id,
            entity,
            id,
            data,
        } => {
            let entity_id = id.unwrap_or_else(|| Uuid::new_v4().to_string());

            match engine.create(&entity, &entity_id, data).await {
                Ok(result) => {
                    response_tx
                        .send(SyncMessage::ack_with_data(request_id, result))
                        .await?;
                }
                Err(e) => {
                    response_tx
                        .send(SyncMessage::ack_error(request_id, e.to_string()))
                        .await?;
                }
            }
        }

        SyncMessage::Update {
            request_id,
            entity,
            id,
            data,
        } => {
            match engine.update(&entity, &id, data).await {
                Ok(result) => {
                    response_tx
                        .send(SyncMessage::ack_with_data(request_id, result))
                        .await?;
                }
                Err(e) => {
                    response_tx
                        .send(SyncMessage::ack_error(request_id, e.to_string()))
                        .await?;
                }
            }
        }

        SyncMessage::UpdateField {
            request_id,
            entity,
            id,
            field,
            value,
        } => {
            match engine.update_field(&entity, &id, &field, value).await {
                Ok(result) => {
                    response_tx
                        .send(SyncMessage::ack_with_data(request_id, result))
                        .await?;
                }
                Err(e) => {
                    response_tx
                        .send(SyncMessage::ack_error(request_id, e.to_string()))
                        .await?;
                }
            }
        }

        SyncMessage::Delete {
            request_id,
            entity,
            id,
        } => {
            match engine.delete(&entity, &id).await {
                Ok(deleted) => {
                    if deleted {
                        response_tx
                            .send(SyncMessage::ack(request_id, true))
                            .await?;
                    } else {
                        response_tx
                            .send(SyncMessage::ack_error(request_id, "Not found"))
                            .await?;
                    }
                }
                Err(e) => {
                    response_tx
                        .send(SyncMessage::ack_error(request_id, e.to_string()))
                        .await?;
                }
            }
        }

        SyncMessage::BinarySync { entity, data } => {
            // Decode base64 and apply changes
            let bytes = BASE64.decode(&data)?;
            engine.apply_changes(&entity, &bytes).await?;

            // Send updated state back
            let new_heads: Vec<String> = engine
                .get_heads(&entity)?
                .iter()
                .map(|h| h.to_string())
                .collect();

            response_tx
                .send(SyncMessage::BinaryState {
                    entity,
                    data: BASE64.encode(&bytes), // Echo back for confirmation
                    heads: new_heads,
                })
                .await?;
        }

        SyncMessage::Ping { timestamp } => {
            response_tx
                .send(SyncMessage::Pong { timestamp })
                .await?;
        }

        _ => {
            tracing::warn!("Unexpected message type from client");
        }
    }

    Ok(())
}
