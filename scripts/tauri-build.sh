#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TAURI_CACHE_DIR="${XDG_CACHE_HOME:-$HOME/.cache}/tauri"

mkdir -p "$TAURI_CACHE_DIR"

LINUXDEPLOY_BIN="$TAURI_CACHE_DIR/linuxdeploy-x86_64.AppImage"
LINUXDEPLOY_URL="https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage"

if [[ ! -f "$LINUXDEPLOY_BIN" ]] || [[ ! -x "$LINUXDEPLOY_BIN" ]]; then
  curl -fsSL "$LINUXDEPLOY_URL" -o "$LINUXDEPLOY_BIN"
  chmod +x "$LINUXDEPLOY_BIN"
fi

export PKG_CONFIG_PATH="$ROOT_DIR/src-tauri/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"

tauri build "$@"
