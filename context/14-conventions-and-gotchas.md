# 14 — Conventions, Gotchas, and Platform Traps

This file collects the non-obvious rules, known bugs, and documentation drift that will bite a future change. Verify each item against the code before relying on it.

## 1. Code conventions

- **Binary crate, no `lib.rs`.** Module tree is `models`, `services`, `ui`; tests are `#[cfg(test)]` inside modules.
- **Service namespaces, not trait objects.** `MetadataService` is a unit struct; `LlmService` wraps a shared client and derives `Clone`. Do not introduce global mutable services.
- **UI code clones before drawing.** In `DetailView::show`, every borrowed value is cloned, the `&Song` is dropped (`let _ = song;`), and only then are `&mut AppState` closures created. Follow this pattern; otherwise the borrow checker will force awkward workarounds.
- **Mutate after rendering.** `GridView` collects `clicked_idx: Option<usize>` and calls `open_song_detail` after the table/grid iteration. Do the same for any new list interaction.
- **Split borrows, not clones.** The grid uses `let songs = &state.songs; let textures = &mut state.album_art_textures;` to keep tiles allocation-free.
- **Errors become `String` at thread boundaries.** Background code maps `AppError` with `.map_err(|e| e.to_string())` and sends `Result<_, String>`.
- **Guard delete with a flag.** Tag cleanup arrays and clippy allows are documented in comments; keep those comments when editing.
- **No logging framework.** User-visible errors go to `error_message`; there is no `log`/`tracing`.
- **Formatting:** `cargo fmt` defaults (4 spaces, 100 columns). Clippy warnings are errors in CI except the two documented allows.

## 2. Naming traps

| Name | Trap |
|---|---|
| `View::Grid` | Means **library**, not grid layout. The library can be in list or grid mode via `LibraryViewMode`. |
| `TextView`/`GridView` | `GridView` is the whole library view (toolbar + list + tiles), not just the tile grid. |
| `LibraryViewMode::List` | Default, even though `View::Grid` is the initial view. |
| `window_size` | Present in `AppSettings` but unused; eframe persistence handles window geometry. |
| `last_opened_folder` | Present but never written or read. |
| `error_message` | Also holds success text ("Connection test successful!"). |
| `album_art_mime` | A `String` like `image/jpeg`, not an enum; unknown values fall back to JPEG on write. |
| `load_songs` | Synchronous helper, `#[allow(dead_code)]`, unused by the UI. |

## 3. README drift

The root `README.md` is user-facing and stale in several places. `context/` is authoritative.

| README claim | Reality |
|---|---|
| "Keyboard Shortcuts: Double-click opens detail" | Single click opens detail; no double-click handler. |
| "Enter (in detail) saves" | No key handler exists. |
| "Escape goes back / closes dialog" | No key handler exists. |
| "34+ metadata fields" (also in the RPM spec) | 18 text fields are exposed, plus artwork/MIME. |
| Windows installer "adding the app to `PATH`" | The `.iss` has no PATH task or registry entry. |
| "Rust 1.75+" | CI/release pin 1.98.1; the lockfile may not build on 1.75. |
| "Back to Grid" | The button says "Back to Library". |
| "Batch Operations" | Multi-file add only; there is no batch edit/save. |
| `src/ui/detail_view.rs#Song metadata editor` | Missing space in the README tree, cosmetic. |
| Dependencies list omits `egui_extras` and `winit`. | Both are direct dependencies. |

## 4. Metadata behaviour that surprises

1. **Saving is a replace, not a merge.** Empty UI fields call `remove_key`, so opening a file, clearing a field, and saving deletes that tag.
2. **Fixed keys are always stripped.** MusicBrainz IDs, AcoustID/fingerprint, ReplayGain (4 keys), Conductor, Arranger, Engineer, MixEngineer are removed on every save, even if untouched. See [05-metadata-pipeline.md](05-metadata-pipeline.md).
3. **Year is written twice** (`RecordingDate` and `Year`). On read, whichever `ItemKey` appears later wins.
4. **Artwork replacement is all-or-nothing.** Any difference in `AlbumArt` (including removal) deletes **every** picture in the tag, then pushes one `CoverFront`. Other picture types (back cover, artist) are lost.
5. **Only the first picture is read.** Additional embedded pictures are ignored.
6. **No write validation.** A corrupt artwork file can be embedded (if the tag format accepts the bytes); the preview will fail to decode.
7. **No new tag creation.** If a file has no tag, save fails with "No writable tag found" instead of creating one.
8. **Writes are in place, not atomic.** A crash or full disk during `save_to_path` can truncate the file; there is no temp-file rename or backup.
9. **Saves block the UI.** `write_metadata` runs inside the egui frame.
10. **Album art is not recompressed/resized**, so a 3000×3000 PNG is embedded as-is.

