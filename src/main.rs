use axum::{
    extract::{Multipart, WebSocketUpgrade},
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    // Create a broadcast channel for messages
    let (tx, _rx) = broadcast::channel::<String>(100);

    // Create shared state for connected clients
    let clients = Arc::new(Mutex::new(HashMap::new()));

    // Build the application router
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/ws", get(ws_handler))
        .route("/upload", post(upload_handler))
        .nest_service("/files", ServeDir::new("shared_files"))
        .with_state((tx, clients));

    // Run the server
    let addr = SocketAddr::from(([0, 0, 0, 0], 9000));
    println!("Server running on http://{}", addr);
    println!("File sharing available at http://{}/files", addr);
    println!("WebSocket chat available at ws://{}/ws", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn index_handler() -> Html<&'static str> {
    Html(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>LAN Sharing</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .container { max-width: 800px; margin: 0 auto; }
        .section { margin-bottom: 40px; }
        h1 { color: #333; }
        .chat-box { height: 300px; border: 1px solid #ccc; overflow-y: scroll; padding: 10px; margin-bottom: 10px; }
        .message { margin-bottom: 5px; }
        .file-list { margin-top: 20px; }
    </style>
</head>
<body>
    <div class="container">
        <h1>LAN File Sharing & Chat</h1>
        
        <div class="section">
            <h2>File Upload</h2>
            <form action="/upload" method="post" enctype="multipart/form-data">
                <input type="file" name="file" required>
                <button type="submit">Upload</button>
            </form>
            <div class="file-list">
                <h3>Shared Files:</h3>
                <ul id="file-list"></ul>
            </div>
        </div>
        
        <div class="section">
            <h2>Chat</h2>
            <div class="chat-box" id="chat-box"></div>
            <input type="text" id="message-input" placeholder="Type a message..." style="width: 80%; padding: 5px;">
            <button onclick="sendMessage()" style="padding: 5px 10px;">Send</button>
        </div>
    </div>

    <script>
        const chatBox = document.getElementById('chat-box');
        const messageInput = document.getElementById('message-input');
        const fileList = document.getElementById('file-list');
        
        // WebSocket connection
        const ws = new WebSocket(`ws://${window.location.host}/ws`);
        
        ws.onmessage = function(event) {
            const message = document.createElement('div');
            message.className = 'message';
            message.textContent = event.data;
            chatBox.appendChild(message);
            chatBox.scrollTop = chatBox.scrollHeight;
        };
        
        function sendMessage() {
            const message = messageInput.value;
            if (message) {
                ws.send(message);
                messageInput.value = '';
            }
        }
        
        messageInput.addEventListener('keypress', function(e) {
            if (e.key === 'Enter') {
                sendMessage();
            }
        });
        
        // Load file list
        async function loadFileList() {
            try {
                const response = await fetch('/files');
                if (response.ok) {
                    const text = await response.text();
                    // Simple parsing of directory listing
                    const parser = new DOMParser();
                    const doc = parser.parseFromString(text, 'text/html');
                    const links = doc.querySelectorAll('a');
                    fileList.innerHTML = '';
                    links.forEach(link => {
                        if (link.href.includes('/files/')) {
                            const li = document.createElement('li');
                            const a = document.createElement('a');
                            a.href = link.href;
                            a.textContent = link.textContent;
                            a.target = '_blank';
                            li.appendChild(a);
                            fileList.appendChild(li);
                        }
                    });
                }
            } catch (error) {
                console.error('Error loading file list:', error);
            }
        }
        
        // Load file list on page load
        loadFileList();
        // Refresh file list every 5 seconds
        setInterval(loadFileList, 5000);
    </script>
</body>
</html>
    "#,
    )
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
                // Broadcast the message to all clients
                let _ = tx2.send(text);
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
