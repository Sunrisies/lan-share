# 局域网聊天工具 - 项目介绍

## 项目简介

**局域网聊天工具** 是一个基于 Rust + Axum 构建的实时聊天应用，专为局域网环境设计。它支持文本消息、图片分享、文件传输，并提供了简洁美观的用户界面。

## 核心特性

### 实时通信

- 基于 WebSocket 的实时消息推送
- 毫秒级消息响应
- 自动重连机制

### 多媒体支持

- 📝 **文本消息** - 支持富文本格式
- 🖼️ **图片分享** - 自动预览，点击查看大图
- 📁 **文件传输** - 支持任意类型文件，最大 100MB

### 用户体验

- 🎨 **现代化界面** - 使用 Tailwind CSS 构建
- 🌙 **深色模式** - 保护眼睛（规划中）
- 📱 **响应式设计** - 支持各种屏幕尺寸
- ⚡ **自动滚动** - 新消息自动滚动到底部

### 智能管理

- 👥 **在线用户统计** - 实时显示当前在线人数
- 💬 **消息历史** - 新用户可查看历史消息
- ⏰ **自动清理** - 可配置的定时清理任务
- 🔧 **灵活配置** - 通过界面轻松调整设置

## 技术架构

### 后端技术栈

```
Rust + Axum + Tokio + WebSocket
```

- **Axum** - 高性能 Web 框架
- **Tokio** - 异步运行时
- **WebSocket** - 实时双向通信
- **Rust Embed** - 静态资源嵌入

### 前端技术栈

```
HTML + Tailwind CSS + JavaScript
```

- **Tailwind CSS** - 实用优先的 CSS 框架
- **原生 JavaScript** - 无框架依赖，轻量高效

## 适用场景

### 办公环境

- 团队内部沟通
- 文件快速分享
- 项目协作交流

### 家庭网络

- 家人之间传文件
- 照片视频分享
- 即时通讯需求

### 教育场景

- 课堂文件分发
- 师生互动交流
- 作业提交收集

### 临时活动

- 会议文件共享
- 活动现场通讯
- 临时团队协作

## 项目亮点

### 1. 零配置启动

```bash
cargo run
```

一条命令即可启动，无需复杂配置。

### 2. 跨平台支持

支持 Windows、macOS、Linux 三大平台。

### 3. 低资源占用

Rust 语言的高效性能，内存占用极低。

### 4. 安全可靠

- 局域网内部通信，数据不出网络
- 支持密码保护（规划中）
- 文件大小限制，防止滥用

## 快速开始

### 安装依赖

确保已安装 Rust 开发环境：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 克隆项目

```bash
git clone https://github.com/Sunrisies/lan-share.git
cd lan-share
```

### 运行项目

```bash
cargo run
```

### 访问应用

打开浏览器访问：http://localhost:3000

## 功能截图

### 主界面

- 简洁的聊天界面
- 在线用户数量显示
- 消息时间戳

### 设置界面

- 自动清理时间配置
- 手动清理按钮
- 上次清理时间显示

## 开源协议

本项目采用 MIT 开源协议，欢迎自由使用和贡献。

## 贡献指南

欢迎提交 Issue 和 Pull Request！

### 贡献方式

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 创建 Pull Request

## 联系方式

如有问题或建议，欢迎通过以下方式联系：

- **GitHub**：[Sunrisies/lan-share](https://github.com/Sunrisies/lan-share)
- **Issues**：[提交问题](https://github.com/Sunrisies/lan-share/issues)
- **Email**：3266420686@qq.com

---

## 📄 文档版本

- **当前版本**：v0.1.0
- **更新日期**：2026年3月28日

---

**让局域网沟通更简单！** 🚀
