use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
};
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;
use crate::AppState;

#[derive(Debug, Clone)]
pub struct WsConnection {
    pub user_id: Uuid,
    pub role: String,
    pub sender: broadcast::Sender<WsMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    // Authentication
    Auth { token: String },
    AuthSuccess { user_id: String, role: String },
    AuthError { message: String },
    
    // Notification events
    Notification {
        id: String,
        title: String,
        content: String,
        notification_type: String,
    },
    
    // Chat messages
    ChatMessage {
        id: String,
        sender_id: String,
        receiver_id: String,
        content: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    
    // Video consultation events
    VideoCallRequest {
        consultation_id: String,
        from_user_id: String,
        to_user_id: String,
    },
    VideoCallAccepted {
        consultation_id: String,
    },
    VideoCallRejected {
        consultation_id: String,
        reason: String,
    },
    VideoCallEnded {
        consultation_id: String,
    },
    
    // Live stream events
    LiveStreamStarted {
        stream_id: String,
        title: String,
        host_name: String,
    },
    LiveStreamEnded {
        stream_id: String,
    },
    LiveStreamViewerCount {
        stream_id: String,
        count: u32,
    },
    
    // System events
    Heartbeat,
    HeartbeatAck,
    Error { message: String },
    SystemAnnouncement { title: String, content: String },
}

pub struct WebSocketManager {
    connections: Arc<RwLock<HashMap<Uuid, WsConnection>>>,
    broadcast_tx: broadcast::Sender<(Uuid, WsMessage)>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(1024);
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
        }
    }
    
    pub async fn add_connection(&self, user_id: Uuid, role: String) -> broadcast::Receiver<WsMessage> {
        let (tx, rx) = broadcast::channel(256);
        let connection = WsConnection {
            user_id,
            role,
            sender: tx,
        };
        
        let mut connections = self.connections.write().await;
        connections.insert(user_id, connection);
        
        rx
    }
    
    pub async fn remove_connection(&self, user_id: Uuid) {
        let mut connections = self.connections.write().await;
        connections.remove(&user_id);
    }
    
    pub async fn send_to_user(&self, user_id: Uuid, message: WsMessage) -> Result<(), String> {
        let connections = self.connections.read().await;
        if let Some(connection) = connections.get(&user_id) {
            connection.sender.send(message)
                .map_err(|e| format!("Failed to send message: {}", e))?;
            Ok(())
        } else {
            Err("User not connected".to_string())
        }
    }
    
    pub async fn broadcast_to_role(&self, role: &str, message: WsMessage) {
        let connections = self.connections.read().await;
        for connection in connections.values() {
            if connection.role == role {
                let _ = connection.sender.send(message.clone());
            }
        }
    }
    
    pub async fn broadcast_to_all(&self, message: WsMessage) {
        let connections = self.connections.read().await;
        for connection in connections.values() {
            let _ = connection.sender.send(message.clone());
        }
    }
    
    pub async fn get_online_users(&self) -> Vec<(Uuid, String)> {
        let connections = self.connections.read().await;
        connections.iter()
            .map(|(id, conn)| (*id, conn.role.clone()))
            .collect()
    }
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(app_state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| websocket_connection(socket, app_state))
}