## 5. State and UI gotchas

- **`Clear All` does not reset async state.** It clears `songs` but leaves `loading_count`, `is_fetching`, `fetch_index`, `selected_index`, and both texture caches. A pending fetch completes into an out-of-range index and is ignored, but the busy flags stay until its message arrives.
- **One global fetch at a time.** `is_fetching` is not per song; Fill Data is disabled for every song while any fetch runs.
- **`fetch_index` is captured at spawn.** If the library changes during a fetch (Clear All, additional adds), the index may refer to another song. There is no generation check.
- **Detail index can go stale.** `DetailView::show` resets to `View::Grid` when `idx >= songs.len()`. Nothing else guards it.
- **Unsaved changes are snapshot-based.** Only one `original_metadata` snapshot exists; there is no undo stack.
- **Quit does not prompt.** Unsaved edits are discarded; only settings are saved in `on_exit`.
- **Theme changes save immediately** (menu selection), while LLM settings save only on the dialog's Save button. Cancel discards the draft.
- **Test connection mutates the draft's `enabled` temporarily.** It restores the previous value, but the cloned settings sent to the task have `enabled = true`.
- **Inline settings errors are string-filtered.** Only messages containing `Connection`, `connection`, or `LLM` appear inside the AI Settings popup; everything else appears in the Error window behind/above it.
- **Error window is single-slot.** A new error replaces the old one; there is no history or queue.
- **No keyboard input handling.** Not even Escape closes dialogs; use the window close button or the dialog's buttons.

## 6. Field-index coupling

`field_names()`, `get_field`, `set_field`, the detail-view loop, and the hard-coded lyrics index `15` must stay in sync.

- The detail view filters `i != 15` and then calls `update_metadata_field(idx, 15, lyrics)` explicitly.
- Adding, removing, or reordering a field requires updating all five places **and** the LLM prompt.
- `get_field`/`set_field` return `""`/no-op for out-of-range indices, so a mismatch silently loses edits rather than panicking.

## 7. LLM and artwork gotchas

- **`parse_response` slices first `{` to last `}`.** Markdown fences are fine; prose with braces, multiple objects, or truncated output is not.
- **`max_tokens = 2000`** can truncate long lyrics, producing invalid JSON.
- **`is_empty()` includes artwork.** A response with only artwork would not be "empty", but the LLM never returns artwork.
- **Local endpoints are prefix-matched.** `http://localhost...`, `http://127.0.0.1...`, and their `https` forms bypass the API-key requirement. A hostname like `http://localhost.evil.example` would also pass the check (and then fail to connect).
- **Default endpoint is OpenAI.** With `base_url = None` and an API key, requests go to `https://api.openai.com/v1`. Without `/v1` in a custom URL, requests 404.
- **Artwork always overwrites.** A successful iTunes lookup replaces existing art on Fill Data, even if the user loaded art manually.
- **iTunes resolution rewrite is literal.** `.replace("100x100bb", "600x600bb")`; if Apple changes the pattern, low-res art is embedded.
- **First iTunes result wins.** No matching by album/artist beyond the query term.
- **Artwork failures are silent.** They surface neither as errors nor as status messages.
- **No cancellation or retry.** Navigating away or closing the dialog does not stop a request; transient failures are final.
- **Task panics can wedge the UI.** A panic in the spawned task drops without sending, leaving `is_fetching = true` and Fill Data disabled until restart.

## 8. Settings and filesystem gotchas

- `SettingsService::new()` creates the config dir and is called in `AppState::new`. `XurPorisoyApp::new` uses `.expect("Failed to initialize app state")`, so an unreadable/malformed `settings.json` **panics at startup**.
- `load()` returns defaults only when the file is **absent**. There is no merge for partial/older JSON; missing non-`theme` fields fail deserialization.
- `save()` uses `fs::write` (truncate + write), not atomic rename.
- The API key is plaintext.
- `SettingsService::default()` also panics on failure.
- Config directory name is `xur-porisoy` for all platforms.
- `assets/fonts/NotoSans-Regular.ttf` is committed but never referenced (eframe `default_fonts` is used).
- `assets/icon.png` is embedded for the window icon; `assets/icon.ico` is used by `build.rs` and Inno Setup.

## 9. Texture cache gotchas

