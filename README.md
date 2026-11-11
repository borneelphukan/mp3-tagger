# XurPorisoy

A native Rust desktop application for editing MP3 metadata with LLM-powered metadata fetching.

## Installation

Prebuilt packages are published from the [Releases page](https://github.com/borneelphukan/xur-porisoy/releases).

### Ubuntu / Debian

```bash
curl -fsSL https://borneelphukan.github.io/xur-porisoy/install.sh | sudo bash
```

That script adds the signed apt repository, installs `xur-porisoy`, and can be run with flags:

```bash
curl -fsSL https://borneelphukan.github.io/xur-porisoy/install.sh | sudo bash -- --print     # show what it will do
curl -fsSL https://borneelphukan.github.io/xur-porisoy/install.sh | sudo bash -- --yes       # no prompts
curl -fsSL https://borneelphukan.github.io/xur-porisoy/install.sh | sudo bash -- --remove    # uninstall and remove the repository
```

Afterwards `apt` knows about the package, so later upgrades are just:

```bash
sudo apt update && sudo apt install --upgrade xur-porisoy
```

To do it by hand instead:

```bash
sudo mkdir -p /usr/share/keyrings
curl -fsSL https://borneelphukan.github.io/xur-porisoy/xur-porisoy-archive-keyring.gpg \
  | sudo tee /usr/share/keyrings/xur-porisoy.gpg >/dev/null

sudo tee /etc/apt/sources.list.d/xur-porisoy.sources >/dev/null <<'EOF'
Types: deb
URIs: https://borneelphukan.github.io/xur-porisoy/apt/
Suites: stable
Components: main
Architectures: amd64
Signed-By: /usr/share/keyrings/xur-porisoy.gpg
EOF

sudo apt update
sudo apt install xur-porisoy
```

To drop the repository again, delete those two files.

### Windows

Download `XurPorisoy-Setup-<version>.exe` from the releases page and run it. The installer offers
an optional Start Menu entry, a desktop shortcut, and adding the app to `PATH`; the application
itself installs to `C:\Program Files\XurPorisoy`.

A portable build is also attached to each release as `XurPorisoy-<version>-windows-x64.zip`.

## Features

- **List and Grid Views**: Browse all your songs in a compact list or metadata table
- **Detail View**: Edit all metadata fields with album artwork preview
- **LLM Integration**: Fetch metadata automatically using any OpenAI-compatible API
- **Batch Operations**: Add one or multiple songs at once
- **Full Metadata Support**: 34+ metadata fields including ID3v2, Vorbis Comments, APE tags
- **Album Artwork**: View, add, and remove cover art
- **Cross-Platform**: Native on Linux (Ubuntu) and Windows
- **Persistent Settings**: API keys and preferences saved locally

## Supported Audio Formats

- MP3 (ID3v2)
- FLAC (Vorbis Comments)
- OGG (Vorbis Comments)
- M4A/MP4 (iTunes-style)
- WAV
- AIFF
- APE (APE Tags)

## Metadata Fields

| Category | Fields |
|----------|--------|
| Basic | Title, Artist, Album, Album Artist, Year, Genre |
| Track Info | Track Number, Total Tracks, Disc Number, Total Discs |
| Credits | Composer, Lyricist, Remixer, Producer |
| Additional | Comment, Lyrics, BPM, ISRC |
| Artwork | Cover Art (embedded) |

## Building

### Prerequisites

- Rust 1.75+
- On Ubuntu: `sudo apt-get install pkg-config libx11-dev libxcb1-dev libxcb-composite0-dev libxcb-cursor-dev libxcb-image0-dev libxcb-icccm4-dev libxcb-keysyms1-dev libxcb-randr0-dev libxcb-render0-dev libxcb-render-util0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxcb-xinerama0-dev libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev`

### Build Commands

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run
cargo run
```

### Faster builds

- `cargo check` and `cargo clippy` are much faster than linking while iterating.
- Debug builds keep line-table-only debug info (`debug = 1`), which cuts the
  size of `target/` substantially compared with the default.
- Release builds use thin LTO and `panic = "abort"`: links are much faster than
  fat LTO and the resulting binary is smaller.
- For even faster local links, install `mold` or `lld` and point
  `RUSTFLAGS="-C link-arg=-fuse-ld=lld"` at it (not enabled by default because
  the linker is not available on every platform).
- The workspace never includes the `wgpu` renderer or every `image` decoder;
  both were removed because the app uses glow and only PNG/JPEG/GIF/BMP art.

### Packaging

```bash
# Regenerate assets/icon.png and assets/icon.ico

# Build dist/xur-porisoy_<version>_amd64.deb
packaging/debian/build-deb.sh

# Build a static apt repository into dist/../pages from the .deb files in dist/
packaging/apt/build-repo.py --debs dist --out pages --suite stable --component main \
    --origin xur-porisoy --label "XurPorisoy"

# Build a portable tarball (VERSION and ARCH are required)
VERSION=0.1.0 ARCH=x86_64 packaging/linux/build-tarball.sh

# Build an AppImage (x86_64 or aarch64)
VERSION=0.1.0 ARCH=x86_64 packaging/linux/build-appimage.sh

# Build an RPM package (requires rpmbuild)
VERSION=0.1.0 RPM_ARCH=x86_64 packaging/rpm/build-rpm.sh

# Build a Flatpak bundle (requires flatpak-builder)
VERSION=0.1.0 ARCH=x86_64 packaging/flatpak/build-flatpak.sh

# Build an Arch package (runs makepkg in Docker)
VERSION=0.1.0 ARCH=x86_64 packaging/arch/build-arch-pkg.sh

# Build the macOS disk image (macOS only, from a universal binary)
VERSION=0.1.0 packaging/macos/build-dmg.sh

# Build the Windows installer (needs Inno Setup and a release build)
ISCC.exe /DMyAppVersion=0.1.0 /DMyVersionInfo=0.1.0.0 /DMyArch=x64 ^
    /DMySourceExe=target\x86_64-pc-windows-msvc\release\xur-porisoy.exe ^
    /DMyOutputBaseFilename=XurPorisoy-Setup-0.1.0-x64 installer\xur-porisoy.iss
```

Every push, pull request and manual run of `.github/workflows/release.yml` passes
a quality gate (`fmt`, `clippy -D warnings`, tests), builds every Linux, Windows
and macOS artifact, verifies the full deliverable set and publishes the bundle
with a `SHA256SUMS` manifest as a workflow artifact. Pushing a `v*` tag does the
same and additionally attaches every file to a GitHub Release. The apt
repository is republished to GitHub Pages on non-pull-request runs.

## Website

The apt repository is republished to GitHub Pages on non-pull-request runs of
the release workflow. Pages hosts only the apt repository and `install.sh`.

> **First deployment note:** the workflow enables GitHub Pages automatically via
> `actions/configure-pages` with `enablement: true`. If that ever fails (e.g. a
> restrictive organisation policy), enable Pages manually under repository
> **Settings → Pages → Build and deployment → GitHub Actions** and re-run the
> workflow.

## Configuration

On first run, the app creates a config directory:
- Linux: `~/.config/xur-porisoy/settings.json`
- Windows: `%APPDATA%\xur-porisoy\settings.json`

### LLM Setup

1. Click **AI** → **LLM Settings** in the menu bar
2. Enter your API key and model name (e.g., `gpt-4o-mini`, `claude-3-haiku`, `llama3`)
3. Optionally set a custom base URL for an OpenAI-compatible API
4. Click **Save** or **Cancel**

The Fill Data button will be enabled when:
- LLM is configured
- Song has both Title and Artist fields filled
- Not currently fetching

## Usage

1. **Add Songs**: Click "Add Songs" and select one or multiple audio files
2. **View Details**: Click a song title in either library view
3. **Edit Metadata**: Modify fields in the detail view
4. **Fetch Metadata**: Enter Title + Artist, click "Fill Data"
5. **Save Changes**: Click "Save" to write tags to file
6. **Navigate**: Use "Back to Grid" to return to library view

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| Double-click | Open song detail |
| Enter (in detail) | Save changes |
| Escape | Go back / Close dialog |

## Architecture

```
src/
├── main.rs           # Entry point
├── models/           # Data structures
│   ├── song.rs       # Song, SongMetadata, LLMSettings
│   └── errors.rs     # Error types
├── services/         # Business logic
│   ├── metadata.rs   # Audio file reading/writing (lofty)
│   ├── llm.rs        # LLM API communication (reqwest)
│   └── settings.rs   # Persistent configuration
└── ui/               # User interface (egui)
    ├── state.rs      # Application state management
    ├── grid_view.rs  # Song library grid
    ├── detail_view.rs# Song metadata editor
    └── settings_view.rs # LLM configuration
```

## Dependencies

- **eframe/egui** - Immediate mode GUI
- **lofty** - Audio metadata handling
- **reqwest** - HTTP client for LLM APIs
- **serde** - Serialization
- **tokio** - Async runtime
- **directories** - Cross-platform config paths
- **image** - Album artwork processing
- **rfd** - Native file dialogs

## License

MIT License
