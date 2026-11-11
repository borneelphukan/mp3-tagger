# 07 — Features and End-to-End Flows

Each section traces a user-visible feature through UI, state, services, and background work. Use this when changing behaviour, not just a widget.

## 1. Add songs

**Entry points:** File → Add Songs; `➕ Add Songs` toolbar button.

1. `rfd::FileDialog::new().add_filter("Audio Files", &[...]).pick_files()` returns `Option<Vec<PathBuf>>`.
2. `AppState::add_songs(paths)` increments `loading_count` by the batch size and sets `pending_changes`.
3. It spawns `std::thread` `library-loader`, moving the paths and a cloned `Sender<AppMessage>`.
4. The thread loops: `MetadataService::load_song(path)` → `AppMessage::SongLoaded(result)`. It breaks if the receiver was dropped (window closed).
5. Each frame, `XurPorisoyApp::update` drains every queued message. `AppState::song_loaded` decrements `loading_count` once per message and either pushes the `Song` or sets an error banner.
6. While `loading_count > 0`, the toolbar shows `"N songs · loading M…"`, the empty state shows "Reading tags…", and the frame requests a repaint in 100 ms.

Properties:

- Songs appear incrementally, in completion order (which for a single thread is input order).
- Duplicate paths are added again; there is no dedupe.
- One bad file produces one error banner but the rest still load. A later error overwrites an earlier one.
- No folder picker and no recursive scan.
- `last_opened_folder` is never updated.

## 2. Browse the library

**List mode (default):** virtualised 7-column table. Row `#`, title (falls back to file name), artist, album, year, genre, track (`—`, `n`, or `n/total`). Clicking any cell opens the detail view.

**Grid mode:** responsive cover tiles, 112 px wide, 72 px art. Art is decoded lazily on the UI thread and cached. A missing or corrupt cover shows a `♫` button. Clicking the art or title opens detail.

**Counts:** the toolbar reports the current number of loaded songs and, while loading, the number still in flight.

**Clear All:** empties `songs` immediately. Texture caches and the current view are not reset; if the detail view was open it will reset to the grid on the next frame because the index is now out of range.

## 3. Open and edit a song

1. `open_song_detail(i)` sets `selected_index` and `current_view = Detail(i)`.
2. `DetailView::show` validates the index, clones the display strings and all 18 field values, and drops the `&Song` borrow.
3. The left pane renders 17 single-line editors plus a multiline lyrics editor.
4. Every change calls `update_metadata_field`, which mutates `songs[i].metadata`, sets `has_changes`, and sets global `pending_changes`.
5. The header shows `display_name` (`artist - title`, or file name) and the file name.
6. The Save button is disabled until `has_changes`.

The editor is not validated: any text is accepted for year, track, BPM, ISRC, and so on. Numbers are stored as strings.

## 4. Save tags

1. User clicks `💾 Save`.
2. `AppState::save_current_song` → `save_song_at(i)` → `MetadataService::write_metadata(&mut song)` on the **UI thread**.
3. The file is re-probed without audio properties, the primary (or first) tag is selected, text fields are replaced/deleted, artwork is replaced when changed, and fixed cleanup keys are stripped.
4. On success `Song::save_changes` snapshots metadata and clears the dirty flag; `pending_changes` is recomputed; the app returns to the grid.
5. On failure the error window appears and the detail view stays open with the dirty flag intact.

There is no save queue, progress indicator, undo, or backup. See [05-metadata-pipeline.md](05-metadata-pipeline.md) for the exact key behaviour.

## 5. Discard changes

Back on a dirty song → "Unsaved changes" dialog.

- **Yes:** `metadata = original_metadata`, `has_changes = false`, view → Grid.
- **No:** dialog closes, edits remain.

Quitting the app does **not** prompt; unsaved edits are lost. Settings are saved in `on_exit`.

## 6. Clear metadata

`Clear` replaces metadata with `SongMetadata::default()` and marks the song changed. Saving afterwards:

- removes every exposed text key,
- removes all embedded pictures (because artwork changed from `Some` to `None`),
- strips the fixed cleanup keys.

This is a destructive "blank the file" action, not a "revert to file" action. Reverting is done by leaving the detail view and answering Yes to the discard prompt.

## 7. Load / remove artwork

**Load Artwork:**

1. `rfd` file picker filtered to `png`, `jpg`, `jpeg`, `gif`, `bmp`.
2. `std::fs::read` the bytes on the UI thread.
3. MIME inferred from the lower-cased extension: `png` → `image/png`, `gif` → `image/gif`, `bmp` → `image/bmp`, anything else → `image/jpeg`.
4. Store `AlbumArt::new(bytes)` and the MIME, mark changed.
5. The preview decodes at 320 px on the next frame and caches by `(hash, 320)`.

