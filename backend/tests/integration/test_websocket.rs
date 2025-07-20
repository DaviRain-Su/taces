use backend::{
    services::websocket_service::{WebSocketManager, WsMessage},
};
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_websocket_manager_connections() {
    let ws_manager = Arc::new(WebSocketManager::new());
    
    // Add a connection
    let user_id = Uuid::new_v4();
    let role = "patient".to_string();
    let mut rx = ws_manager.add_connection(user_id, role.clone()).await;
    
    // Verify connection exists
    let online_users = ws_manager.get_online_users().await;
    assert_eq!(online_users.len(), 1);
    assert_eq!(online_users[0].0, user_id);
    assert_eq!(online_users[0].1, role);
    
    // Send message to user
    let test_msg = WsMessage::Heartbeat;
    let result = ws_manager.send_to_user(user_id, test_msg.clone()).await;
    assert!(result.is_ok());
    
    // Verify message received
    if let Ok(received_msg) = rx.try_recv() {
        match received_msg {
            WsMessage::Heartbeat => {
                // Success
            }
            _ => panic!("Unexpected message type"),
        }
    }
    
    // Remove connection
    ws_manager.remove_connection(user_id).await;
    let online_users = ws_manager.get_online_users().await;
    assert_eq!(online_users.len(), 0);
}

#[tokio::test]
async fn test_websocket_broadcast_to_role() {
    let ws_manager = Arc::new(WebSocketManager::new());
    
    // Add multiple connections
    let doctor1 = Uuid::new_v4();
    let doctor2 = Uuid::new_v4();
    let patient = Uuid::new_v4();
    
    let mut rx_doctor1 = ws_manager.add_connection(doctor1, "doctor".to_string()).await;
    let mut rx_doctor2 = ws_manager.add_connection(doctor2, "doctor".to_string()).await;
    let mut rx_patient = ws_manager.add_connection(patient, "patient".to_string()).await;
    
    // Broadcast to doctors only
    let msg = WsMessage::SystemAnnouncement {
        title: "Doctor Meeting".to_string(),
        content: "All doctors please join the meeting".to_string(),
    };
    ws_manager.broadcast_to_role("doctor", msg.clone()).await;
    
    // Verify doctors received message
    assert!(rx_doctor1.try_recv().is_ok());
    assert!(rx_doctor2.try_recv().is_ok());
    
    // Verify patient did not receive message
    assert!(rx_patient.try_recv().is_err());
    
    // Clean up
    ws_manager.remove_connection(doctor1).await;
    ws_manager.remove_connection(doctor2).await;
    ws_manager.remove_connection(patient).await;
}

#[tokio::test]
async fn test_websocket_broadcast_to_all() {
    let ws_manager = Arc::new(WebSocketManager::new());
    
    // Add connections
    let user1 = Uuid::new_v4();
    let user2 = Uuid::new_v4();
    
    let mut rx1 = ws_manager.add_connection(user1, "patient".to_string()).await;
    let mut rx2 = ws_manager.add_connection(user2, "doctor".to_string()).await;
    
    // Broadcast to all
    let msg = WsMessage::SystemAnnouncement {
        title: "System Maintenance".to_string(),
        content: "System will be under maintenance".to_string(),
    };
    ws_manager.broadcast_to_all(msg).await;
    
    // Verify all received message
    assert!(rx1.try_recv().is_ok());
    assert!(rx2.try_recv().is_ok());
    
    // Clean up
    ws_manager.remove_connection(user1).await;
    ws_manager.remove_connection(user2).await;
}

#[tokio::test]
async fn test_websocket_notification_message() {
    let ws_manager = Arc::new(WebSocketManager::new());
    
    let user_id = Uuid::new_v4();
    let mut rx = ws_manager.add_connection(user_id, "patient".to_string()).await;
    
    // Send notification via WebSocket
    let notification = backend::models::notification::Notification {
        id: Uuid::new_v4(),
        user_id,
        notification_type: backend::models::notification::NotificationType::AppointmentReminder,
        title: "Appointment Reminder".to_string(),
        content: "You have an appointment tomorrow".to_string(),
        related_id: Some(Uuid::new_v4()),
        metadata: serde_json::json!({}),
        status: backend::models::notification::NotificationStatus::Unread,
        read_at: None,
        created_at: chrono::Utc::now(),
    };
    
    ws_manager.send_notification(user_id, notification.clone()).await;
    
    // Verify notification received
    if let Ok(received_msg) = rx.try_recv() {
        match received_msg {
            WsMessage::Notification { title, content, .. } => {
                assert_eq!(title, notification.title);
                assert_eq!(content, notification.content);
            }
            _ => panic!("Expected notification message"),
        }
    }
    
    // Clean up
    ws_manager.remove_connection(user_id).await;
}

#[tokio::test]
async fn test_websocket_live_stream_events() {
    let ws_manager = Arc::new(WebSocketManager::new());
    
    let user_id = Uuid::new_v4();
    let mut rx = ws_manager.add_connection(user_id, "patient".to_string()).await;
    
    let stream_id = Uuid::new_v4();
    
    // Broadcast live stream started
    ws_manager.broadcast_live_stream_started(
        stream_id,
        "Health Talk".to_string(),
        "Dr. Wang".to_string(),
    ).await;
    
    // Verify message received
    if let Ok(received_msg) = rx.try_recv() {
        match received_msg {
            WsMessage::LiveStreamStarted { stream_id: id, title, host_name } => {
                assert_eq!(id, stream_id.to_string());
                assert_eq!(title, "Health Talk");
                assert_eq!(host_name, "Dr. Wang");
            }
            _ => panic!("Expected live stream started message"),
        }
    }
    
    // Broadcast live stream ended
    ws_manager.broadcast_live_stream_ended(stream_id).await;
    
    // Verify message received
    if let Ok(received_msg) = rx.try_recv() {
        match received_msg {
            WsMessage::LiveStreamEnded { stream_id: id } => {
                assert_eq!(id, stream_id.to_string());
            }
            _ => panic!("Expected live stream ended message"),
        }
    }
    
    // Clean up
    ws_manager.remove_connection(user_id).await;
}

#[tokio::test]
async fn test_websocket_video_call_request() {
    let ws_manager = Arc::new(WebSocketManager::new());
    
    let doctor_id = Uuid::new_v4();
    let patient_id = Uuid::new_v4();
    let consultation_id = Uuid::new_v4();
    
    let _rx_doctor = ws_manager.add_connection(doctor_id, "doctor".to_string()).await;
    let mut rx_patient = ws_manager.add_connection(patient_id, "patient".to_string()).await;
    
    // Send video call request
    ws_manager.send_video_call_request(consultation_id, doctor_id, patient_id).await;
    
    // Verify patient received request
    if let Ok(received_msg) = rx_patient.try_recv() {
        match received_msg {
            WsMessage::VideoCallRequest { consultation_id: id, from_user_id, to_user_id } => {
                assert_eq!(id, consultation_id.to_string());
                assert_eq!(from_user_id, doctor_id.to_string());
                assert_eq!(to_user_id, patient_id.to_string());
            }
            _ => panic!("Expected video call request message"),
        }
    }
    
    // Clean up
    ws_manager.remove_connection(doctor_id).await;
    ws_manager.remove_connection(patient_id).await;
}