use crate::discovery::{DeviceInfo, SharedState};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub timestamp: i64,
    pub payload: MessagePayload,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePayload {
    pub content: String,
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_format() -> String {
    "plain".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub target_ip: String,
    pub target_port: u16,
    pub message: ChatMessage,
}

#[tauri::command]
pub async fn send_message(req: SendMessageRequest) -> Result<String, String> {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;

    let url = format!("ws://{}:{}/ws", req.target_ip, req.target_port);

    tracing::info!("Connecting to WebSocket: {}", url);

    let (mut ws_stream, _) = tokio_tungstenite::connect_async(&url)
        .await
        .map_err(|e| format!("WebSocket connect failed: {}", e))?;

    let json = serde_json::to_string(&req.message)
        .map_err(|e| format!("Serialize failed: {}", e))?;

    ws_stream
        .send(Message::Text(json.into()))
        .await
        .map_err(|e| format!("Send failed: {}", e))?;

    let response = tokio::time::timeout(std::time::Duration::from_secs(10), ws_stream.next())
        .await
        .map_err(|_| "Response timeout".to_string())?
        .ok_or("No response".to_string())?
        .map_err(|e| format!("Receive error: {}", e))?;

    let ack = match response {
        Message::Text(text) => text.to_string(),
        _ => return Err("Unexpected response type".to_string()),
    };

    tracing::info!("Received ACK: {}", ack);
    Ok(ack)
}

#[tauri::command]
pub async fn get_device_list(state: State<'_, SharedState>) -> Result<Vec<DeviceInfo>, String> {
    let devices = state.devices.read().await;
    Ok(devices.values().cloned().collect())
}

pub fn create_ack(msg_id: &str) -> ChatMessage {
    ChatMessage {
        id: uuid::Uuid::new_v4().to_string(),
        msg_type: "ack".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
        payload: MessagePayload {
            content: String::new(),
            format: "plain".to_string(),
        },
        ref_id: Some(msg_id.to_string()),
    }
}

pub fn create_chat_message(content: &str, format: &str) -> ChatMessage {
    ChatMessage {
        id: uuid::Uuid::new_v4().to_string(),
        msg_type: "text".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
        payload: MessagePayload {
            content: content.to_string(),
            format: format.to_string(),
        },
        ref_id: None,
    }
}
