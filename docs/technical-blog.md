# 用 Rust 构建高性能局域网聊天工具

## 前言

在日常工作中，我们经常需要在局域网内快速分享文件和进行即时通讯。虽然市面上有很多成熟的聊天工具，但它们往往需要注册账号、连接外网，对于纯粹的局域网场景来说过于复杂。

今天，我想分享如何使用 Rust 语言构建一个轻量级、高性能的局域网聊天工具。

## 为什么选择 Rust？

### 性能优势
Rust 以其零成本抽象和内存安全著称，非常适合构建高性能网络服务：

- **零成本抽象** - 无运行时开销
- **内存安全** - 无垃圾回收，无内存泄漏
- **并发安全** - 编译期保证线程安全

### 生态成熟
Rust 的异步生态已经非常成熟：

- **Tokio** - 高性能异步运行时
- **Axum** - 现代化 Web 框架
- **Serde** - 序列化/反序列化

## 技术选型

### 后端框架：Axum

Axum 是基于 Tower 生态构建的 Web 框架，具有以下特点：

```rust
use axum::{
    routing::{get, post},
    Router,
};

let app = Router::new()
    .route("/ws", get(ws_handler))
    .route("/upload", post(upload_handler));
```

- 类型安全的路由
- 优秀的中间件支持
- 与 Tokio 无缝集成

### 实时通信：WebSocket

WebSocket 提供了全双工通信能力，非常适合实时聊天：

```rust
async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket))
}

async fn handle_socket(socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();

    // 处理消息...
}
```

### 异步运行时：Tokio

Tokio 是 Rust 生态中最流行的异步运行时：

```rust
#[tokio::main]
async fn main() {
    // 异步代码
}
```

## 核心功能实现

### 1. 消息广播

使用 `broadcast::channel` 实现消息广播：

```rust
// 创建广播通道
let (tx, _rx) = broadcast::channel::<String>(100);

// 广播消息
let _ = tx.send(json);

// 接收消息
let mut rx = tx.subscribe();
while let Ok(msg) = rx.recv().await {
    // 处理消息
}
```

### 2. 文件上传

使用 Multipart 处理文件上传：

```rust
async fn upload_handler(mut multipart: Multipart) -> impl IntoResponse {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let file_name = field.file_name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        // 保存文件
        tokio::fs::write(&path, &data).await.unwrap();
    }
}
```

### 3. 静态资源嵌入

使用 `rust-embed` 将静态文件嵌入二进制：

```rust
#[derive(RustEmbed)]
#[folder = "static/"]
struct StaticAssets;

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    match StaticAssets::get(path) {
        Some(content) => {
            // 返回文件内容
        }
        None => {
            // 返回 404
        }
    }
}
```

### 4. 定时任务

使用 Tokio 实现定时清理任务：

```rust
async fn start_auto_clean_task(config: Arc<Mutex<Config>>) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

        // 检查是否需要清理
        if should_clean {
            clean_shared_files().await;
        }
    }
}
```

## 前端实现

### WebSocket 连接

```javascript
const ws = new WebSocket(`ws://${window.location.host}/ws`);

ws.onopen = function() {
    console.log('已连接');
};

ws.onmessage = function(event) {
    const data = JSON.parse(event.data);
    // 处理消息
};

ws.send(JSON.stringify({
    type: 'message',
    content: '你好！',
    sender_id: myClientId
}));
```

### 文件上传

```javascript
async function uploadChatFiles(files) {
    const formData = new FormData();
    for (let file of files) {
        formData.append('file', file);
    }

    const response = await fetch('/upload', {
        method: 'POST',
        body: formData
    });

    const result = await response.json();
    // 处理结果
}
```

## 性能优化

### 1. 内存管理

- 使用 `Arc<Mutex<T>>` 共享状态
- 限制历史消息数量（最多100条）
- 及时释放不需要的资源

### 2. 并发处理

- 每个 WebSocket 连接独立任务
- 使用 `tokio::select!` 处理多路复用
- 异步文件 I/O

### 3. 错误处理

- 使用 `Result` 类型处理错误
- 避免 `unwrap()` 导致的 panic
- 优雅降级

## 部署建议

### 开发环境

```bash
cargo run
```

### 生产环境

```bash
cargo build --release
./target/release/lan-share
```

### Docker 部署

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
COPY --from=builder /app/target/release/lan-share /usr/local/bin/
CMD ["lan-share"]
```

## 总结

通过这个项目，我们展示了如何使用 Rust 构建一个实用的局域网聊天工具。Rust 的类型系统和内存安全保证让我们能够编写出高性能、可靠的代码。

如果你也想尝试 Rust 开发 Web 应用，这个项目是一个很好的起点。欢迎查看源码，提出建议！

## 相关资源

- [Axum 官方文档](https://docs.rs/axum)
- [Tokio 官方文档](https://tokio.rs)
- [Rust 官方文档](https://doc.rust-lang.org)
- [项目源码](https://github.com/Sunrisies/lan-share)

## 联系方式

- **GitHub**：[Sunrisies/lan-share](https://github.com/Sunrisies/lan-share)
- **Issues**：[提交问题](https://github.com/Sunrisies/lan-share/issues)
- **Email**：3266420686@qq.com

## 📄 文档版本

- **当前版本**：v0.1.0
- **更新日期**：2026年3月28日

---

**Happy Coding!** 🦀
