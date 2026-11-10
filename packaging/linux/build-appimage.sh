#!/usr/bin/env bash
#
# Build an AppImage for a Linux build of mp3-tagger.
#
# Environment:
#   VERSION        package version (required)
#   ARCH           x86_64 or aarch64 (required)
#   BINARY         path to the release binary (default: target/release/mp3-tagger)
#   OUT_DIR        output directory (default: dist)
#   APPIMAGETOOL   path to a pre-downloaded appimagetool (optional)
#
# Usage: VERSION=0.1.0 ARCH=x86_64 packaging/linux/build-appimage.sh

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"

VERSION="${VERSION:?VERSION is required}"
ARCH="${ARCH:?ARCH is required}"
BINARY="${BINARY:-$ROOT/target/release/mp3-tagger}"
OUT_DIR="${OUT_DIR:-$ROOT/dist}"

die() { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

case "$ARCH" in
    x86_64|aarch64) ;;
    *) die "unsupported AppImage architecture: $ARCH" ;;
esac

[ -f "$BINARY" ] || die "release binary not found at $BINARY"
BINARY="$(cd -- "$(dirname -- "$BINARY")" && pwd)/$(basename -- "$BINARY")"

STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

APPDIR="$STAGE/AppDir"
install -d -m 0755 \
    "$APPDIR/usr/bin" \
    "$APPDIR/usr/share/applications" \
    "$APPDIR/usr/share/icons/hicolor/256x256/apps"

install -m 0755 "$BINARY" "$APPDIR/usr/bin/mp3-tagger"
install -m 0644 "$ROOT/packaging/debian/mp3-tagger.desktop" "$APPDIR/mp3-tagger.desktop"
install -m 0644 "$ROOT/assets/icon.png" "$APPDIR/mp3-tagger.png"
install -m 0644 "$ROOT/packaging/debian/mp3-tagger.desktop" \
    "$APPDIR/usr/share/applications/mp3-tagger.desktop"
install -m 0644 "$ROOT/assets/icon.png" \
    "$APPDIR/usr/share/icons/hicolor/256x256/apps/mp3-tagger.png"

cat > "$APPDIR/AppRun" <<'EOF'
#!/bin/sh
HERE="$(dirname "$(readlink -f "$0")")"
exec "$HERE/usr/bin/mp3-tagger" "$@"
EOF
chmod 0755 "$APPDIR/AppRun"

TOOL="${APPIMAGETOOL:-}"
if [ -z "$TOOL" ]; then
    TOOL="$STAGE/appimagetool"
    URL="https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-${ARCH}.AppImage"
    printf 'downloading %s\n' "$URL"
    curl -fsSL "$URL" -o "$TOOL"
    chmod 0755 "$TOOL"
fi

mkdir -p "$OUT_DIR"
OUTPUT="$OUT_DIR/MP3Tagger-${VERSION}-${ARCH}.AppImage"

ARCH="$ARCH" APPIMAGE_EXTRACT_AND_RUN=1 "$TOOL" --no-appstream "$APPDIR" "$OUTPUT"

printf 'built %s\n' "$OUTPUT"
