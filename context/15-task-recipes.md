# 15 — Task Recipes

Concrete change guides. Each recipe lists the files in dependency order and the checks to run. Commands are run from the repository root.

---

## 1. Add a metadata field

**Goal:** expose a new tag end to end (read → edit → LLM → write).

| Step | File | Change |
|---|---|---|
| 1 | `src/models/song.rs` | Add `pub my_field: String` to `SongMetadata`. `#[serde(default)]` already covers LLM responses. |
| 2 | `src/models/song.rs` | Add the label to `field_names()` **and** update `get_field`/`set_field` indices. Append at the end (index 18) to avoid disturbing the lyrics index `15`. |
| 3 | `src/models/song.rs` | Add it to `is_empty()` if it should count toward "no metadata". |
| 4 | `src/services/metadata.rs` | In `extract_tag_data`, map the appropriate `ItemKey` to the field. |
| 5 | `src/services/metadata.rs` | In `write_tag_data`, add a `set_if_not_empty!(tag, ItemKey::..., metadata.my_field);`. |
| 6 | `src/services/llm.rs` | Add the key to `build_prompt`'s JSON template. |
| 7 | `context/04-data-model.md`, `05-metadata-pipeline.md`, `08-ai-and-llm.md` | Update the field tables. |
| 8 | Verify | Run the commands in §18, then round-trip a real file: read → edit → save → reopen. |

**Do not** insert a field in the middle unless you also update:
- `get_field`/`set_field`,
- `field_names`,
- the hard-coded lyrics index `15` in `detail_view.rs`,
- the `i != 15` filter.

The compiler will not catch index drift; edits silently go to the wrong field.

If the new key is one of the always-stripped keys (MusicBrainz, ReplayGain, etc.), remove it from the cleanup arrays too.

---

## 2. Change the tag cleanup policy

**Goal:** preserve MusicBrainz/ReplayGain/etc., or strip a different set.

File: `src/services/metadata.rs::write_tag_data`.

1. Edit the two arrays after the field writes:
   - MusicBrainz IDs,
   - Conductor/Arranger/Engineer/MixEngineer/AcoustID/ReplayGain.
2. Remember this runs on **every save**, so removing a key from the list means the app will now preserve whatever was on disk (it does not write it).
3. Update [05-metadata-pipeline.md](05-metadata-pipeline.md) §5.3 and [14-conventions-and-gotchas.md](14-conventions-and-gotchas.md) §4.
4. Test with a file that contains the keys; save without editing and confirm they survive (or disappear).

**Related:** to stop empty fields from deleting keys, change the `set_if_not_empty!` macro semantics. That macro is called for every field, so an "only write when non-empty, otherwise leave existing" policy requires a different helper (e.g. read the current key first). State the new policy in the docs.

---

## 3. Change the LLM prompt or requested fields

File: `src/services/llm.rs::build_prompt`.

1. Edit the JSON template and instruction text. Keep it a **single object** with string values.
2. If you add a field, also complete recipe 1.
3. Consider `max_tokens` (2000) if the new field can be long.
4. Update [08-ai-and-llm.md](08-ai-and-llm.md) §5.
5. Test against a local OpenAI-compatible server and a hosted model; verify the response parses with and without Markdown fences.

**Do not** add artwork to the prompt; artwork is fetched from iTunes separately.

---

## 4. Add or change a model preset

File: `src/ui/settings_view.rs`.

1. Add the name to the array `["gpt-4o-mini", "gpt-4o", "Custom"]`.
2. No other change is required; `model_name` is free text and is sent verbatim.
3. Keep "Custom" last and preserve the "selecting Custom clears only when empty" logic.
4. Update [08-ai-and-llm.md](08-ai-and-llm.md) if the default changes.

To change the default base URL, edit the `get_or_insert_with` defaults for API and local modes. The request fallback lives in `LlmService::call_llm` (`https://api.openai.com/v1`) and must match.

---

## 5. Change the set of accepted file types

