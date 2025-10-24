use futures_util::SinkExt;
use serde_json;
use shared_types::router::{WireResponse, WireResponseSender};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;

#[derive(Clone)]
pub struct WebSocketSender {
    sender: Arc<Mutex<futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>, Message>>>,
}

impl WebSocketSender {
    pub fn new(
        sender: futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>, Message>,
    ) -> Self {
        Self {
            sender: Arc::new(Mutex::new(sender)),
        }
    }
}

impl WireResponseSender for WebSocketSender {
    fn send_response(&self, response: WireResponse) {
        let sender = Arc::clone(&self.sender);
        tokio::spawn(async move {
            let json = serde_json::to_string(&response).unwrap_or_else(|e| {
                format!(r#"[0,{{"Error":"Failed to serialize response: {}"}}]"#, e)
            });
            
            let mut sender = sender.lock().await;
            if let Err(e) = sender.send(Message::Text(json)).await {
                eprintln!("Failed to send WebSocket message: {}", e);
            }
        });
    }
}
