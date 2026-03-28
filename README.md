# 局域网聊天工具

一个基于 Rust + Axum 的局域网实时聊天工具，支持文本、图片和文件分享。

## 功能特性

- ✅ 实时文本聊天
- ✅ 图片上传和预览
- ✅ 文件上传和下载
- ✅ 在线用户数量显示
- ✅ 消息历史记录
- ✅ 消息时间戳
- ✅ 文件大小限制提示（100MB）
- ✅ 消息提示弹窗
- ✅ 自动滚动到最新消息

## 技术栈

- **后端**: Rust + Axum + Tokio + WebSocket
- **前端**: HTML + Tailwind CSS + JavaScript
- **通信**: WebSocket 实时通信
- **文件处理**: Multipart 文件上传

## 快速开始

### 环境要求

- Rust 1.70+
- Cargo

### 安装

```bash
# 克隆项目
git clone <repository-url>
cd synchronization

# 编译项目
cargo build --release

# 运行项目
cargo run
```

### 访问

服务器启动后，访问 http://localhost:3000

## 使用说明

### 聊天功能

1. **发送文本消息**

   - 在输入框中输入消息
   - 按 Enter 键或点击"发送"按钮
2. **发送图片**

   - 点击附件按钮
   - 选择图片文件
   - 图片会自动上传并显示预览
3. **发送文件**

   - 点击附件按钮
   - 选择任意文件
   - 文件会自动上传，其他用户可下载

### 界面说明

- **在线用户数量**: 页面顶部显示当前在线用户数
- **消息历史**: 新用户加入时可查看之前的消息
- **时间戳**: 每条消息显示发送时间
- **消息提示**: 操作成功或失败会显示提示弹窗

## 配置说明

### 修改端口

编辑 `src/main.rs` 文件：

```rust
let addr = SocketAddr::from(([0, 0, 0, 0], 3000)); // 修改端口号
```

### 修改文件大小限制

编辑 `src/main.rs` 文件：

```rust
const MAX_FILE_SIZE: usize = 100 * 1024 * 1024; // 修改大小限制（字节）
```

### 修改历史消息数量

编辑 `src/main.rs` 文件：

```rust
if history.len() > 100 { // 修改历史消息数量
    history.remove(0);
}
```

## 项目结构

```
synchronization/
├── src/
│   └── main.rs          # 后端主程序
├── static/
│   └── index.html       # 前端界面
├── shared_files/        # 上传文件存储目录
├── Cargo.toml           # 项目依赖
└── README.md            # 项目文档
```

## API 接口

### WebSocket

- **连接地址**: `ws://localhost:3000/ws`
- **消息格式**: JSON

#### 消息类型

1. **文本消息**

```json
{
  "type": "message",
  "content": "消息内容",
  "sender_id": "用户ID"
}
```

2. **图片消息**

```json
{
  "type": "image",
  "content": "",
  "sender_id": "用户ID",
  "file_url": "/files/xxx.jpg",
  "file_name": "图片.jpg",
  "file_type": "image"
}
```

3. **文件消息**

```json
{
  "type": "file",
  "content": "",
  "sender_id": "用户ID",
  "file_url": "/files/xxx.zip",
  "file_name": "文件.zip",
  "file_type": "file"
}
```

### HTTP

- `GET /` - 主页面
- `POST /upload` - 文件上传
- `GET /files/{filename}` - 文件下载
- `GET /files` - 文件列表

## 常见问题

### 1. 文件上传失败

- 检查文件大小是否超过 100MB
- 检查网络连接是否正常
- 查看服务器控制台错误信息

### 2. WebSocket 连接失败

- 确保服务器正在运行
- 检查防火墙设置
- 确认端口未被占用

### 3. 无法访问其他设备

- 确保所有设备在同一局域网
- 检查服务器绑定地址（默认 0.0.0.0）
- 检查防火墙设置

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！
