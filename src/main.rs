use axum::{
    extract::{Multipart, WebSocketUpgrade},
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tower_http::services::ServeDir;

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    #[serde(rename = "type")]
    msg_type: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sender_id: Option<String>,
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
    let client_id_clone = client_id.clone();
    let mut client_id_from_client: Option<String> = None;
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let axum::extract::ws::Message::Text(text) = msg {
                // Parse the incoming message
                if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(&text) {
                    match chat_msg.msg_type.as_str() {
                        "register" => {
                            // 客户端发送的注册消息，包含client_id
                            if let Some(cid) = chat_msg.sender_id {
                                client_id_from_client = Some(cid);
                            }
                        }
                        "message" => {
                            // 使用客户端发送的ID，如果没有则使用服务器生成的ID
                            let sender_id = client_id_from_client.clone().unwrap_or_else(|| client_id_clone.clone());
                            // Create a message with sender ID
                            let broadcast_msg = ChatMessage {
                                msg_type: "message".to_string(),
                                content: chat_msg.content,
                                sender_id: Some(sender_id),
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
    while let Some(field) = multipart.next_field().await.unwrap() {
        let file_name = field.file_name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        // Create shared_files directory if it doesn't exist
        tokio::fs::create_dir_all("shared_files").await.unwrap();

        // Save the file
        let path = format!("shared_files/{}", file_name);
        tokio::fs::write(&path, &data).await.unwrap();

        println!("Uploaded file: {} ({} bytes)", file_name, data.len());
    }

    Html("File uploaded successfully! <a href='/'>Go back</a>")
}
