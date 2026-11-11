# 04 — Data Model and Contracts

This file is the quick reference for the Rust types that cross module boundaries. Field names are exact. There is no `lib.rs`; every type below lives in the binary crate.

## 1. Type map

| Type | Module | Role |
|---|---|---|
| `AlbumArt` | `models/song.rs` | `Arc<[u8]>` cover bytes plus a precomputed hash. |
| `SongMetadata` | `models/song.rs` | 18 text fields, artwork bytes, artwork MIME. Serialised to/from JSON. |
| `Song` | `models/song.rs` | A path, edited metadata, the last-saved snapshot, and a dirty flag. |
| `LLMSettings` | `models/song.rs` | Credentials, model, base URL, enabled flag. |
| `AppSettings` | `models/song.rs` | LLM settings, last folder, window size, theme. Persisted to `settings.json`. |
| `ThemePreference` | `models/song.rs` | `Light`, `Dark`, `System` (default). |
| `AppError` / `Result<T>` | `models/errors.rs` | `thiserror` error enum and crate result alias. |
| `View` | `ui/state.rs` | `Grid` or `Detail(usize)`. |
| `LibraryViewMode` | `ui/state.rs` | `List` (default) or `Grid`. Not persisted. |
| `AppMessage` | `ui/state.rs` | Background-to-UI message enum. |
| `TextureCache` | `ui/state.rs` | Bounded LRU-ish cache of decoded egui textures. |
| `AppState` | `ui/state.rs` | All library, view, settings, and cache state. |
| `XurPorisoyApp` | `ui/mod.rs` | eframe application; owns `AppState` plus dialog state. |
| `LlmService` | `services/llm.rs` | Cheap-to-clone wrapper around the shared `reqwest::Client`. |
| `MetadataService` | `services/metadata.rs` | Zero-sized namespace for tag I/O. |
| `SettingsService` | `services/settings.rs` | Owns the config directory path. |

## 2. `AlbumArt`

```rust
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "Vec<u8>", into = "Vec<u8>")]
pub struct AlbumArt {
    data: Arc<[u8]>,
    hash: u64,
}
```

- `AlbumArt::new(Vec<u8>)` computes `DefaultHasher` over the bytes **once**.
- `bytes()` returns `&[u8]`; `hash()` returns the cached fingerprint.
- Cloning is cheap: `Arc` bump plus a `u64` copy. UI code clones metadata freely.
- `PartialEq` therefore compares `Arc<[u8]>` contents and the hash. It is the basis of artwork-change detection in `write_metadata`.
- Serde goes through a plain `Vec<u8>`; an `AlbumArt` in JSON is just an array of numbers.
- `Debug` is hand-written to print only `len` and `hash`, never the bytes.
- The texture-cache key is `art.hash() ^ (u64::from(max_side) << 32)`, so the 96 px and 320 px variants cannot collide.

## 3. `SongMetadata`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct SongMetadata {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: String,
    pub year: String,
    pub genre: String,
    pub track_number: String,
    pub total_tracks: String,
    pub disc_number: String,
    pub total_discs: String,
    pub composer: String,
    pub lyricist: String,
    pub remixer: String,
    pub producer: String,
    pub comment: String,
    pub lyrics: String,
    pub bpm: String,
    pub isrc: String,
    pub album_art: Option<AlbumArt>,
    pub album_art_mime: Option<String>,
}
```

`#[serde(default)]` is load-bearing: the LLM is asked for only the text fields, and without it a valid response would fail while deserialising `album_art`. Every field is a `String` (even year/BPM/track), because tags are text and the UI does not validate numbers.

### Field indexing

`SongMetadata::field_names()` returns 18 labels in this exact order. `get_field(index)` and `set_field(index, value)` switch on the same indices.

| Index | Field | Index | Field |
|---|---|---|---|
| 0 | Title | 9 | Total Discs |
| 1 | Artist | 10 | Composer |
| 2 | Album | 11 | Lyricist |
| 3 | Album Artist | 12 | Remixer |
| 4 | Year | 13 | Producer |
| 5 | Genre | 14 | Comment |
| 6 | Track Number | 15 | Lyrics |
| 7 | Total Tracks | 16 | BPM |
| 8 | Disc Number | 17 | ISRC |

Index 15 is special in the detail view: `show_metadata_fields` filters it out of the single-line loop and renders it with a multiline editor. Out-of-range indices return `""` / do nothing.

### Methods

| Method | Behaviour |
|---|---|
| `is_empty()` | True only when all 18 strings are empty **and** `album_art.is_none()`. |
| `has_required_for_llm()` | `title` and `artist` both non-blank after `trim()`. |
| `field_names()` | Static slice of the 18 labels. |
| `get_field` / `set_field` | Index-based access used by the generic editor loop. |

## 4. `Song`

```rust
pub struct Song {
    pub path: PathBuf,
    pub metadata: SongMetadata,
    pub original_metadata: SongMetadata,
    pub has_changes: bool,
}
```

- `Song::new` clones the freshly read metadata into `original_metadata` and sets `has_changes = false`.
- `mark_changed()` sets `has_changes = true`; it never diffs fields.
- `save_changes()` copies `metadata` into `original_metadata` and clears `has_changes`. Called by `MetadataService::write_metadata` after a successful save.
- `file_name_str()` borrows the OS file name (or `"Unknown"`) to avoid per-frame allocation; `file_name()` allocates; `display_name()` returns `"artist - title"` or the file name.
- Unsaved-changes handling is a **snapshot restore**, not a diff: `confirm_discard_and_go_back()` replaces `metadata` with `original_metadata`.