- Keys are `art.hash() ^ (u64::from(max_side) << 32)`; the same art at 96 and 320 px gets different entries.
- Library cap is 512, detail cap is 8.
- Eviction is insertion-order FIFO (`VecDeque`), not true LRU; hitting an existing entry does not move it to the back.
- `failed: HashSet<u64>` is never evicted, so a pathological library of unique corrupt covers grows it without bound (small: 8 bytes per hash).
- Decoding happens synchronously on the UI thread; fast scrolling a large library can stutter.
- `decode_thumbnail` limits: 4096×4096 and 256 MiB allocation. Truncated or non-image bytes return `None`.

## 10. Concurrency gotchas

- `library-loader` is spawned per Add Songs call; two batches interleave messages.
- `loading_count` is decremented per message, including errors, but a thread panic leaves it stuck above zero (permanent 100 ms repaint).
- The UI only requests repaints while `is_fetching || loading_count > 0`. A message sent after both flags clear waits for user input.
- `set_tokio_handle` silently ignores a second set.
- There is no file locking: loading a file while saving an earlier copy of it can race.
- Saves happen on the UI thread, so they cannot race with UI edits but do freeze the window.

## 11. CI and packaging gotchas

- `ci.yml` and `release.yml` duplicate the fmt/clippy/test block; keep them identical.
- Clippy allows `clippy::never_loop` and `clippy::large_enum_variant`; removing them may surface pre-existing findings.
- Path filters exclude `context/**`, `README.md`, and `.cargo/**`, so docs-only commits run no CI.
- Tag pushes ignore path filters.
- The release job asserts an exact list of 22 artifact globs; renaming outputs breaks it.
- The apt repository publishes **amd64 only**.
- The apt repo is unsigned unless `APT_REPO_KEY_ID` + `APT_REPO_PRIVATE_KEY` are configured; unsigned clients need `Trusted: yes`.
- `REPO_BASE` is hard-coded in `build-repo.py` and `install.sh`; forks must change both.
- **No `LICENSE` file is committed**, although `Cargo.toml`, `build.rs`, the Debian copyright template, and the RPM spec all say MIT. Debian/tarball/installer installs are conditional. Adding a root `LICENSE` would automatically flow into all three.
- macOS artifacts are unsigned and un-notarized; Windows installers are unsigned.
- RPM packages have no `Requires`; Flatpak grants `--filesystem=home`.
- AppImage builds download `appimagetool` from GitHub's continuous release at build time.
- The Windows job verifies `ProductName`/`FileVersion`; changing `build.rs` strings or the numeric version derivation breaks it.
- `metadata` reads the **first** `version =` line in `Cargo.toml`. Adding another `version =` line above `[package]` would break it.
- Rust is pinned to 1.98.1 in both workflows and every packaging job. Bump deliberately.

## 12. Platform traps

| Platform | Trap |
|---|---|
| Linux | GUI libraries are `dlopen`ed; `ldd` does not list them. A binary can build and still fail at startup without `libx11`, `libxcb`, `libxkbcommon`, `libwayland`, `libEGL`, `libGL`, `libdbus`. |
| Linux | Ubuntu 24.04 restricts unprivileged user namespaces; Flatpak builds need the `apparmor_restrict_unprivileged_userns=0` sysctl. |
| Linux | `libm6` must not be listed in Debian Depends (glibc ≥ 2.34 provides it via `libc6`). |
| Windows | `#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]` hides the console only in release. |
| Windows | ARM64 builds use `aarch64-pc-windows-msvc`; the installer has an `arm64` architecture guard. |
| macOS | Minimum target 11.0; universal binary must be produced with `lipo`. No code signing. |
| Wayland | The direct `winit = "0.30"` dependency is required for dynamic loading and client-side decorations; removing it causes undecorated or non-starting windows. |
| All | `panic = "abort"` in release means no unwinding; a panic terminates the process without a dialog. |

## 13. Documentation-pack rules

- Prefer **function/module names** over line numbers; line numbers drift.
- When behaviour changes, update the relevant `context/` file in the same PR.
- If `Cargo.toml`, `Cargo.lock`, `installer/**`, `packaging/**`, or the workflows change, re-check [11-ci-cd.md](11-ci-cd.md) and [12-packaging-and-release.md](12-packaging-and-release.md).
- If a field is added/removed, update [04-data-model.md](04-data-model.md), [05-metadata-pipeline.md](05-metadata-pipeline.md), and [08-ai-and-llm.md](08-ai-and-llm.md).
- State the verified commit/version in [README.md](README.md) when you refresh the pack.
