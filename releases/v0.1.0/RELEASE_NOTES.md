# PasteMind v0.1.0

Release date: 2026-03-09

## Downloads

- `PasteMind_0.1.0_macos_aarch64.app.zip`
- Checksum: `SHA256SUMS.txt`

## Highlights

- Local-first clipboard manager for macOS 13+ (Apple Silicon)
- Menu bar resident app with global hotkey summon
- Clipboard history for both text and images
- Image persistence to local files with SQLite metadata index
- Search, delete item, clear all, and retention policy
- Privacy-first: no cloud sync, no outbound clipboard upload
- Source app exclusion list for password manager protection
- Auto paste flow with Accessibility permission guidance and fallback
- Bilingual UI (Chinese / English)
- Clickable preview: click text/image content to paste immediately
- Custom hotkey recording with support for `F1` to `F24`

## Performance and limits

- Polling watcher interval: 300ms
- Max history items: 500 (auto-prune oldest)
- Max single image size: 10MB

## Notes

- Build target in this release: `macOS aarch64`
- This release is unsigned and not notarized yet.