The extension list is duplicated in two places:

| Location | Code |
|---|---|
| File menu | `ui/mod.rs` → `rfd::FileDialog::new().add_filter("Audio Files", &[...])` |
| Library toolbar | `ui/grid_view.rs` → the same `add_filter` |

1. Update both arrays.
2. If the new format needs a different reader/writer path, verify `lofty` support and the `ItemKey` mapping in `services/metadata.rs`.
3. If you add an image format, update **three** lists:
   - `Cargo.toml` `image` features,
   - `services/artwork.rs` `guess_format` match,
   - `services/metadata.rs` MIME match plus `ui/detail_view.rs` extension→MIME match.
4. Update [01-project-overview.md](01-project-overview.md), [05-metadata-pipeline.md](05-metadata-pipeline.md), and the README.

Test with a real file of the new type, including a tag-less file (read should succeed; save will fail with "No writable tag found" unless a tag is created).

---

## 6. Change artwork sizes or cache limits

| Knob | Location | Current |
|---|---|---|
| Library decode size | `ui/grid_view.rs` → `get_or_load(..., 96)` | 96 px |
| Library display size | `ui/grid_view.rs` → `add_sized([72, 72])` | 72 px |
| Detail decode size | `ui/state.rs::detail_album_art` → `get_or_load(..., 320)` | 320 px |
| Detail display size | `ui/detail_view.rs` → `fit_to_exact_size(300, 300)` | 300 px |
| Library cache cap | `ui/state.rs::AppState::new` → `TextureCache::new(512)` | 512 entries |
| Detail cache cap | `ui/state.rs::AppState::new` → `TextureCache::new(8)` | 8 entries |
| Decode limits | `services/images.rs` | 4096×4096, 256 MiB |

1. Keep the decode size ≥ display size to avoid upscaling.
2. The cache key includes `max_side`, so changing it invalidates old entries automatically within a session.
3. Larger caps increase GPU memory; each entry is a decoded RGBA texture.
4. Update [06-ui-and-interactions.md](06-ui-and-interactions.md) and [09-worker-and-concurrency.md](09-worker-and-concurrency.md) if limits change.

---

## 7. Add a menu item or a dialog

### Menu item

1. Add `ui.menu_button(...)` or `ui.button(...)` in `XurPorisoyApp::update`'s menu bar (`src/ui/mod.rs`).
2. Store dialog visibility as a new field on `XurPorisoyApp` (follow `show_about`).
3. Render the dialog after the main content:
   ```rust
   if self.show_my_dialog { MyDialog::show(ctx, &mut self.state, &mut self.show_my_dialog); }
   ```
4. Use `egui::Window::new(...).collapsible(false).anchor(Align2::CENTER_CENTER, Vec2::ZERO)` for modal-like behavior. egui has no true modality; consider disabling background controls if needed.

### New top-level view

1. Add a variant to `View` in `ui/state.rs`.
2. Dispatch it in `ui/mod.rs`'s `match self.state.current_view`.
3. Create `ui/my_view.rs` with an `impl MyView { pub fn show(ctx, state) }`, and add `pub mod my_view;` in `ui/mod.rs`.
4. Update any `matches!`/exhaustive matches (e.g. `can_go_back`, the menu label).

Update [06-ui-and-interactions.md](06-ui-and-interactions.md).

---

## 8. Add keyboard shortcuts (the README already claims them)

There is no key handling today. To implement:

1. In `XurPorisoyApp::update`, after draining messages and before drawing:
   ```rust
   ctx.input(|i| {
       if i.key_pressed(egui::Key::Escape) { ... }
   });
   ```
   Or use `ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))` so text fields do not also receive the key.
2. Decide precedence: text editors consume Enter/Escape when focused. Check `ctx.memory(|m| m.focused())` or use `consume_key`.
3. For double-click in the list, replace `Sense::click()` with `Sense::click()` plus `response.double_clicked()` (egui reports both single and double clicks; gate single-click navigation accordingly).
4. For Enter-to-save, call `state.save_current_song()` and handle the error like the Save button.
5. Update the README table and [06-ui-and-interactions.md](06-ui-and-interactions.md) §9.

