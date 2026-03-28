use axum::{
    extract::{Multipart, WebSocketUpgrade},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tower_http::services::ServeDir;

/// 接收消息
#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    #[serde(rename = "type")]
    msg_type: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sender_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_type: Option<String>,
}

#[tokio::main]
async fn main() {
    // Create a broadcast channel for messages
    let (tx, _rx) = broadcast::channel::<String>(100);

    // Create shared state for connected clients
    let clients = Arc::new(Mutex::new(HashMap::new()));

    // Build the application router
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/upload", post(upload_handler))
        .nest_service("/files", ServeDir::new("shared_files"))
        .nest_service("/", ServeDir::new("static"))
        .with_state((tx, clients));

    // Run the server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server running on http://{}", addr);
    println!("File sharing available at http://{}/files", addr);
    println!("WebSocket chat available at ws://{}/ws", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::State((tx, clients)): axum::extract::State<(
        broadcast::Sender<String>,
        Arc<Mutex<HashMap<String, String>>>,
    )>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, tx, clients))
}

async fn handle_socket(
    socket: axum::extract::ws::WebSocket,
    tx: broadcast::Sender<String>,
    clients: Arc<Mutex<HashMap<String, String>>>,
) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = tx.subscribe();
    let client_id = uuid::Uuid::new_v4().to_string();
    // Add client to the list
    {
        let mut clients_lock = clients.lock().unwrap();
        clients_lock.insert(client_id.clone(), String::from("connected"));
    }

    // Spawn a task to broadcast messages to this client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender
                .send(axum::extract::ws::Message::Text(msg))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    // Spawn a task to receive messages from this client
    let tx2 = tx.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let axum::extract::ws::Message::Text(text) = msg {
                // Parse the incoming message
                if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(&text) {
                    match chat_msg.msg_type.as_str() {
                        "message" | "file" | "image" => {
                            // 直接使用消息中的 sender_id
                            let sender_id = chat_msg.sender_id.unwrap_or_default();
                            // Create a message with sender ID
                            let broadcast_msg = ChatMessage {
                                msg_type: chat_msg.msg_type,
                                content: chat_msg.content,
                                sender_id: Some(sender_id),
                                file_url: chat_msg.file_url,
                                file_name: chat_msg.file_name,
                                file_type: chat_msg.file_type,
                            };
                            let json = serde_json::to_string(&broadcast_msg).unwrap();
                            let _ = tx2.send(json);
                        }
                        _ => {}
                    }
                }
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    // Remove client from the list
    {
        let mut clients_lock = clients.lock().unwrap();
        clients_lock.remove(&client_id);
    }
}

async fn upload_handler(mut multipart: Multipart) -> impl IntoResponse {
    let mut uploaded_files = Vec::new();
    
    while let Some(field) = multipart.next_field().await.unwrap() {
        let file_name = field.file_name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        // Create shared_files directory if it doesn't exist
        tokio::fs::create_dir_all("shared_files").await.unwrap();

        // Generate unique filename to avoid conflicts
        let unique_name = format!("{}_{}", uuid::Uuid::new_v4(), file_name);
        let path = format!("shared_files/{}", unique_name);
        tokio::fs::write(&path, &data).await.unwrap();

        println!("Uploaded file: {} ({} bytes)", file_name, data.len());
        
        uploaded_files.push(serde_json::json!({
            "file_name": file_name,
            "file_url": format!("/files/{}", unique_name),
            "file_size": data.len()
        }));
    }

    // Return JSON response with file info
    axum::Json(serde_json::json!({
        "success": true,
        "files": uploaded_files
    }))
}
