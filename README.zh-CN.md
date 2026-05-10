# Cliply

[English](README.md) | [简体中文](README.zh-CN.md)

Cliply 是一个面向 Windows 的本地优先剪贴板管理器。它让剪贴板历史快速、
可搜索、可控，不需要账号，也不会把剪贴板内容发送到 Cliply 托管云服务。

## 截图

| 主窗口（浅色） | 主窗口（深色） |
| --- | --- |
| ![Cliply 浅色主窗口](docs/assets/screenshots/main-light.png) | ![Cliply 深色主窗口](docs/assets/screenshots/main-dark.png) |

| 同步设置 | 安装器 |
| --- | --- |
| ![Cliply 同步设置](docs/assets/screenshots/settings-sync.png) | ![Cliply 安装器](docs/assets/screenshots/installer.png) |

## 功能特性

- 支持文本、链接、代码片段和图片剪贴板历史
- 快速搜索、类型筛选、固定、删除和详情预览
- 粘贴、复制、无格式粘贴，以及自动粘贴回上一个应用
- 本地 SQLite 存储，可配置历史保留和重复内容处理
- 图片缩略图和本地图片 blob 存储
- 可配置快捷键、启动行为和粘贴行为
- 浅色/深色主题、强调色和贴近 Windows 的界面控件
- 加密 `.cliply-sync` 同步包导入和导出
- 通过用户自有存储同步：本地文件夹、WebDAV、FTP 和 FTPS
- 支持可配置间隔的自动同步和图片同步模式
- Windows 安装器支持安装、更新、卸载、开机自启和用户数据保留控制
- 可在“关于”页手动检查更新，并打开 GitHub Release 下载页面

## 隐私

Cliply 采用本地优先设计：

- 剪贴板历史保存在你的 Windows 本机。
- Cliply 不需要账号。
- Cliply 不提供、也不使用托管云服务保存你的剪贴板数据。
- 同步包会在写入磁盘或上传到你配置的 provider 之前加密。
- 远程同步 provider 接收的是加密同步包，不是明文剪贴板历史。
- 检查更新只请求 GitHub Releases，不会包含剪贴板历史、同步密码或本地数据库内容。
- 日志和诊断信息不得包含剪贴板正文、同步密码、provider 密码、token、
  Authorization header、private key 或图片内容。

默认 Windows 数据位置：

```text
%APPDATA%\com.cliply.app\
```

更多信息见 [PRIVACY.md](PRIVACY.md) 和
[docs/privacy-and-logs.md](docs/privacy-and-logs.md)。

## 安全

安全敏感区域包括剪贴板捕获、粘贴行为、同步包加密、远程 provider 认证、
诊断信息和安装器升级/卸载流程。

请不要在公开 issue 中粘贴生产密钥或敏感剪贴板内容。若发现安全或隐私问题，
请按照 [SECURITY.md](SECURITY.md) 处理。

## 开发

克隆仓库：

```powershell
git clone https://github.com/<owner>/cliply.git
cd cliply
```

安装依赖：

```powershell
npm install
```

运行桌面应用开发模式：

```powershell
npm run tauri dev
```

构建前端：

```powershell
npm run build
```

运行后端检查：

```powershell
cargo check --manifest-path .\src-tauri\Cargo.toml
```

构建现代安装器：

```powershell
npm run build:modern-installer
```

## 文档

- [隐私政策](PRIVACY.md)
- [安全政策](SECURITY.md)
- [更新日志](CHANGELOG.md)
- [同步设计](docs/sync-design.md)
- [安装器说明](docs/installer.md)
- [隐私和日志](docs/privacy-and-logs.md)

## 技术栈

- 桌面外壳：Tauri v2
- 前端：React、TypeScript、Vite、Tailwind CSS
- 后端：Rust
- 存储：SQLite via `rusqlite`
- 同步加密：AES-GCM + Argon2 密钥派生
- 安装器：Tauri 应用式现代安装器，另有 NSIS 备用安装器

## 许可证

Cliply 使用 [MIT License](LICENSE) 授权。
