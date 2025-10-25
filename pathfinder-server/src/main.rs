mod transport;

use futures_util::StreamExt;
use editor_core::EditorHandler;
use shared_types::receiver::Receiver;
use shared_types::router::Request;
use shared_types::storage::NoStorage;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use transport::WebSocketSender;

async fn handle_connection(stream: TcpStream, addr: SocketAddr) {
    println!("New WebSocket connection: {}", addr);

    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Error during WebSocket handshake: {}", e);
            return;
        }
    };

    let (write, mut read) = ws_stream.split();
    let session_id = format!("ws-{}", addr);
    let handler = EditorHandler::<NoStorage>::new(None);
    let receiver = Receiver::new(session_id, handler, None::<NoStorage>);
    let ws_sender = WebSocketSender::new(write);

    while let Some(message) = read.next().await {
        match message {
            Ok(msg) => {
                if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
                    match serde_json::from_str::<Request>(&text) {
                        Ok(request) => {
                            let sender: Box<dyn shared_types::router::WireResponseSender> = Box::new(ws_sender.clone());
                            receiver.handle_request(request, sender);
                        }
                        Err(e) => {
                            eprintln!("Failed to parse request: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
        }
    }

    println!("WebSocket connection closed: {}", addr);
}

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:10810";
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    println!("WebSocket server listening on: ws://{}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, addr));
    }
}
