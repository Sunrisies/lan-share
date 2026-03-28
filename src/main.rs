use axum::{
    extract::{DefaultBodyLimit, Multipart, WebSocketUpgrade},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use chrono::Local;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
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

type AppState = (
    broadcast::Sender<String>,
    Arc<Mutex<HashMap<String, String>>>,
    Arc<Mutex<Vec<String>>>,
);
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

/// 配置文件
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Config {
    /// 自动清理时间（小时），默认24小时
    auto_clean_hours: u32,
    /// 上次清理时间
    last_clean_time: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auto_clean_hours: 24,
            last_clean_time: None,
        }
    }
}

#[tokio::main]
async fn main() {
    // Create a broadcast channel for messages
    let (tx, _rx) = broadcast::channel::<String>(100);

    // Create shared state for connected clients
    let clients: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));

    // Create shared state for message history
    let message_history: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    // 加载配置文件
    let config = load_config().await;
    let config: Arc<Mutex<Config>> = Arc::new(Mutex::new(config));

    // 启动定时清理任务
    let config_clone = config.clone();
    tokio::spawn(async move {
        start_auto_clean_task(config_clone).await;
    });

    // Build the application router
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/upload", post(upload_handler))
        .route("/api/config", get(get_config).post(update_config))
        .route("/api/clean", post(clean_files))
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

/// 加载配置文件
async fn load_config() -> Config {
    let config_path = "config.json";
    if let Ok(content) = tokio::fs::read_to_string(config_path).await {
        if let Ok(config) = serde_json::from_str::<Config>(&content) {
            return config;
        }
    }
    Config::default()
}

/// 保存配置文件
async fn save_config(config: &Config) {
    let config_path = "config.json";
    if let Ok(json) = serde_json::to_string_pretty(config) {
        let _ = tokio::fs::write(config_path, json).await;
    }
}

/// 定时清理任务
async fn start_auto_clean_task(config: Arc<Mutex<Config>>) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await; // 每分钟检查一次

        let should_clean = {
            let config_lock = config.lock().unwrap();
            if let Some(last_clean) = &config_lock.last_clean_time {
                if let Ok(last_time) =
                    chrono::NaiveDateTime::parse_from_str(last_clean, "%Y-%m-%d %H:%M:%S")
                {
                    let now = Local::now().naive_local();
                    let duration = now.signed_duration_since(last_time);
                    duration.num_hours() >= config_lock.auto_clean_hours as i64
                } else {
                    true
                }
            } else {
                true
            }
        };

        if should_clean {
            println!("执行自动清理...");
            clean_shared_files().await;

            // 更新上次清理时间
            let config_to_save = {
                let mut config_lock = config.lock().unwrap();
                config_lock.last_clean_time =
                    Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
                config_lock.clone()
            };
            save_config(&config_to_save).await;
        }
    }
}

/// 清理共享文件目录
async fn clean_shared_files() {
    let dir = "shared_files";
    if let Ok(mut entries) = tokio::fs::read_dir(dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_file() {
                if let Err(e) = tokio::fs::remove_file(&path).await {
                    println!("删除文件失败 {:?}: {}", path, e);
                } else {
                    println!("已删除文件: {:?}", path);
                }
            }
        }
    }
}

/// 获取配置
async fn get_config() -> impl IntoResponse {
    let config = load_config().await;
    axum::Json(serde_json::json!({
        "success": true,
        "config": config
    }))
}

/// 更新配置
async fn update_config(
    axum::extract::Json(body): axum::extract::Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some(hours) = body.get("auto_clean_hours").and_then(|v| v.as_u64()) {
        let mut config = load_config().await;
        config.auto_clean_hours = hours as u32;
        save_config(&config).await;

        axum::Json(serde_json::json!({
            "success": true,
            "message": "配置已更新"
        }))
    } else {
        axum::Json(serde_json::json!({
            "success": false,
            "message": "无效的配置参数"
        }))
    }
}

/// 手动清理文件
async fn clean_files() -> impl IntoResponse {
    clean_shared_files().await;

    // 更新上次清理时间
    let mut config = load_config().await;
    config.last_clean_time = Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    save_config(&config).await;

    axum::Json(serde_json::json!({
        "success": true,
        "message": "文件清理完成"
    }))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::State((tx, clients, message_history)): axum::extract::State<AppState>,
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
        "success": !uploaded_files.is_empty(),
        "files": uploaded_files,
        "errors": errors
    }))
}
