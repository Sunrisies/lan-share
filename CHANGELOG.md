# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

---

## [v0.3.0] - 2026-03-30

### 🚀 功能增强 & 体验优化

#### ✨ 新增功能

**文件上传增强**
- 拖拽上传支持：直接将文件拖入聊天窗口即可上传
- 剪贴板粘贴图片：在输入框 Ctrl+V 即可粘贴并发送图片

**图片预览**
- 点击图片在当前页面打开预览
- 支持下载原图
- 多种关闭方式（点击遮罩、关闭按钮、ESC 键）

**开机自启**
- Windows：添加到注册表启动项
- Linux：创建 desktop 启动文件
- 设置中一键开关，默认为关闭

#### 🐛 Bug 修复

- 修复刷新页面后消息归属显示错误的问题
- 修复图片在新页面打开的问题，改为预览弹窗

#### 📱 移动端优化

- 拖拽上传支持移动端浏览器
- 图片预览适配移动端屏幕

#### 🛠️ 技术改进

- 后端新增 `/api/autostart` API
- 前端使用原生 Tailwind CSS 弹窗样式
- 添加 `dirs` 依赖用于获取用户配置目录

---

## [v0.2.0] - 2026-03-28

### 🎉 新增扫码加入 & 移动端适配

#### ✨ 新增功能

**扫码加入**
- 新增二维码扫码功能，手机扫描即可加入聊天
- 自动获取局域网 IP 地址
- 支持一键复制访问地址
- 使用 qrcode.js 生成二维码

**移动端适配**
- 完整的响应式布局支持
- 移动端优化的输入框和按钮
- 弹窗自适应屏幕尺寸
- 动态视口高度（100dvh）

**用户体验优化**
- 消息归属持久化（使用 localStorage）
- 刷新页面后自己的消息仍显示正确
- ES Module 支持
- 优化的 toast 提示动画

#### 🐛 Bug 修复

- 修复刷新页面后消息归属错误的问题
- 修复移动端样式错乱问题
- 修复 `<script type="module">` 无法调用内联事件的问题

#### 🛠️ 技术改进

- 新增 `local-ip-address` 依赖，用于获取局域网 IP
- 新增 `/api/server-info` API 接口
- 使用 localStorage 持久化客户端 ID
- 前端代码重构，支持 ES Module

#### 📱 移动端优化

| 元素 | 优化内容 |
|------|----------|
| 头部 | 紧凑布局，标题自适应 |
| 在线用户 | 移动端/桌面端分开显示 |
| 输入框 | 更小的 padding |
| 发送按钮 | 移动端显示图标 |
| 弹窗 | 全宽但有边距 |
| 提示框 | 顶部通栏显示 |

#### 🔗 相关链接

- **GitHub**: [Sunrisies/lan-share](https://github.com/Sunrisies/lan-share)
- **文档**: [项目介绍](docs/project-introduction.md)
- **Issues**: [提交问题](https://github.com/Sunrisies/lan-share/issues)

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

[v0.3.0]: https://github.com/Sunrisies/lan-share/releases/tag/v0.3.0
[v0.2.0]: https://github.com/Sunrisies/lan-share/releases/tag/v0.2.0
[v0.1.0]: https://github.com/Sunrisies/lan-share/releases/tag/v0.1.0
