# MP3 Tagger

A native Rust desktop application for editing MP3 metadata with LLM-powered metadata fetching.

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
| Credits | Composer, Lyricist, Conductor, Remixer, Producer, Arranger, Engineer, Mixer |
| Additional | Comment, Lyrics, BPM, ISRC |
| MusicBrainz | Track ID, Artist ID, Album ID, Album Artist ID, Release Group ID, Work ID |
| AcoustID | Fingerprint, ID |
| ReplayGain | Track Gain, Track Peak, Album Gain, Album Peak |
| Artwork | Cover Art (embedded) |

## Building

### Prerequisites

- Rust 1.75+
- On Ubuntu: `sudo apt-get install libssl-dev pkg-config libasound2-dev libxcb-composite0-dev libxcb-cursor-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev`

### Build Commands

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run
cargo run
```

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
