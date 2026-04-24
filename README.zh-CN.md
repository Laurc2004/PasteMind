# PasteMind

PasteMind 是一个面向 macOS 13+（Apple Silicon）的本地优先剪贴板管理工具。

[English](README.md)

## 核心目标

- 低开销记录文字与图片剪贴板
- 数据仅保存在本机，不上传云端
- 提供菜单栏快速访问与一键自动粘贴

## 功能

- 菜单栏常驻，可自定义呼出快捷键（默认 `Cmd+Shift+V`）
- 剪贴板历史搜索（文本 + 图片）
- 本地 SQLite 元数据 + 本地文件存储图片
- 历史记录上限（默认 500 条），图片大小上限（默认 10MB）
- 基于内容哈希的重复抑制
- 敏感来源排除（密码管理器 Bundle ID）
- 自动粘贴，权限不足时优雅降级
- 应用内双语切换（`中文 / English`）
- 点击文本/图片预览即可粘贴
- 快捷键录制支持 `F1` 至 `F24`（可单独或配合修饰键）

## 架构

```text
┌──────────────────────────────────────────┐
│               SvelteKit UI               │
│     历史列表、搜索、权限提示              │
└───────────────▲───────────────┬──────────┘
                │ invoke/events │
┌───────────────┴───────────────▼──────────┐
│               Tauri Commands             │
│  get_history / select_entry / update_    │
│  settings                                │
└───────────────▲───────────────┬──────────┘
                │               │
   ┌────────────┴───────┐   ┌───┴────────────────┐
   │ Clipboard Watcher  │   │ Selection Pipeline │
   │ 300ms 轮询         │   │ 写入 + 自动粘贴   │
   └────────────▲───────┘   └───▲────────────────┘
                │               │
         ┌──────┴───────────────┴──────┐
         │      Storage (SQLite+Files)  │
         └──────────────────────────────┘
```

## 目录结构

```text
src/                 SvelteKit 前端
src-tauri/src/       Rust 后端（监听/存储/命令）
src-tauri/tests/     Rust 集成测试
docs/zh/             中文文档
docs/en/             English docs
scripts/             辅助脚本
```

## 快速开始

### 前置条件

- Node.js 20+
- pnpm 9+
- Rust stable
- Xcode Command Line Tools

### 安装与运行

```bash
pnpm install
pnpm dev
```

### 运行测试

```bash
pnpm test
```

### 重新生成图标（可选）

```bash
python3 scripts/generate_icons.py
```

## 发布产物

GitHub Releases: `https://github.com/Laurc2004/PasteMind/releases`

每个版本发布以下文件：

- `PasteMind_<version>_macos_aarch64.dmg`
- `PasteMind_<version>_macos_aarch64.app.zip`
- `SHA256SUMS.txt`
- `RELEASE_NOTES.md`

## macOS 安装排障

如果 macOS 提示 `"PasteMind" 已损坏，无法打开`，通常是 Gatekeeper 隔离加上未签名/未公证策略导致的。

对于自己下载的构建，可以运行：

```bash
xattr -dr com.apple.quarantine /Applications/PasteMind.app
```

然后在 Finder 中右键点击 **打开** 即可。

若要公开分发，请在 GitHub Actions 中配置 Apple 签名 + 公证密钥以彻底避免此警告。

## 隐私与权限

- 无云同步 — 数据仅保存在应用数据目录
- 自动粘贴需要 macOS 辅助功能权限
- 默认排除已知密码管理器的 Bundle ID

详细文档：

- 中文开发文档: `docs/zh/开发指南.md`
- 中文隐私文档: `docs/zh/隐私与权限.md`
- English development doc: `docs/en/development.md`
- English privacy doc: `docs/en/privacy-and-permissions.md`

## 路线图（v1）

- [x] 文本/图片历史记录捕获
- [x] 本地持久化与留存
- [x] 菜单栏体验与全局快捷键
- [x] 权限感知的自动粘贴降级
- [ ] 签名与公证分发
- [ ] 登录项选项

## 许可证

MIT
