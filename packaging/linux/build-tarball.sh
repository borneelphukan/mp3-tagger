#!/usr/bin/env bash
#
# Build a portable tar.gz for a Linux build of mp3-tagger.
#
# Environment:
#   VERSION   package version (required)
#   ARCH      packaging architecture, e.g. x86_64, aarch64, riscv64 (required)
#   BINARY    path to the release binary (default: target/release/mp3-tagger)
#   OUT_DIR   output directory (default: dist)
#
# Usage: VERSION=0.1.0 ARCH=x86_64 packaging/linux/build-tarball.sh

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"

VERSION="${VERSION:?VERSION is required}"
ARCH="${ARCH:?ARCH is required}"
BINARY="${BINARY:-$ROOT/target/release/mp3-tagger}"
OUT_DIR="${OUT_DIR:-$ROOT/dist}"

die() { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

[ -f "$BINARY" ] || die "release binary not found at $BINARY"
BINARY="$(cd -- "$(dirname -- "$BINARY")" && pwd)/$(basename -- "$BINARY")"

NAME="MP3Tagger-${VERSION}-linux-${ARCH}"
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

DEST="$STAGE/$NAME"
install -d -m 0755 "$DEST"
install -m 0755 "$BINARY" "$DEST/mp3-tagger"
install -m 0644 "$ROOT/README.md" "$DEST/README.md"
if [ -f "$ROOT/LICENSE" ]; then
    install -m 0644 "$ROOT/LICENSE" "$DEST/LICENSE"
fi
install -m 0644 "$ROOT/packaging/debian/mp3-tagger.desktop" "$DEST/mp3-tagger.desktop"
install -m 0644 "$ROOT/assets/icon.png" "$DEST/mp3-tagger.png"

mkdir -p "$OUT_DIR"
ARCHIVE="$OUT_DIR/$NAME.tar.gz"
tar -C "$STAGE" --owner=0 --group=0 --numeric-owner -czf "$ARCHIVE" "$NAME"

printf 'built %s\n' "$ARCHIVE"
