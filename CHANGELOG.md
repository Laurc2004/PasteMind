# Changelog

## [0.2.1] - 2026-04-24

### Changed

- Release workflow now uses generated `RELEASE_NOTES.md` as the GitHub Release body instead of static text.
- Signing mode and notarization status removed from public release notes.

### Fixed

- Added SQL identifier validation to `ensure_column_exists` to guard against future injection.
- Objective-C FFI: explicitly retain `NSData` before reading bytes; added null check on `NSString` alloc.
- Hotkey registration failure no longer overwrites the user's saved preference — fallback is in-memory only.

## [0.2.0] - 2026-04-24

### Added

- CI workflow (`ci.yml`) for automated lint, format, type check, clippy, and test on every push/PR.
- Version synchronization script (`scripts/sync-version.sh`) and `pnpm version` command.
- Version consistency validation step in release workflow.
- Rust compilation cache (`Swatinem/rust-cache`) in release workflow.
- Modular frontend architecture: extracted `$lib/i18n`, `$lib/format`, `$lib/hotkey`, and Svelte components (`EntryCard`, `ConfirmModal`, `Toast`).

### Changed

- Enabled Content Security Policy (CSP) in Tauri webview for security hardening.
- Split monolithic `+page.svelte` (1319 lines) into focused modules and components.
- Migrated component event handlers to Svelte 5 `onclick` syntax.

### Fixed

- Race condition in `reloadEntries()` — concurrent calls no longer corrupt loading state or produce stale data (generation counter).
- Unhandled promise rejection in `onPasteNow` — `reloadPermission()` now has its own try-catch.
- Event listener leak on rapid mount/unmount — cleanup now awaits listener setup promises.
- `localStorage` operations now wrapped in try-catch for Safari private browsing compatibility.
- Clippy warnings: collapsed nested if, needless `as_bytes()`, needless borrows, and dead code in `main.rs`.

## [0.1.5] - 2026-03-09

### Fixed

- Prevented `tauri build` from reading Apple signing/notarization environment variables too early in CI.
- Scoped Apple credentials to post-build signing/notarization steps, fixing `v0.1.4` release workflow failure.

## [0.1.4] - 2026-03-09

### Fixed

- Reworked macOS release pipeline to explicitly sign the generated `.app` before packaging, preventing malformed bundle signatures.
- Added optional Developer ID signing + notarization path in CI when Apple credentials are configured.
- Rebuilt release `.dmg` from the signed app in CI instead of relying on unsigned intermediate output.

## [0.1.3] - 2026-03-09

### Fixed

- Made CI release asset collection robust to dynamic bundle filename outputs.
- Added explicit app/dmg output discovery and failure diagnostics in release workflow.

## [0.1.2] - 2026-03-09

### Fixed

- Fixed GitHub Actions release workflow YAML parsing issue.
- Restored automatic macOS release pipeline trigger on tag push.

## [0.1.1] - 2026-03-09

### Added

- GitHub Actions macOS release workflow to build and publish both `.dmg` and `.app.zip`.
- Release asset generation with SHA256 checksum and release notes attachment.

### Changed

- Updated macOS bundle identifier to `com.laurc2004.pastemind`.
- Kept local default bundle target to `app` while CI builds `app,dmg`.

## [0.1.0] - 2026-03-08

### Added

- Initial Tauri + SvelteKit clipboard app scaffold.
- Clipboard watcher for text/image capture.
- SQLite metadata store + image file storage.
- History search, delete, clear and retention policy.
- Auto paste flow with accessibility permission checks.
- Menu bar/tray interaction and global shortcut target (`Cmd+Shift+V`).
- Bilingual open-source documentation.

### Changed

- Updated UI to glass style with bilingual toggle and improved interaction flow.
- Settings panel placed below clipboard history for Win+V-like usage rhythm.
- Added clickable preview area (text/image) for direct paste action.
- Hotkey recorder now supports `F1` to `F24` (with or without modifiers).

### Fixed

- Closing main window now hides the window instead of quitting the app.
- Improved image clipboard capture reliability on macOS with native pasteboard fallback.
- Prevented duplicate history insertion during auto-paste selection flow.