async fn websocket_connection(socket: WebSocket, app_state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    
    // Wait for authentication message
    let auth_msg = match receiver.next().await {
        Some(Ok(Message::Text(text))) => text,
        _ => {
            let _ = sender.send(Message::Text(
                serde_json::to_string(&WsMessage::AuthError {
                    message: "Expected authentication message".to_string()
                }).unwrap()
            )).await;
            return;
        }
    };
    
    // Parse auth message
    let auth_data: Result<WsMessage, _> = serde_json::from_str(&auth_msg);
    let token = match auth_data {
        Ok(WsMessage::Auth { token }) => token,
        _ => {
            let _ = sender.send(Message::Text(
                serde_json::to_string(&WsMessage::AuthError {
                    message: "Invalid authentication message".to_string()
                }).unwrap()
            )).await;
            return;
        }
    };
    
    // Validate token and get user info
    let user_info = match validate_ws_token(&app_state, &token).await {
        Ok(info) => info,
        Err(e) => {
            let _ = sender.send(Message::Text(
                serde_json::to_string(&WsMessage::AuthError {
                    message: format!("Authentication failed: {}", e)
                }).unwrap()
            )).await;
            return;
        }
    };
    
    // Send auth success
    let _ = sender.send(Message::Text(
        serde_json::to_string(&WsMessage::AuthSuccess {
            user_id: user_info.0.to_string(),
            role: user_info.1.clone(),
        }).unwrap()
    )).await;
    
    // Add connection to manager
    let ws_manager = app_state.ws_manager.clone();
    let mut rx = ws_manager.add_connection(user_info.0, user_info.1.clone()).await;
    
    // Spawn task to handle incoming messages
    let user_id = user_info.0;
    let ws_manager_clone = ws_manager.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                        handle_ws_message(ws_msg, user_id, &ws_manager_clone).await;
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });
    
    // Send messages to client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Ok(text) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(text)).await.is_err() {
                    break;
                }
            }
        }
    });
    
    // Wait for either task to complete
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
    
    // Remove connection
    ws_manager.remove_connection(user_id).await;
}

async fn validate_ws_token(app_state: &AppState, token: &str) -> Result<(Uuid, String), String> {
    use crate::utils::jwt::decode_token;
    
    match decode_token(token, &app_state.config.jwt_secret) {
        Ok(claims) => Ok((claims.sub, claims.role)),
        Err(e) => Err(format!("Invalid token: {}", e)),
    }
}

async fn handle_ws_message(msg: WsMessage, user_id: Uuid, ws_manager: &WebSocketManager) {
    match msg {
        WsMessage::Heartbeat => {
            let _ = ws_manager.send_to_user(user_id, WsMessage::HeartbeatAck).await;
        }
        WsMessage::ChatMessage { receiver_id, content, .. } => {
            if let Ok(receiver_uuid) = Uuid::parse_str(&receiver_id) {
                let chat_msg = WsMessage::ChatMessage {
                    id: Uuid::new_v4().to_string(),
                    sender_id: user_id.to_string(),
                    receiver_id: receiver_id.clone(),
                    content,
                    timestamp: chrono::Utc::now(),
                };
                
                // Send to receiver
                let _ = ws_manager.send_to_user(receiver_uuid, chat_msg.clone()).await;
                
                // Echo back to sender
                let _ = ws_manager.send_to_user(user_id, chat_msg).await;
            }
        }
        _ => {
            // Handle other message types as needed
        }
    }
}

// Helper functions for sending specific types of messages

impl WebSocketManager {
    pub async fn send_notification(&self, user_id: Uuid, notification: crate::models::notification::Notification) {
        let msg = WsMessage::Notification {
            id: notification.id.to_string(),
            title: notification.title,
            content: notification.content,
            notification_type: format!("{:?}", notification.notification_type),
        };
        let _ = self.send_to_user(user_id, msg).await;
    }
    
    pub async fn broadcast_live_stream_started(&self, stream_id: Uuid, title: String, host_name: String) {
        let msg = WsMessage::LiveStreamStarted {
            stream_id: stream_id.to_string(),
            title,
            host_name,
        };
        self.broadcast_to_all(msg).await;
    }
    
    pub async fn broadcast_live_stream_ended(&self, stream_id: Uuid) {
        let msg = WsMessage::LiveStreamEnded {
            stream_id: stream_id.to_string(),
        };
        self.broadcast_to_all(msg).await;
    }
    
    pub async fn send_video_call_request(&self, consultation_id: Uuid, from_user_id: Uuid, to_user_id: Uuid) {
        let msg = WsMessage::VideoCallRequest {
            consultation_id: consultation_id.to_string(),
            from_user_id: from_user_id.to_string(),
            to_user_id: to_user_id.to_string(),
        };
        let _ = self.send_to_user(to_user_id, msg).await;
    }
}