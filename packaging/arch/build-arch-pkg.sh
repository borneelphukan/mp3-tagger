#!/usr/bin/env bash
#
# Build an Arch Linux package for a Linux build of mp3-tagger.
#
# makepkg only runs on Arch, so this script uses Docker: the official image for
# x86_64 and an Arch Linux ARM image for aarch64. The binary itself is packaged
# as-is, so no compilation happens inside the container.
#
# Environment:
#   VERSION   package version (required)
#   ARCH      x86_64 or aarch64 (required)
#   BINARY    path to the release binary (default: target/release/mp3-tagger)
#   OUT_DIR   output directory (default: dist)
#
# Usage: VERSION=0.1.0 ARCH=x86_64 packaging/arch/build-arch-pkg.sh

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"

VERSION="${VERSION:?VERSION is required}"
ARCH="${ARCH:?ARCH is required}"
BINARY="${BINARY:-$ROOT/target/release/mp3-tagger}"
OUT_DIR="${OUT_DIR:-$ROOT/dist}"

die() { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

case "$ARCH" in
    x86_64) IMAGE="archlinux:base-devel" ;;
    aarch64) IMAGE="menci/archlinuxarm:base-devel" ;;
    *) die "unsupported Arch architecture: $ARCH" ;;
esac

command -v docker >/dev/null 2>&1 || die "docker is required to run makepkg"
[ -f "$BINARY" ] || die "release binary not found at $BINARY"
BINARY="$(cd -- "$(dirname -- "$BINARY")" && pwd)/$(basename -- "$BINARY")"

# makepkg needs a writable build directory owned by a non-root user.
WORK="$ROOT/target/arch-pkg-$ARCH"
rm -rf "$WORK"
mkdir -p "$WORK"
cp "$ROOT/packaging/arch/PKGBUILD" "$WORK/PKGBUILD"

BINARY_REL="${BINARY#"$ROOT/"}"
[ "$BINARY_REL" != "$BINARY" ] || die "BINARY must live inside the repository"

docker run --rm \
    -v "$ROOT:/work" \
    -w "/work/target/arch-pkg-$ARCH" \
    -e "PKGVER=$VERSION" \
    -e "BINARY=/work/$BINARY_REL" \
    -e "ROOT=/work" \
    "$IMAGE" \
    bash -euo pipefail -c '
        if ! id builder >/dev/null 2>&1; then
            useradd -m builder
        fi
        chown -R builder:"$(id -gn builder)" .
        su -s /bin/bash builder -c \
            "PKGVER=$(printf %q "$PKGVER") BINARY=$(printf %q "$BINARY") ROOT=$(printf %q "$ROOT") makepkg -f --nodeps --noconfirm"
    '

mkdir -p "$OUT_DIR"
find "$WORK" -maxdepth 1 -type f -name '*.pkg.tar.*' -exec cp -f {} "$OUT_DIR/" \;

PACKAGE="$(find "$WORK" -maxdepth 1 -type f -name "*-$ARCH.pkg.tar.*" | head -n1)"
[ -n "$PACKAGE" ] || die "makepkg did not produce a package"

printf 'built %s\n' "$OUT_DIR/$(basename "$PACKAGE")"
