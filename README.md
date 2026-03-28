# 局域网聊天工具 (LAN-Share)

一个基于 Rust + Axum 构建的高性能局域网实时聊天工具，支持文本、图片和文件分享。

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![GitHub](https://img.shields.io/badge/GitHub-Sunrisies/lan--share-blue.svg)](https://github.com/Sunrisies/lan-share)

---

## ✨ 功能特性

### 💬 实时通信
- ✅ 基于 WebSocket 的实时消息推送
- ✅ 毫秒级消息响应
- ✅ 消息自动滚动到最新

### 📁 多媒体支持
- ✅ 文本消息
- ✅ 图片上传和预览（支持 JPG、PNG、GIF、WebP）
- ✅ 文件上传和下载（最大 100MB）

### 👥 用户管理
- ✅ 在线用户数量实时显示
- ✅ 消息历史记录（最多 100 条）
- ✅ 消息时间戳

### ⚙️ 智能管理
- ✅ 可配置的自动清理任务
- ✅ 手动清理功能
- ✅ 图形化设置界面

### 🎨 用户体验
- ✅ 现代化 UI 设计（Tailwind CSS）
- ✅ 消息提示弹窗
- ✅ 文件大小限制提示

---

## 🚀 快速开始

### 环境要求

- Rust 1.70+
- Cargo

### 安装 Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 运行项目

```bash
# 克隆项目
git clone https://github.com/Sunrisies/lan-share.git
cd lan-share

# 编译运行
cargo run
```

### 访问应用

打开浏览器访问：http://localhost:3000

局域网内其他设备访问：http://YOUR_IP:3000

---

## 📖 使用说明

### 发送消息

1. **文本消息**：在输入框输入消息，按 Enter 键发送
2. **图片分享**：点击附件按钮 📎，选择图片文件
3. **文件传输**：点击附件按钮 📎，选择任意文件

### 设置功能

点击右上角 ⚙️ 图标打开设置：
- 配置自动清理间隔（1-168 小时）
- 查看上次清理时间
- 手动清理文件

---

## 🛠️ 技术栈

| 类别 | 技术 |
|------|------|
| 后端 | Rust + Axum + Tokio |
| 前端 | HTML + Tailwind CSS + JavaScript |
| 通信 | WebSocket |
| 文件处理 | Multipart |

---

## 📁 项目结构

```
synchronization/
├── src/
│   └── main.rs              # 后端主程序
├── static/
│   └── index.html           # 前端界面
├── shared_files/            # 上传文件存储目录
├── docs/                    # 文档目录
├── Cargo.toml               # 项目配置
├── README.md                # 项目说明
```

---

## 🔌 API 接口

### WebSocket

- **连接地址**: `ws://localhost:3000/ws`
- **消息格式**: JSON

### HTTP

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/` | 主页面 |
| POST | `/upload` | 文件上传 |
| GET | `/files/{filename}` | 文件下载 |
| GET | `/api/config` | 获取配置 |
| POST | `/api/config` | 更新配置 |
| POST | `/api/clean` | 手动清理 |

---

## ⚙️ 配置说明

### 修改端口

编辑 `src/main.rs`：

```rust
let addr = SocketAddr::from(([0, 0, 0, 0], 3000)); // 修改端口号
```

### 修改文件大小限制

编辑 `src/main.rs`：

```rust
const MAX_FILE_SIZE: usize = 100 * 1024 * 1024; // 修改大小限制（字节）
```

---

## 📚 文档

- [文档目录](docs/README.md)
- [项目介绍](docs/project-introduction.md)
- [技术博客](docs/technical-blog.md)
- [产品白皮书](docs/product-whitepaper.md)
- [产品宣传](docs/promotion.md)

---

## ❓ 常见问题

### 文件上传失败

- 检查文件大小是否超过 100MB
- 检查网络连接
- 查看服务器控制台错误信息

### WebSocket 连接失败

- 确保服务器正在运行
- 检查防火墙设置
- 确认端口未被占用

### 无法访问其他设备

- 确保所有设备在同一局域网
- 检查服务器绑定地址（默认 0.0.0.0）
- 检查防火墙设置

---

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

### 贡献方式

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 创建 Pull Request

详见 [项目介绍 - 贡献指南](docs/project-introduction.md#贡献指南)

## 📧 联系方式

- **GitHub**: [Sunrisies/lan-share](https://github.com/Sunrisies/lan-share)
- **Email**: 3266420686@qq.com
- **Issues**: [提交问题](https://github.com/Sunrisies/lan-share/issues)

---

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

---

## 🙏 致谢

- [Axum](https://github.com/tokio-rs/axum) - Web 框架
- [Tokio](https://tokio.rs/) - 异步运行时
- [Tailwind CSS](https://tailwindcss.com/) - CSS 框架
- [Rust Embed](https://github.com/pyros2097/rust-embed) - 静态资源嵌入

## ⭐ Star History

如果这个项目对您有帮助，请给我们一个 Star！

[![Star History Chart](https://api.star-history.com/svg?repos=Sunrisies/lan-share&type=Date)](https://star-history.com/#Sunrisies/lan-share&Date)

---

**让局域网沟通更简单！** 🚀

Made with ❤️ by [Sunrisies](https://github.com/Sunrisies)
