# Privacy and Permissions

## Data policy

- PasteMind never uploads clipboard data.
- Clipboard history is stored locally only.
- Images are saved as local files; SQLite stores metadata and file paths.

## Default safeguards

- Known password manager bundle IDs are excluded by default.
- Image capture has a default 10MB single-item limit.
- History keeps the latest 500 entries by default.

## Permission model

- Auto-paste requires macOS Accessibility permission.
- Without permission, PasteMind still writes selected content back to clipboard.
