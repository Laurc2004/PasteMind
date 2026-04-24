# PasteMind

PasteMind is a local-first clipboard manager for macOS 13+ (Apple Silicon).

[中文文档](README.zh-CN.md)

## Core Goals

- Capture text and image clipboard items with low overhead
- Keep all data on-device — no cloud sync, no remote upload
- Provide fast menu bar access and one-step auto paste

## Features

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

## Architecture

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

## Project Layout

```text
src/                 SvelteKit frontend
src-tauri/src/       Rust backend (watcher/storage/commands)
src-tauri/tests/     Rust integration tests
docs/zh/             Chinese docs
docs/en/             English docs
scripts/             Helper scripts
```

## Quick Start

### Prerequisites

- Node.js 20+
- pnpm 9+
- Rust stable
- Xcode Command Line Tools

### Install and run

```bash
pnpm install
pnpm dev
```

### Run tests

```bash
pnpm test
```

### Regenerate icons (optional)

```bash
python3 scripts/generate_icons.py
```

## Release Artifacts

GitHub Releases: `https://github.com/Laurc2004/PasteMind/releases`

Each release publishes:

- `PasteMind_<version>_macos_aarch64.dmg`
- `PasteMind_<version>_macos_aarch64.app.zip`
- `SHA256SUMS.txt`
- `RELEASE_NOTES.md`

## macOS Install Troubleshooting

If macOS shows `"PasteMind" is damaged and can't be opened`, it is usually Gatekeeper quarantine plus unsigned/not-notarized build policy.

For your own downloaded build, run:

```bash
xattr -dr com.apple.quarantine /Applications/PasteMind.app
```

Then open once from Finder with right-click **Open**.

For public distribution, configure Apple signing + notarization secrets in GitHub Actions to avoid this warning entirely.

## Privacy & Permissions

- No cloud — data is stored in app data directory only
- Auto paste requires macOS Accessibility permission
- By default, known password manager bundle IDs are excluded from capture

Detailed docs:

- English development doc: `docs/en/development.md`
- English privacy doc: `docs/en/privacy-and-permissions.md`
- 中文开发文档: `docs/zh/开发指南.md`
- 中文隐私文档: `docs/zh/隐私与权限.md`

## Roadmap (v1)

- [x] Text/image history capture
- [x] Local persistence and retention
- [x] Menu bar UX and global shortcut
- [x] Permission-aware auto paste fallback
- [ ] Signed & notarized distribution
- [ ] Login item option

## License

MIT
