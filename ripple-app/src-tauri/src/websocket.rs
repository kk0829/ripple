use crate::messages::{create_ack, ChatMessage};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;

pub async fn start_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind("0.0.0.0:9700").await?;
    tracing::info!("WebSocket server listening on ws://0.0.0.0:9700");

    loop {
        let (stream, addr) = listener.accept().await?;
        tracing::info!("New WebSocket connection from {}", addr);
        tokio::spawn(handle_connection(stream));
    }
}

async fn handle_connection(stream: tokio::net::TcpStream) {
    let ws_stream = tokio_tungstenite::accept_async(stream).await;

    match ws_stream {
        Ok(ws_stream) => {
            let (mut write, mut read) = ws_stream.split();

            while let Some(msg_result) = read.next().await {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        tracing::info!("Received message: {}", text);

                        match serde_json::from_str::<ChatMessage>(&text) {
                            Ok(chat_msg) => {
                                if chat_msg.msg_type == "ack" {
                                    tracing::info!("ACK received for ref: {:?}", chat_msg.ref_id);
                                    continue;
                                }

                                tracing::info!(
                                    "Message from peer: [{}] {}",
                                    chat_msg.msg_type,
                                    chat_msg.payload.content
                                );

                                let ack = create_ack(&chat_msg.id);
                                let ack_json =
                                    serde_json::to_string(&ack).unwrap_or_else(|_| "{}".to_string());

                                if let Err(e) = write.send(Message::Text(ack_json.into())).await {
                                    tracing::error!("Failed to send ACK: {}", e);
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Failed to parse message: {}", e);
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        tracing::info!("WebSocket connection closed");
                        break;
                    }
                    Ok(Message::Ping(data)) => {
                        let _ = write.send(Message::Pong(data)).await;
                    }
                    Err(e) => {
                        tracing::error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        }
        Err(e) => {
            tracing::error!("WebSocket handshake failed: {}", e);
        }
    }
}
