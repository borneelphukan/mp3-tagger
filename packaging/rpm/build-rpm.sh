#!/usr/bin/env bash
#
# Build a binary RPM for a Linux build of mp3-tagger.
#
# The binary is packaged as-is, so this works for native and cross-compiled
# targets as long as rpmbuild is available on the host.
#
# Environment:
#   VERSION    package version (required)
#   RPM_ARCH   x86_64, aarch64 or riscv64 (required)
#   BINARY     path to the release binary (default: target/release/mp3-tagger)
#   OUT_DIR    output directory (default: dist)
#
# Usage: VERSION=0.1.0 RPM_ARCH=x86_64 packaging/rpm/build-rpm.sh

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"

VERSION="${VERSION:?VERSION is required}"
RPM_ARCH="${RPM_ARCH:?RPM_ARCH is required}"
BINARY="${BINARY:-$ROOT/target/release/mp3-tagger}"
OUT_DIR="${OUT_DIR:-$ROOT/dist}"

die() { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

command -v rpmbuild >/dev/null 2>&1 || die "rpmbuild is required (install the 'rpm' package)"
[ -f "$BINARY" ] || die "release binary not found at $BINARY"
BINARY="$(cd -- "$(dirname -- "$BINARY")" && pwd)/$(basename -- "$BINARY")"

TOPDIR="$(mktemp -d)"
trap 'rm -rf "$TOPDIR"' EXIT
mkdir -p "$TOPDIR/BUILD" "$TOPDIR/BUILDROOT" "$TOPDIR/RPMS" \
    "$TOPDIR/SOURCES" "$TOPDIR/SPECS" "$TOPDIR/SRPMS"

SPEC="$TOPDIR/SPECS/mp3-tagger.spec"
cat > "$SPEC" <<EOF
%global debug_package %{nil}
%global __strip /bin/true

Name:           mp3-tagger
Version:        $VERSION
Release:        1
Summary:        MP3 metadata editor with LLM-powered metadata fetching
License:        MIT
URL:            https://github.com/borneelphukan/mp3-tagger

%description
MP3 Tagger is a native desktop application for editing metadata of MP3,
FLAC, OGG, M4A, WAV, AIFF and APE files. It offers list and grid library
views, a detail view covering more than 34 metadata fields, embedded album
artwork handling and batch adding of files. Metadata can be filled in
automatically through any OpenAI compatible API.

%prep

%build

%install
set -e
install -Dm0755 "$BINARY" %{buildroot}%{_bindir}/mp3-tagger
install -Dm0644 "$ROOT/packaging/debian/mp3-tagger.desktop" \
    %{buildroot}%{_datadir}/applications/mp3-tagger.desktop
install -Dm0644 "$ROOT/packaging/debian/mp3-tagger.metainfo.xml" \
    %{buildroot}%{_datadir}/metainfo/mp3-tagger.metainfo.xml
install -Dm0644 "$ROOT/assets/icon.png" \
    %{buildroot}%{_datadir}/icons/hicolor/256x256/apps/mp3-tagger.png
install -Dm0644 "$ROOT/README.md" \
    %{buildroot}%{_datadir}/doc/mp3-tagger/README.md

%post
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database -q %{_datadir}/applications || :
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q -f -t %{_datadir}/icons/hicolor || :
fi

%postun
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database -q %{_datadir}/applications || :
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q -f -t %{_datadir}/icons/hicolor || :
fi

%files
%{_bindir}/mp3-tagger
%{_datadir}/applications/mp3-tagger.desktop
%{_datadir}/metainfo/mp3-tagger.metainfo.xml
%{_datadir}/icons/hicolor/256x256/apps/mp3-tagger.png
%{_datadir}/doc/mp3-tagger/README.md
EOF

rpmbuild -bb --target "$RPM_ARCH" --define "_topdir $TOPDIR" "$SPEC"

mkdir -p "$OUT_DIR"
find "$TOPDIR/RPMS" -type f -name '*.rpm' -exec cp -f {} "$OUT_DIR/" \;

printf 'built %s\n' "$OUT_DIR/mp3-tagger-${VERSION}-1.${RPM_ARCH}.rpm"
