use axum::{
    extract::{DefaultBodyLimit, Multipart, WebSocketUpgrade},
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
#[derive(Debug, Serialize, Deserialize, Clone)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<String>,
}

/// 用户数量消息
#[derive(Debug, Serialize, Deserialize)]
struct UserCountMessage {
    #[serde(rename = "type")]
    msg_type: String,
    count: usize,
}

/// 历史消息
#[derive(Debug, Serialize, Deserialize)]
struct HistoryMessage {
    #[serde(rename = "type")]
    msg_type: String,
    messages: Vec<ChatMessage>,
}

#[tokio::main]
async fn main() {
    // Create a broadcast channel for messages
    let (tx, _rx) = broadcast::channel::<String>(100);

    // Create shared state for connected clients
    let clients: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));

    // Create shared state for message history
    let message_history: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    // Build the application router
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/upload", post(upload_handler))
        .nest_service("/files", ServeDir::new("shared_files"))
        .nest_service("/", ServeDir::new("static"))
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024)) // 设置最大100MB
        .with_state((tx, clients, message_history));

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
    axum::extract::State((tx, clients, message_history)): axum::extract::State<(
        broadcast::Sender<String>,
        Arc<Mutex<HashMap<String, String>>>,
        Arc<Mutex<Vec<String>>>,
    )>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, tx, clients, message_history))
}

async fn handle_socket(
    socket: axum::extract::ws::WebSocket,
    tx: broadcast::Sender<String>,
    clients: Arc<Mutex<HashMap<String, String>>>,
    message_history: Arc<Mutex<Vec<String>>>,
) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = tx.subscribe();
    let client_id = uuid::Uuid::new_v4().to_string();

    // Add client to the list
    {
        let mut clients_lock = clients.lock().unwrap();
        clients_lock.insert(client_id.clone(), String::from("connected"));
    }

    // Send message history to new client
    let history_json = {
        let history = message_history.lock().unwrap();
        if !history.is_empty() {
            let history_msg = HistoryMessage {
                msg_type: "history".to_string(),
                messages: history
                    .iter()
                    .filter_map(|msg| serde_json::from_str(msg).ok())
                    .collect(),
            };
            Some(serde_json::to_string(&history_msg).unwrap())
        } else {
            None
        }
    };

    if let Some(json) = history_json {
        let _ = sender.send(axum::extract::ws::Message::Text(json)).await;
    }

    // Send current user count to new client
    let user_count = {
        let clients_lock = clients.lock().unwrap();
        clients_lock.len()
    };
    let count_msg = UserCountMessage {
        msg_type: "user_count".to_string(),
        count: user_count,
    };
    let count_json = serde_json::to_string(&count_msg).unwrap();
    let _ = sender
        .send(axum::extract::ws::Message::Text(count_json.clone()))
        .await;

    // Broadcast user count to all clients
    let _ = tx.send(count_json);

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
    let history_clone = message_history.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let axum::extract::ws::Message::Text(text) = msg {
                // Parse the incoming message
                if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(&text) {
                    match chat_msg.msg_type.as_str() {
                        "message" | "file" | "image" => {
                            // 添加时间戳
                            let timestamp =
                                chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                            let sender_id = chat_msg.sender_id.unwrap_or_default();

                            // Create a message with sender ID and timestamp
                            let broadcast_msg = ChatMessage {
                                msg_type: chat_msg.msg_type,
                                content: chat_msg.content,
                                sender_id: Some(sender_id),
                                file_url: chat_msg.file_url,
                                file_name: chat_msg.file_name,
                                file_type: chat_msg.file_type,
                                timestamp: Some(timestamp),
                            };
                            let json = serde_json::to_string(&broadcast_msg).unwrap();

                            // Save to history
                            {
                                let mut history = history_clone.lock().unwrap();
                                history.push(json.clone());
                                // 限制历史记录数量，最多保存100条
                                if history.len() > 100 {
                                    history.remove(0);
                                }
                            }

                            // Broadcast the message
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

    // Broadcast updated user count
    let user_count = {
        let clients_lock = clients.lock().unwrap();
        clients_lock.len()
    };
    let count_msg = UserCountMessage {
        msg_type: "user_count".to_string(),
        count: user_count,
    };
    let count_json = serde_json::to_string(&count_msg).unwrap();
    let _ = tx.send(count_json);
}

const MAX_FILE_SIZE: usize = 100 * 1024 * 1024; // 100MB

async fn upload_handler(mut multipart: Multipart) -> impl IntoResponse {
    let mut uploaded_files = Vec::new();
    let mut errors = Vec::new();

    while let Some(field_result) = multipart.next_field().await.transpose() {
        match field_result {
            Ok(field) => {
                // 获取文件名，如果没有则跳过
                let file_name = match field.file_name() {
                    Some(name) => name.to_string(),
                    None => continue,
                };

                // 读取文件数据
                let data = match field.bytes().await {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        // 如果是 multipart 解析错误，说明文件太大
                        if e.to_string().contains("multipart/form-data") {
                            let error_msg =
                                format!("文件太大，超过{}MB限制", MAX_FILE_SIZE / (1024 * 1024));
                            println!("{}", error_msg);
                            errors.push(error_msg);
                            break;
                        }
                        let error_msg = format!("文件 '{}' 读取失败: {}", file_name, e);
                        println!("{}", error_msg);
                        errors.push(error_msg);
                        continue;
                    }
                };

                // 检查文件大小
                if data.len() > MAX_FILE_SIZE {
                    let error_msg = format!(
                        "文件 '{}' 太大: {:.2}MB，最大允许: {}MB",
                        file_name,
                        data.len() as f64 / (1024.0 * 1024.0),
                        MAX_FILE_SIZE / (1024 * 1024)
                    );
                    println!("{}", error_msg);
                    errors.push(error_msg);
                    continue;
                }

                // Create shared_files directory if it doesn't exist
                if let Err(e) = tokio::fs::create_dir_all("shared_files").await {
                    let error_msg = format!("创建目录失败: {}", e);
                    println!("{}", error_msg);
                    errors.push(error_msg);
                    continue;
                }

                // Generate unique filename to avoid conflicts
                let unique_name = format!("{}_{}", uuid::Uuid::new_v4(), file_name);
                let path = format!("shared_files/{}", unique_name);

                // 保存文件
                match tokio::fs::write(&path, &data).await {
                    Ok(_) => {
                        println!("Uploaded file: {} ({} bytes)", file_name, data.len());

                        uploaded_files.push(serde_json::json!({
                            "file_name": file_name,
                            "file_url": format!("/files/{}", unique_name),
                            "file_size": data.len()
                        }));
                    }
                    Err(e) => {
                        let error_msg = format!("文件 '{}' 保存失败: {}", file_name, e);
                        println!("{}", error_msg);
                        errors.push(error_msg);
                        continue;
                    }
                }
            }
            Err(e) => {
                let error_msg = if e.to_string().contains("multipart/form-data") {
                    format!("文件太大，超过{}MB限制", MAX_FILE_SIZE / (1024 * 1024))
                } else {
                    format!("读取上传数据失败: {}", e)
                };
                println!("{}", error_msg);
                errors.push(error_msg);
                break;
            }
        }
    }

    // Return JSON response with file info and errors
    axum::Json(serde_json::json!({
        "success": uploaded_files.len() > 0,
        "files": uploaded_files,
        "errors": errors
    }))
}