The image is not validated at load time; a corrupt file shows "Failed to load artwork image" and will fail the decode permanently for that hash.

**Remove Artwork:** clears bytes and MIME and marks changed. On save, all pictures are removed.

Artwork is **not** resized before writing; the original bytes are embedded.

## 8. Fill metadata with an LLM

**Preconditions:** LLM configured (`enabled`, model, key or localhost), title and artist non-blank, no other fetch running.

1. `start_metadata_fetch(i)` sets `is_fetching = true` and `fetch_index = Some(i)`. The button label changes to `⏳ Fetching...`.
2. The UI clones `LlmService`, `LLMSettings`, `message_tx`, and the title/artist strings, then `get_tokio_handle().spawn(...)`.
3. The task calls `LlmService::fetch_metadata`:
   - builds the prompt,
   - `POST {base_url or https://api.openai.com/v1}/chat/completions`,
   - extracts `choices[0].message.content`,
   - slices from the first `{` to the last `}` and deserialises `SongMetadata`,
   - rejects an all-empty result as "song not found".
4. On success, the task calls `fetch_cover_art(title, artist, album)` on iTunes. Artwork failure is ignored; metadata still returns.
5. It sends `AppMessage::MetadataFetched(i, result)`.
6. The next frame calls `complete_metadata_fetch`, which clears the flags and either merges the metadata or shows "Could not fetch the song's metadata…".

Merge rules:

- If the returned `lyrics` is blank, keep the existing lyrics.
- If no new artwork was fetched, keep the existing artwork and MIME.
- Otherwise the whole metadata object is replaced (including blank fields), the song is marked changed, and `pending_changes` is set.

The user must still press Save.

## 9. Configure and test the LLM

AI → LLM Settings.

- Draft starts as a clone of the saved settings.
- The Local/API toggle only changes which fields are shown; `base_url` is created lazily with a localhost or OpenAI default.
- Model presets fill `model_name`; "Custom" leaves the free-text field.
- Save is gated on model + (local or key), forces `enabled = true`, writes `settings.json`, and closes.
- Test connection is gated the same way. It temporarily enables the settings for the request, spawns `test_connection`, and sends `AppMessage::LlmTestResult`.
- Result handling reuses `error_message`: success shows "Connection test successful!" in the Error window; failure shows "Connection test failed: …". Because the settings popup is open, its inline error filter also displays failures containing `connection`/`LLM`.

`test_connection` performs a real chat completion ("Test connection. Reply with 'OK'.") and discards the answer. It costs tokens on hosted providers.

## 10. Theme switching

View → Theme → Default / Dark / Light.

1. `selectable_value` mutates `settings.theme`.
2. `save_settings` persists immediately; failure shows the error window.
3. On the next frame, `update` detects `settings.theme != applied_theme`, calls `ctx.set_theme`, and records `applied_theme`.

The initial theme is applied in `XurPorisoyApp::new` before the first frame. System is the serde default.

## 11. About, errors, quit

- **About:** static window; its "Credits / Website / Release Notes" text is not clickable.
- **Errors:** a single centered window; any new error replaces the old one; `OK` clears it. Artwork load errors, tag read errors, save errors, settings errors, and LLM test results all share it.
- **Quit:** menu File → Quit sends `ViewportCommand::Close`. `on_exit` saves settings, ignoring errors. Unsaved song edits are not saved or prompted.

## 12. Startup and first run

1. Tokio runtime starts with 2 workers; `TOKIO_HANDLE` is set.
2. The window icon is decoded from embedded `assets/icon.png`; failure is non-fatal.
3. The viewport is 1200×800 (min 800×600), titled `XurPorisoy`.
4. `AppState::new` creates/loads the config directory and `settings.json`. A settings error panics (`expect` in `XurPorisoyApp::new`).
5. Style and theme are applied.
6. The empty library is shown.

There is no onboarding, no CLI argument handling, and no migration of older settings schemas beyond serde defaults.

## 13. Feature-to-code map

| Feature | UI | State | Service |
|---|---|---|---|
| Add songs | menu, toolbar | `add_songs`, `song_loaded` | `MetadataService::load_song` |
| List/grid | `GridView` | `library_view_mode` | — |
| Edit fields | `DetailView::show_metadata_fields` | `update_metadata_field` | — |
| Save | Save button | `save_current_song` | `MetadataService::write_metadata` |
| Clear | Clear button | `clear_current_metadata` | — |
| Artwork load/remove | `show_album_art` | inline mutation | `AlbumArt`, `images::decode_thumbnail` |
| Fill data | Fill Data button | `start_metadata_fetch`, `complete_metadata_fetch` | `LlmService`, `artwork::fetch_cover_art` |
| LLM settings | `SettingsView` | `save_settings` | `SettingsService` |
| Theme | View menu | `settings.theme` | — |
| Errors | `show_error_popup` | `error_message` | — |
