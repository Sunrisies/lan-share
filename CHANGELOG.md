# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

---

## [v0.1.0] - 2026-03-28

### 🎉 首个正式版本发布

#### ✨ 新增功能

**实时通信**
- 基于 WebSocket 的实时消息推送
- 毫秒级消息响应
- 消息自动滚动到最新

**多媒体支持**
- 文本消息发送和接收
- 图片上传和预览（支持 JPG、PNG、GIF、WebP）
- 文件上传和下载（最大 100MB）

**用户管理**
- 在线用户数量实时显示
- 消息历史记录（最多 100 条）
- 消息时间戳显示

**智能管理**
- 可配置的自动清理任务（1-168 小时）
- 手动清理功能
- 图形化设置界面

**用户体验**
- 现代化 UI 设计（Tailwind CSS）
- 消息提示弹窗（成功/错误/警告/信息）
- 文件大小限制提示
- 静态资源嵌入（支持单文件部署）

#### 🛠️ 技术栈

| 类别 | 技术 |
|------|------|
| 后端 | Rust + Axum + Tokio |
| 前端 | HTML + Tailwind CSS + JavaScript |
| 通信 | WebSocket |
| 文件处理 | Multipart |
| 静态资源 | Rust Embed |

#### 📦 安装方式

```bash
# 克隆项目
git clone https://github.com/Sunrisies/lan-share.git
cd lan-share

# 编译运行
cargo run

# 或者编译发布版本
cargo build --release
```

#### 🔗 相关链接

- **GitHub**: [Sunrisies/lan-share](https://github.com/Sunrisies/lan-share)
- **文档**: [项目介绍](docs/project-introduction.md)
- **Issues**: [提交问题](https://github.com/Sunrisies/lan-share/issues)

#### 🙏 致谢

感谢所有为这个项目做出贡献的人！

---

[v0.1.0]: https://github.com/Sunrisies/lan-share/releases/tag/v0.1.0