---

## 9. Make saving asynchronous

**Goal:** stop blocking the UI thread in `MetadataService::write_metadata`.

1. Add `AppMessage::SongSaved(usize, Result<(), String>)` in `ui/state.rs`.
2. Add `is_saving: bool` (and optionally `saving_index`) to `AppState`.
3. In `save_song_at`, clone the needed data. `Song` is `Clone`, but writing must use the **edited** metadata; the simplest approach is to take the `Song` out (`std::mem::replace` with a placeholder) or clone it and write the clone, then adopt the snapshot on success.
4. Spawn a `std::thread` (tag I/O, not HTTP) with the sender; call `MetadataService::write_metadata` there.
5. On `SongSaved`: clear `is_saving`, adopt `original_metadata`/`has_changes` on success, show the error on failure, and navigate back only on success.
6. Disable Save while saving; include `is_saving` in the repaint condition (`update`).
7. Add cancellation/generation if the user can navigate away and start another save.
8. Update [09-worker-and-concurrency.md](09-worker-and-concurrency.md) and [05-metadata-pipeline.md](05-metadata-pipeline.md).

Beware: `write_metadata` mutates `Song` (`save_changes`). If you move it to a thread, send back the updated snapshot or re-read the file.

---

## 10. Add a persisted setting

1. Add the field to `AppSettings` in `models/song.rs`. Consider `#[serde(default)]` for backward compatibility; the rest of the struct has none.
2. Add UI to change it (menu, settings dialog, etc.).
3. Call `state.save_settings()` when it changes (or rely on `on_exit`, but explicit save matches the theme behavior).
4. Update [04-data-model.md](04-data-model.md) and [10-build-run-and-config.md](10-build-run-and-config.md).
5. Test with an old `settings.json` (missing the new key) to confirm serde behavior.

If the setting affects background work, thread it through the values cloned into the task; background code must not read `AppState`.

---

## 11. Add a new library view mode

1. Add a variant to `LibraryViewMode` (`ui/state.rs`).
2. Add a `selectable_label` in the `GridView` toolbar.
3. Add a `match` arm in `GridView::show` calling a new `show_<mode>(ui, state, &mut clicked_idx)`.
4. Follow the borrow-splitting and deferred-click patterns. Use `show_rows`/`TableBuilder` for virtualization.
5. `LibraryViewMode` is not persisted; no serde change needed unless you want persistence.

Update [06-ui-and-interactions.md](06-ui-and-interactions.md).

---

## 12. Add a test

Unit tests live in-module:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn my_test() { ... }
}
```

- Pure logic (serde, field indexing, prompt building, URL rewriting) is easy to test.
- `MetadataService::read_metadata`/`write_metadata` need real audio fixtures; none exist. Add small fixtures under `tests/fixtures/` and use `tempfile`-style copies (a dev-dependency would need adding) to avoid mutating committed files.
- The `LlmService` HTTP path is not abstracted behind a trait, so mocking requires either a local server or a refactor.
- Run `cargo test --workspace --locked`.
- Update the test count in [README.md](README.md) / [01-project-overview.md](01-project-overview.md) if it changes materially.

---

## 13. Bump the version and release

1. Edit `version` in `Cargo.toml`.
2. Run `cargo check` (or `cargo build`) so `Cargo.lock` records the new root version.
3. Update any documentation that names the current version (`context/README.md`).
4. Commit both files:
   ```bash
   git add Cargo.toml Cargo.lock context/
   git commit -m "chore(release): bump version to X.Y.Z"
   ```
5. Tag and push:
   ```bash
   git tag vX.Y.Z
   git push origin main
   git push origin vX.Y.Z
   ```
6. Watch the `Release` workflow. The tag run ignores path filters and runs every job. The `release` job attaches all artifacts plus `SHA256SUMS`; `publish-pages` republishes the apt repo.
7. Verify the GitHub Release and `https://borneelphukan.github.io/xur-porisoy/`.

