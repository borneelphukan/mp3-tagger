# 02 — Project Structure

## Top level

```text
xur-porisoy/
├── Cargo.toml            # single-package crate, release profile, winresource build-dep
├── Cargo.lock            # committed; CI/release builds use --locked
├── build.rs              # embeds assets/icon.ico + version info on Windows only
├── README.md             # user-facing docs (occasionally stale)
├── assets/
│   ├── icon.png          # window icon, Linux icon, macOS icon source
│   ├── icon.ico          # Windows executable icon
│   └── fonts/NotoSans-Regular.ttf
├── installer/
│   └── xur-porisoy.iss   # Inno Setup script, parameterised by /D defines
├── packaging/
│   ├── debian/           # .deb builder, desktop entry, AppStream, maintainer scripts
│   ├── apt/              # static repo generator + install.sh
│   ├── linux/            # tarball and AppImage builders
│   ├── rpm/              # RPM spec builder
│   ├── flatpak/          # Flatpak bundle builder
│   ├── arch/             # PKGBUILD + Docker wrapper
│   └── macos/            # universal .app/.dmg builder
├── src/
│   ├── main.rs
│   ├── models/           # data types and errors
│   ├── services/         # I/O, HTTP, LLM, settings
│   └── ui/               # egui views and app state
└── .github/workflows/    # ci.yml, audit.yml, release.yml
```

`target/`, `dist/`, and `*.log` are ignored. `Cargo.lock` is intentionally committed.

## `src/` ownership map

| File | Owns | Notes |
|---|---|---|
| `src/main.rs` | Process startup | Loads `assets/icon.png`, builds a 1200×800 viewport (min 800×600), caches `tokio::runtime::Handle`, runs eframe. |
| `src/models/mod.rs` | Re-exports | `pub use errors::*; pub use song::*;` |
| `src/models/song.rs` | Domain types | `AlbumArt`, `SongMetadata`, `Song`, `LLMSettings`, `AppSettings`, `ThemePreference`, plus two unit tests. |
| `src/models/errors.rs` | `AppError`, `Result<T>` | `thiserror`; several variants are `#[allow(dead_code)]`. |
| `src/services/mod.rs` | Re-exports | `artwork`, `http`, `images`, `llm`, `metadata`, `settings`. |
| `src/services/metadata.rs` | Tag read/write | `MetadataService::{read_metadata, write_metadata, load_song, load_songs}`. |
| `src/services/llm.rs` | LLM client | Prompt building, `/chat/completions` call, response extraction, `TOKIO_HANDLE`. |
| `src/services/artwork.rs` | iTunes lookup | `fetch_cover_art(title, artist, album)`. |
| `src/services/http.rs` | Shared HTTP client | One `reqwest::Client` in a `OnceLock`. |
| `src/services/images.rs` | Image decoding | `decode_thumbnail(bytes, max_side)` with decompression-bomb limits. |
| `src/services/settings.rs` | Settings persistence | `SettingsService::{new, load, save}`. |
| `src/ui/mod.rs` | `XurPorisoyApp` | Menu bar, theme application, message draining, About/Error windows, `eframe::App`. |
| `src/ui/state.rs` | `AppState`, messages, caches | `View`, `LibraryViewMode`, `AppMessage`, `TextureCache`, all state mutations. |
| `src/ui/grid_view.rs` | Library views | Toolbar, list table, grid tiles, row click handling. |
| `src/ui/detail_view.rs` | Metadata editor | Top bar, field editors, artwork panel, unsaved-changes dialog, LLM fetch spawn. |
| `src/ui/settings_view.rs` | LLM settings popup | Presets, custom model, local/API toggle, connection test. |

## Where to make common changes

| Change | Touch |
|---|---|
| Add or rename a metadata field | `SongMetadata` fields + `field_names` + `get_field`/`set_field`; then `extract_tag_data`/`write_tag_data` in `services/metadata.rs`. The detail view iterates `field_names()` automatically. |
| Change what the LLM is asked for | `services/llm.rs::build_prompt` **and** the `SongMetadata` field list in the prompt. |
| Change the system/user message shape | `services/llm.rs::call_llm`. |
| Change supported file extensions | `ui/grid_view.rs` (two `add_filter` calls) and `ui/mod.rs` (File → Add Songs). |
| Change artwork sizing or cache limits | `ui/grid_view.rs` (96), `ui/state.rs::detail_album_art` (320), `TextureCache::new` (512/8). |
| Change tag cleanup behaviour | `services/metadata.rs::write_tag_data`. |
| Add a menu item | `ui/mod.rs` menu bar. |
| Add a new top-level view | `View` in `ui/state.rs`, dispatch in `ui/mod.rs`, new module in `ui/mod.rs`. |
| Change release packaging | `.github/workflows/release.yml` plus the relevant `packaging/**` script. |

## Module dependency direction

```text
main → ui → services → models
         └──────────────┘
```

`models` depends only on `serde`/`thiserror`/`std`. `services` may use `models`. `ui` uses both. There is no `lib.rs`; this is a binary crate, so unit tests live inside modules (`#[cfg(test)]`).
