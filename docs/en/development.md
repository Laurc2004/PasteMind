# Development Guide

## Stack

- Frontend: SvelteKit + TypeScript
- Desktop shell: Tauri 2
- Backend: Rust
- Storage: SQLite metadata + local image files

## Run locally

```bash
pnpm install
pnpm dev
```

## Main modules

- `clipboard_watcher.rs`: 300ms polling for text/image capture.
- `storage.rs`: persistence, dedupe, retention cleanup.
- `permissions.rs`: accessibility checks and system settings routing.
- `selection.rs`: copy/auto-paste behavior.
- `commands.rs`: Tauri command contracts.

## Test

```bash
pnpm test
```

## Debug tips

- Check accessibility permission first if auto-paste fails.
- Verify excluded source rules if new records are missing.
