# 03 — Architecture

## 1. Process and thread model

```text
┌────────────────────────────────────────────────────────────────────┐
│ Native process                                                      │
│                                                                    │
│  ┌──────────────────────────────┐                                  │
│  │ Main / UI thread             │                                  │
│  │ eframe + egui (glow)         │                                  │
│  │ XurPorisoyApp → AppState     │                                  │
│  │ egui textures created here   │                                  │
│  └──────────┬───────────────────┘                                  │
│             │ mpsc::Sender<AppMessage>                              │
│             ▼                                                      │
│  ┌──────────────────────────────┐   ┌───────────────────────────┐   │
│  │ std::thread "library-loader" │   │ tokio runtime (2 workers) │   │
│  │ lofty tag parsing, 1 msg/    │   │ LLM HTTP + iTunes HTTP     │   │
│  │ song, created per Add Songs  │   │ spawned via cached Handle  │   │
│  └──────────┬───────────────────┘   └──────────┬────────────────┘   │
│             │                                  │                    │
│             └──────────► Receiver<AppMessage> ◄┘                    │
│                        drained once per frame                       │
└────────────────────────────────────────────────────────────────────┘
```

Key properties:

- **No worker owns a shared mutable service.** `MetadataService` is a zero-sized namespace; `LlmService` is cheap to clone and wraps a shared `reqwest::Client`.
- **The library loader is created per `add_songs` call.** It streams one `SongLoaded` message per path and exits. A failed `send` (window closed) stops the loop.
- **The tokio runtime is process-wide.** `main` is `#[tokio::main(worker_threads = 2)]`; `main.rs` stores `Handle::current()` in `services::llm::TOKIO_HANDLE`. UI code calls `get_tokio_handle()` and `handle.spawn(...)`.
- **Messages are pulled, not pushed.** There is no egui wake-up callback. `XurPorisoyApp::update` drains `try_recv()` and, while `is_fetching` or `loading_count > 0`, requests a repaint every 100 ms.
- **Textures are created only on the UI thread** in `TextureCache::get_or_load` via `ctx.load_texture`.
- **Saving is synchronous on the UI thread.** `save_current_song` → `save_song_at` → `MetadataService::write_metadata` performs filesystem I/O inside the frame.

## 2. Startup sequence

1. `tokio::main` starts the 2-worker runtime.
2. `load_icon()` decodes `assets/icon.png` with the `image` crate and wraps it in `Arc<egui::IconData>`.
3. A viewport is built: 1200×800, minimum 800×600, title `XurPorisoy`, optional icon.
4. `main.rs` stores `tokio::runtime::Handle::current()` in `TOKIO_HANDLE`.
5. `eframe::run_native` constructs `XurPorisoyApp::new`:
   - creates an `mpsc` channel;
   - `AppState::new` loads `settings.json` (creating the config dir if needed);
   - applies spacing style for both themes and the saved theme;
   - clones the saved `LLMSettings` into the dialog draft;
   - derives `ai_settings_local_mode` from `base_url` containing `localhost` or `127.0.0.1`.
6. The first frame renders the library empty state.

There is no splash screen, no CLI argument handling, and no first-run wizard.

## 3. Frame lifecycle

`impl eframe::App for XurPorisoyApp` is in `src/ui/mod.rs` and implements `fn update(&mut self, ctx, _frame)`.

Order per frame:

1. **Theme sync.** If `settings.theme != applied_theme`, call `ctx.set_theme(...)` and remember it.
2. **Drain messages.** `while let Ok(msg) = message_rx.try_recv()` handles every queued message:
   - `SongLoaded` → decrement `loading_count`, push or set `error_message`;
   - `MetadataFetched` → clear fetching flags, merge results, mark changed;
   - `LlmTestResult` → reuse `error_message` as a status banner ("Connection test successful!") or error.
3. **Menu bar** (`TopBottomPanel::top("menu_bar")`): File, AI, View → Theme, Help, plus a right-aligned "📚 Library" label when in the grid.
4. **Main content** by `View`:
   - `View::Grid` → `GridView::show(ctx, &mut state)`;
   - `View::Detail(i)` → `DetailView::show(ctx, &mut state)`; if the index is stale it resets to `View::Grid`.
5. **AI settings popup** when `show_ai_settings`.
6. **About popup** when `show_about`.
7. **Error popup** whenever `state.error_message` is `Some`.
8. **Repaint polling** while async work is running.

Because messages are handled before drawing, a result sent during frame *N* is normally visible in frame *N+1*.

## 4. State ownership

`XurPorisoyApp` (in `ui/mod.rs`) owns:

| Field | Purpose |
|---|---|
| `state: AppState` | All library/view/settings state. |
| `message_rx` | Receiver for background results. |
| `show_ai_settings` | Popup visibility. |
| `ai_settings_draft` | Edited copy of `LLMSettings`; only committed on Save. |
| `ai_settings_local_mode` | Which tab the settings dialog shows. |
| `show_about` | About window visibility. |
| `applied_theme` | Last theme pushed into egui. |

`AppState` (in `ui/state.rs`) owns `songs`, `current_view`, `selected_index`, `settings`, `llm_service`, `settings_service`, both `TextureCache`s, loading/fetch flags, the message `Sender`, and `confirm_discard_changes`.

`Song` owns a path, current metadata, an `original_metadata` snapshot, and a `has_changes` flag. `original_metadata` is what the unsaved-changes dialog restores and what artwork-change detection compares against.

## 5. Message flow

```text
Add Songs
  └─ AppState::add_songs(paths)
       ├─ loading_count += n; pending_changes = true
       └─ spawn "library-loader"
            └─ for path: MetadataService::load_song → AppMessage::SongLoaded
                 └─ UI drains → state.song_loaded → songs.push(song)

Fill Data
  └─ DetailView: start_metadata_fetch(idx)
       └─ tokio spawn:
            LlmService::fetch_metadata
            └─ artwork::fetch_cover_art (best effort)
            └─ AppMessage::MetadataFetched(idx, result)
                 └─ complete_metadata_fetch merges lyrics/art, marks changed

Test connection
  └─ SettingsView: tokio spawn → LlmService::test_connection
       └─ AppMessage::LlmTestResult
```

## 6. Borrow and mutation patterns

- The detail view clones every value it needs (title, artist, fields, artwork handle) before the UI closures, then drops the `&Song`. This keeps `&mut AppState` available for buttons.
- The grid grid view splits borrows: `let songs = &state.songs; let textures = &mut state.album_art_textures;` so tiles do not clone artwork or titles.
- Clicked indices are collected into `clicked_idx` and applied after the virtualised iterator finishes.
- `AlbumArt` is `Arc`-backed, so cloning it to move into an async task does not copy pixels.

## 7. Failure behaviour

- A settings load/save error becomes a user-visible `error_message`.
- A missing LLM handle (only possible if `main` never stored it) ends the fetch and shows "The async runtime is not available".
- A failed tag parse is reported per song and does not abort the rest of the batch.
- Failed artwork decoding is cached in `TextureCache::failed`; the UI shows the placeholder or "Failed to load artwork image".
- Failed LLM/artwork calls are non-fatal for the metadata result: artwork is best effort, and a failed LLM fetch shows a generic "enter the metadata manually" message.
- There is no retry, backoff, cancellation token, or timeout beyond the shared client's timeouts.