## 5. Settings types

```rust
pub struct LLMSettings {
    pub api_key: String,
    pub model_name: String,
    pub base_url: Option<String>,
    pub enabled: bool,
}
```

`is_configured()` is `enabled && !model_name.is_empty() && (api_key non-empty || base_url starts with http://localhost, http://127.0.0.1, https://localhost, or https://127.0.0.1)`. Note it is a **prefix** test; a remote host is never accepted without a key.

```rust
pub struct AppSettings {
    pub llm: LLMSettings,
    pub last_opened_folder: Option<PathBuf>,
    pub window_size: Option<(f32, f32)>,
    pub theme: ThemePreference, // #[serde(default)] = System
}

pub enum ThemePreference { Light, Dark, #[default] System }
```

`last_opened_folder` and `window_size` exist in the model but the app does not currently read or write them; eframe's `persistence` feature handles window state separately.

## 6. `AppState` and UI state

```rust
pub enum View { Grid, Detail(usize) }
pub enum LibraryViewMode { #[default] List, Grid }

pub enum AppMessage {
    SongLoaded(Result<Song>),
    MetadataFetched(usize, Result<SongMetadata>),
    LlmTestResult(Result<()>),
}
```

Important: `AppMessage` uses `ui::state::Result<T> = std::result::Result<T, String>`, **not** `models::Result<T>`. Background work stringifies `AppError` at the thread boundary.

`AppState` fields:

| Group | Fields |
|---|---|
| Library | `songs: Vec<Song>`, `selected_index: Option<usize>`, `pending_changes: bool` |
| Navigation | `current_view: View`, `library_view_mode: LibraryViewMode` |
| Settings | `settings: AppSettings`, `settings_service: SettingsService` |
| Async | `llm_service: LlmService`, `message_tx: Sender<AppMessage>`, `loading_count: usize`, `is_fetching: bool`, `fetch_index: Option<usize>` |
| Dialogs | `error_message: Option<String>`, `confirm_discard_changes: bool` |
| Textures | `album_art_textures` (512 entries), `detail_album_art_textures` (8 entries) |

`XurPorisoyApp` adds: `message_rx`, `show_ai_settings`, `ai_settings_draft: LLMSettings`, `ai_settings_local_mode: bool`, `show_about`, `applied_theme`.

### Invariants

- `current_view == View::Detail(i)` is only valid while `i < songs.len()`. `DetailView::show` resets to `Grid` otherwise.
- `is_fetching == true` implies `fetch_index == Some(i)` for a song being filled; only one fill runs at a time.
- `loading_count` counts songs not yet delivered by `library-loader`; it is decremented per `SongLoaded`, including failures.
- `pending_changes` means "some song has unsaved changes"; it is recomputed on save and set on every edit.
- `error_message` is overloaded as a **status banner**: success of a connection test is stored here too.

## 7. Errors

`AppError` variants: `Io`, `AudioMetadataParse`, `AudioMetadataEncode`, `Http`, `Json`, `Image`, `Llm`, `Config`, `FileNotFound`, `UnsupportedFormat`, `NoInternet`, `SongNotFound`, `InvalidMetadata`. `FileNotFound`, `UnsupportedFormat`, and `NoInternet` are currently `#[allow(dead_code)]`.

`From` conversions exist for `std::io::Error`, `reqwest::Error`, `serde_json::Error`, and `image::ImageError`. Everything else is constructed with a `String` payload. The UI reports errors as `String` after `.map_err(|e| e.to_string())`.

## 8. Wire contracts

### `settings.json`

Pretty-printed `AppSettings`. Example:

```json
{
  "llm": {
    "api_key": "sk-...",
    "model_name": "gpt-4o-mini",
    "base_url": "https://api.openai.com/v1",
    "enabled": true
  },
  "last_opened_folder": null,
  "window_size": null,
  "theme": "System"
}
```

The API key is stored in **plaintext** with default file permissions.

### LLM request (`POST <base_url>/chat/completions`)

```json
{
  "model": "gpt-4o-mini",
  "messages": [
    { "role": "system", "content": "You are a music metadata expert. Return only valid JSON." },
    { "role": "user", "content": "<prompt>" }
  ],
  "temperature": 0.1,
  "max_tokens": 2000
}
```

`Authorization: Bearer <api_key>` is added only when `api_key` is non-empty.

### LLM response

Only `choices[0].message.content` is read. Extra fields are ignored; there is no `usage`, `finish_reason`, or error-envelope handling beyond the HTTP status check. See [08-ai-and-llm.md](08-ai-and-llm.md).

### iTunes search response

Only `results[].artworkUrl100` is read. No storefront, country, or explicit-content filtering is requested.

## 9. Unit tests

Two tests in `models/song.rs`:

1. `llm_response_without_art_deserializes` — proves `#[serde(default)]` lets `{"title":"Song","artist":"Artist"}` parse.
2. `album_art_round_trips_through_serde` — proves the `Vec<u8>` serde bridge preserves bytes and equality.

There are no integration tests.
