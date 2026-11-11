#!/usr/bin/env bash
#
# Build a Flatpak bundle for a Linux build of xur-porisoy.
#
# The app is packaged from an already-built binary, so the flathub runtime and
# SDK are the only build dependencies (flatpak-builder installs them for us).
#
# Environment:
#   VERSION   package version (required)
#   ARCH      x86_64 or aarch64 (required, used for the output file name)
#   BINARY    path to the release binary (default: target/release/xur-porisoy)
#   OUT_DIR   output directory (default: dist)
#
# Usage: VERSION=0.1.0 ARCH=x86_64 packaging/flatpak/build-flatpak.sh

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"

VERSION="${VERSION:?VERSION is required}"
ARCH="${ARCH:?ARCH is required}"
BINARY="${BINARY:-$ROOT/target/release/xur-porisoy}"
OUT_DIR="${OUT_DIR:-$ROOT/dist}"

APP_ID="io.github.borneelphukan.XurPorisoy"

die() { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

command -v flatpak >/dev/null 2>&1 || die "flatpak is required"
command -v flatpak-builder >/dev/null 2>&1 || die "flatpak-builder is required"
[ -f "$BINARY" ] || die "release binary not found at $BINARY"
BINARY="$(cd -- "$(dirname -- "$BINARY")" && pwd)/$(basename -- "$BINARY")"

STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

install -m 0755 "$BINARY" "$STAGE/xur-porisoy"
install -m 0644 "$ROOT/packaging/debian/xur-porisoy.desktop" "$STAGE/xur-porisoy.desktop"
install -m 0644 "$ROOT/packaging/debian/xur-porisoy.metainfo.xml" "$STAGE/xur-porisoy.metainfo.xml"
install -m 0644 "$ROOT/assets/icon.png" "$STAGE/xur-porisoy.png"

cat > "$STAGE/$APP_ID.yml" <<EOF
app-id: $APP_ID
runtime: org.freedesktop.Platform
runtime-version: '25.08'
sdk: org.freedesktop.Sdk
command: xur-porisoy
rename-desktop-file: xur-porisoy.desktop
rename-icon: xur-porisoy
finish-args:
  - --share=ipc
  - --share=network
  - --socket=wayland
  - --socket=fallback-x11
  - --device=dri
  - --filesystem=home
modules:
  - name: xur-porisoy
    buildsystem: simple
    build-commands:
      - install -Dm755 xur-porisoy /app/bin/xur-porisoy
      - install -Dm644 xur-porisoy.desktop /app/share/applications/xur-porisoy.desktop
      - install -Dm644 xur-porisoy.metainfo.xml /app/share/metainfo/$APP_ID.metainfo.xml
      - install -Dm644 xur-porisoy.png /app/share/icons/hicolor/256x256/apps/xur-porisoy.png
    sources:
      - type: file
        path: xur-porisoy
      - type: file
        path: xur-porisoy.desktop
      - type: file
        path: xur-porisoy.metainfo.xml
      - type: file
        path: xur-porisoy.png
EOF

flatpak remote-add --if-not-exists --user flathub \
    https://flathub.org/repo/flathub.flatpakrepo

flatpak-builder \
    --user \
    --assumeyes \
    --disable-rofiles-fuse \
    --force-clean \
    --install-deps-from=flathub \
    --repo="$STAGE/repo" \
    "$STAGE/build" \
    "$STAGE/$APP_ID.yml"

mkdir -p "$OUT_DIR"
OUTPUT="$OUT_DIR/XurPorisoy-${VERSION}-${ARCH}.flatpak"
flatpak build-bundle "$STAGE/repo" "$OUTPUT" "$APP_ID"

printf 'built %s\n' "$OUTPUT"
