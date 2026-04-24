#!/usr/bin/env bash
# Sync version across package.json, Cargo.toml, and tauri.conf.json
# Usage: ./scripts/sync-version.sh <version>   (e.g. 0.2.0)
set -euo pipefail

VERSION="${1:?Usage: $0 <version>}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Error: version must be semver (e.g. 0.2.0), got: $VERSION" >&2
  exit 1
fi

# package.json — only replace the top-level "version" field (line 3)
PACKAGE_JSON="$ROOT/package.json"
if [[ "$(uname)" == "Darwin" ]]; then
  sed -i '' "3s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" "$PACKAGE_JSON"
else
  sed -i "3s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" "$PACKAGE_JSON"
fi
echo "Updated $PACKAGE_JSON -> $VERSION"

# Cargo.toml — only replace the top-level version field (line 3)
CARGO_TOML="$ROOT/src-tauri/Cargo.toml"
if [[ "$(uname)" == "Darwin" ]]; then
  sed -i '' "3s/version = \"[^\"]*\"/version = \"$VERSION\"/" "$CARGO_TOML"
else
  sed -i "3s/version = \"[^\"]*\"/version = \"$VERSION\"/" "$CARGO_TOML"
fi
echo "Updated $CARGO_TOML -> $VERSION"

# tauri.conf.json — only replace the top-level version field (line 4)
TAURI_CONF="$ROOT/src-tauri/tauri.conf.json"
if [[ "$(uname)" == "Darwin" ]]; then
  sed -i '' "4s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" "$TAURI_CONF"
else
  sed -i "4s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" "$TAURI_CONF"
fi
echo "Updated $TAURI_CONF -> $VERSION"

echo "All three files synced to $VERSION"
