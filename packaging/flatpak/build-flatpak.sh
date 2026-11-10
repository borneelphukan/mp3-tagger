#!/usr/bin/env bash
#
# Build a Flatpak bundle for a Linux build of mp3-tagger.
#
# The app is packaged from an already-built binary, so the flathub runtime and
# SDK are the only build dependencies (flatpak-builder installs them for us).
#
# Environment:
#   VERSION   package version (required)
#   ARCH      x86_64 or aarch64 (required, used for the output file name)
#   BINARY    path to the release binary (default: target/release/mp3-tagger)
#   OUT_DIR   output directory (default: dist)
#
# Usage: VERSION=0.1.0 ARCH=x86_64 packaging/flatpak/build-flatpak.sh

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"

VERSION="${VERSION:?VERSION is required}"
ARCH="${ARCH:?ARCH is required}"
BINARY="${BINARY:-$ROOT/target/release/mp3-tagger}"
OUT_DIR="${OUT_DIR:-$ROOT/dist}"

APP_ID="io.github.borneelphukan.mp3Tagger"

die() { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

command -v flatpak >/dev/null 2>&1 || die "flatpak is required"
command -v flatpak-builder >/dev/null 2>&1 || die "flatpak-builder is required"
[ -f "$BINARY" ] || die "release binary not found at $BINARY"
BINARY="$(cd -- "$(dirname -- "$BINARY")" && pwd)/$(basename -- "$BINARY")"

STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

install -m 0755 "$BINARY" "$STAGE/mp3-tagger"
install -m 0644 "$ROOT/packaging/debian/mp3-tagger.desktop" "$STAGE/mp3-tagger.desktop"
install -m 0644 "$ROOT/packaging/debian/mp3-tagger.metainfo.xml" "$STAGE/mp3-tagger.metainfo.xml"
install -m 0644 "$ROOT/assets/icon.png" "$STAGE/mp3-tagger.png"

cat > "$STAGE/$APP_ID.yml" <<EOF
app-id: $APP_ID
runtime: org.freedesktop.Platform
runtime-version: '25.08'
sdk: org.freedesktop.Sdk
command: mp3-tagger
rename-desktop-file: mp3-tagger.desktop
rename-icon: mp3-tagger
finish-args:
  - --share=ipc
  - --share=network
  - --socket=wayland
  - --socket=fallback-x11
  - --device=dri
  - --filesystem=home
modules:
  - name: mp3-tagger
    buildsystem: simple
    build-commands:
      - install -Dm755 mp3-tagger /app/bin/mp3-tagger
      - install -Dm644 mp3-tagger.desktop /app/share/applications/mp3-tagger.desktop
      - install -Dm644 mp3-tagger.metainfo.xml /app/share/metainfo/$APP_ID.metainfo.xml
      - install -Dm644 mp3-tagger.png /app/share/icons/hicolor/256x256/apps/mp3-tagger.png
    sources:
      - type: file
        path: mp3-tagger
      - type: file
        path: mp3-tagger.desktop
      - type: file
        path: mp3-tagger.metainfo.xml
      - type: file
        path: mp3-tagger.png
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
OUTPUT="$OUT_DIR/MP3Tagger-${VERSION}-${ARCH}.flatpak"
flatpak build-bundle "$STAGE/repo" "$OUTPUT" "$APP_ID"

printf 'built %s\n' "$OUTPUT"