Do not tag before CI passes; the tag run has its own quality gate, but a failure means no release.

---

## 14. Add a packaging target or artifact

1. Write a script under `packaging/<family>/` that:
   - requires `VERSION` and an architecture variable,
   - accepts `BINARY` and `OUT_DIR`,
   - never compiles Rust,
   - prints the output path.
2. Add jobs/steps to `.github/workflows/release.yml`, upload an artifact with a stable name.
3. Add the filename glob to the `expected` array in the `release` job.
4. If it is a Debian package, ensure `publish-pages` downloads it if it should be in the apt repo; the script derives architectures from the `.deb` files.
5. Add a row to [12-packaging-and-release.md](12-packaging-and-release.md) and the README packaging section.
6. Test locally with a release binary.

The `release` job fails if a listed pattern is missing; extra files are fine.

---

## 15. Change the apt repository URL or signing

**URL:**

1. `packaging/apt/build-repo.py` → `REPO_BASE`.
2. `packaging/apt/install.sh` → `BASE_URL` default.
3. README install instructions and the sources snippet.
4. [13-website.md](13-website.md) canonical URLs.

**Signing:**

1. Add the private key as `APT_REPO_PRIVATE_KEY`, the passphrase as `APT_REPO_PASSPHRASE`, and the key id as the repository **variable** `APT_REPO_KEY_ID`.
2. The workflow passes `--gpg-key-id/--gpg-key-file/--gpg-passphrase-file`; `build-repo.py` writes `InRelease`, `Release.gpg`, and the public keyring.
3. Verify a fresh container can `apt update` with `Signed-By`.
4. Without the material the repo is unsigned and clients need `Trusted: yes`; document that fallback.

---

## 16. Debug "stuck loading" or "Fetch disabled"

| Symptom | Check |
|---|---|
| Toolbar says "loading N…" forever | A `library-loader` thread panicked or a path never sent. `loading_count` only decrements on `SongLoaded`. Restart clears it. |
| Fill Data stays disabled and says "Fetching…" | A spawned task panicked without sending `MetadataFetched`, or the Tokio handle was never set. Restart. |
| Fill Data disabled with no hint | `settings.llm.is_configured()` is false: check `enabled`, `model_name`, and key/localhost base URL. |
| Fetch fails immediately | Check endpoint (`/v1` suffix), key, model name, and the response body in the error window. |
| Settings dialog shows no error | The inline filter only matches `Connection`/`connection`/`LLM`; dismiss the Error window behind it. |
| Artwork missing after Fill Data | iTunes lookup failed silently; check network and the title/artist/album term. |
| Save fails "No writable tag found" | The file has no tag; the app will not create one. |

---

## 17. Refresh this context pack

1. Read `context/README.md` and note the verified commit/version.
2. Re-read the files affected by the change (workflows, packaging, `Cargo.toml`).
3. Update the relevant numbered file **in the same PR**.
4. Check every relative link resolves (all files are siblings, so links are plain `NN-....md`).
5. Update the commit hash and version in `README.md`.
6. Do not add line numbers; use function/module names.

---

## 18. Verify a change locally

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- \
  -D warnings -A clippy::never_loop -A clippy::large_enum_variant
cargo test --workspace --locked
cargo build --release --locked

# packaging smoke test
packaging/debian/build-deb.sh
dpkg-deb --contents dist/*.deb | head

# Pages smoke test
python3 packaging/apt/build-repo.py --debs dist --out /tmp/xur-pages \
  --suite stable --component main --origin xur-porisoy --label "XurPorisoy"
python3 -m http.server 8000 --directory /tmp/xur-pages
```

Then exercise the UI: add two files, switch list/grid, edit and save, clear a field and save, load/remove artwork, configure a local LLM, Fill Data, and discard changes.
