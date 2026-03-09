# PasteMind

PasteMind is a local-first clipboard manager for macOS 13+ (Apple Silicon).  
PasteMind 是一个面向 macOS 13+（Apple Silicon）的本地优先剪贴板管理工具。

## Core Goals | 核心目标

- Capture text and image clipboard items with low overhead.  
  低开销记录文字与图片剪贴板。
- Keep all data on-device (no cloud sync, no remote upload).  
  数据仅保存在本机，不上传云端。
- Provide fast menu bar access and one-step auto paste.  
  提供菜单栏快速访问与一键自动粘贴。

## Features | 功能

- Menu bar app with customizable summon shortcut (default: `Cmd+Shift+V`)
- Clipboard history with search (text + image)
- Local SQLite metadata + local file storage for images
- History cap (default 500), image size cap (default 10MB)
- Duplicate suppression with content hash
- Sensitive source exclusion (password manager bundle IDs)
- Auto paste with graceful permission fallback
- Bilingual UI switch (`中文 / English`) in-app
- Click text/image preview to paste immediately
- Hotkey recording supports `F1` to `F24` (standalone or with modifiers)

## Release Artifacts | 发布产物

- GitHub Releases: `https://github.com/Laurc2004/PasteMind/releases`
- Each release publishes:
  - `PasteMind_<version>_macos_aarch64.dmg`
  - `PasteMind_<version>_macos_aarch64.app.zip`
  - `SHA256SUMS.txt`
  - `RELEASE_NOTES.md`

## Architecture | 架构

```text
┌──────────────────────────────────────────┐
│               SvelteKit UI               │
│  history list, search, permissions hint  │
└───────────────▲───────────────┬──────────┘
                │ invoke/events │
┌───────────────┴───────────────▼──────────┐
│               Tauri Commands             │
│ get_history/select_entry/update_settings │
└───────────────▲───────────────┬──────────┘
                │               │
   ┌────────────┴───────┐   ┌───┴────────────────┐
   │ Clipboard Watcher  │   │ Selection Pipeline │
   │ 300ms polling      │   │ write + auto paste │
   └────────────▲───────┘   └───▲────────────────┘
                │               │
         ┌──────┴───────────────┴──────┐
         │      Storage (SQLite+Files)  │
         └──────────────────────────────┘
```

## Project Layout | 目录结构

```text
src/                 SvelteKit frontend
src-tauri/src/       Rust backend (watcher/storage/commands)
src-tauri/tests/     Rust integration tests
docs/zh/             中文文档
docs/en/             English docs
scripts/             Helper scripts
```

## Quick Start | 快速开始

### 1) Prerequisites

- Node.js 20+
- pnpm 9+
- Rust stable
- Xcode Command Line Tools

### 2) Install and run

```bash
pnpm install
pnpm dev
```

### 3) Run tests

```bash
pnpm test
```

### 4) Regenerate icons (optional)

```bash
python3 scripts/generate_icons.py
```

## Privacy & Permissions | 隐私与权限

- No cloud: data is stored in app data directory only.
- Auto paste requires macOS Accessibility permission.
- By default, known password manager bundle IDs are excluded from capture.

Detailed docs:

- 中文开发文档: `docs/zh/开发指南.md`
- 中文隐私文档: `docs/zh/隐私与权限.md`
- English development doc: `docs/en/development.md`
- English privacy doc: `docs/en/privacy-and-permissions.md`

## Roadmap (v1 scope) | 路线图（v1）

- [x] Text/image history capture
- [x] Local persistence and retention
- [x] Menu bar UX and global shortcut
- [x] Permission-aware auto paste fallback
- [ ] Signed & notarized distribution
- [ ] Login item option

## License

MIT
