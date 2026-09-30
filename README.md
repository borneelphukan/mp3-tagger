# MP3 Tagger

A native Rust desktop application for editing MP3 metadata with LLM-powered metadata fetching.

## Installation

Prebuilt packages are published from the [Releases page](https://github.com/borneelphukan/mp3-tagger/releases).

### Ubuntu / Debian

```bash
curl -fsSL https://borneelphukan.github.io/mp3-tagger/install.sh | sudo bash
```

That script adds the signed apt repository, installs `mp3-tagger`, and can be run with flags:

```bash
curl -fsSL https://borneelphukan.github.io/mp3-tagger/install.sh | sudo bash -- --print     # show what it will do
curl -fsSL https://borneelphukan.github.io/mp3-tagger/install.sh | sudo bash -- --yes       # no prompts
curl -fsSL https://borneelphukan.github.io/mp3-tagger/install.sh | sudo bash -- --remove    # uninstall and remove the repository
```

Afterwards `apt` knows about the package, so later upgrades are just:

```bash
sudo apt update && sudo apt install --upgrade mp3-tagger
```

To do it by hand instead:

```bash
sudo mkdir -p /usr/share/keyrings
curl -fsSL https://borneelphukan.github.io/mp3-tagger/mp3-tagger-archive-keyring.gpg \
  | sudo tee /usr/share/keyrings/mp3-tagger.gpg >/dev/null

sudo tee /etc/apt/sources.list.d/mp3-tagger.sources >/dev/null <<'EOF'
Types: deb
URIs: https://borneelphukan.github.io/mp3-tagger/apt/
Suites: stable
Components: main
Architectures: amd64
Signed-By: /usr/share/keyrings/mp3-tagger.gpg
EOF

sudo apt update
sudo apt install mp3-tagger
```

To drop the repository again, delete those two files.

### Windows

Download `MP3Tagger-Setup-<version>.exe` from the releases page and run it. The installer offers
an optional Start Menu entry, a desktop shortcut, and adding the app to `PATH`; the application
itself installs to `C:\Program Files\MP3 Tagger`.

A portable build is also attached to each release as `MP3Tagger-<version>-windows-x64.zip`.

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

### Packaging

```bash
# Regenerate assets/icon.png and assets/icon.ico

# Build dist/mp3-tagger_<version>_amd64.deb
packaging/debian/build-deb.sh

# Build a static apt repository into dist/../pages from the .deb files in dist/
packaging/apt/build-repo.py --debs dist --out pages --suite stable --component main \
    --origin mp3-tagger --label "MP3 Tagger"

# Build the NSIS installer (needs makensis and a release build)
makensis -DAPP_VERSION=0.1.0 -DAPP_VERSION_NUM=0.1.0.0 \
    -DOUTPUT_NAME=MP3Tagger-Setup-0.1.0.exe installer/mp3-tagger.nsi
```

Pushing a `v*` tag runs `.github/workflows/windows-installer.yml` and
`.github/workflows/debian.yml`, which attach the `.exe` and `.deb` to the release and republish the
apt repository to GitHub Pages.

## Configuration

On first run, the app creates a config directory:
- Linux: `~/.config/mp3-tagger/settings.json`
- Windows: `%APPDATA%\mp3-tagger\settings.json`

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
