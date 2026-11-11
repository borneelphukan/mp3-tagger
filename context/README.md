# xur-porisoy — AI Agent Context Pack

This directory is the **authoritative, current context** for anyone (human or AI) working on `xur-porisoy`. It describes the application as it exists at:

| Field | Value |
|---|---|
| Repository | `github.com/borneelphukan/xur-porisoy` |
| Branch verified | `main` |
| Commit verified | `4e0fcb1` (`chore: rename project to xur-porisoy`) |
| Version | `0.0.1` (`Cargo.toml`) |
| Rust edition | `2021` |
| UI stack | `eframe` / `egui` `0.29` (glow renderer) |
| Audio metadata | `lofty` `0.25` |
| HTTP client | `reqwest` `0.12` (rustls, no OpenSSL) |
| Async runtime | `tokio` `1.40`, 2 worker threads |
| Platforms | Windows, Linux (Debian/Ubuntu primary), macOS universal |
| License | MIT (declared in `Cargo.toml`; **no `LICENSE` file is committed**) |
| Website | none; GitHub Pages hosts the apt repository and `install.sh` |

> There is no `AGENTS.md` in this repository. Treat this `context/` directory as the source of truth. The root `README.md` is user-facing and is sometimes stale (see [14-conventions-and-gotchas.md](14-conventions-and-gotchas.md)).

---

## What to read first

| Question | Read |
|---|---|
| What does the app do and what are its boundaries? | [01-project-overview.md](01-project-overview.md) |
| Which file owns which behaviour? | [02-project-structure.md](02-project-structure.md) |
| How do the process, threads, UI, and state fit together? | [03-architecture.md](03-architecture.md) |
| What are the Rust types and message contracts? | [04-data-model.md](04-data-model.md) |
| How are tags read, merged, written, and artwork handled? | [05-metadata-pipeline.md](05-metadata-pipeline.md) |
| How is the egui UI assembled and how do interactions work? | [06-ui-and-interactions.md](06-ui-and-interactions.md) |
| How does each user-facing feature work end to end? | [07-features-and-flows.md](07-features-and-flows.md) |
| How does LLM metadata lookup and artwork fetching work? | [08-ai-and-llm.md](08-ai-and-llm.md) |
| How does background work and messaging work? | [09-worker-and-concurrency.md](09-worker-and-concurrency.md) |
| How do I build, run, and configure it locally? | [10-build-run-and-config.md](10-build-run-and-config.md) |
| How do CI, audits, and releases work? | [11-ci-cd.md](11-ci-cd.md) |
| How are Linux/Windows/macOS packages produced? | [12-packaging-and-release.md](12-packaging-and-release.md) |
| What is on GitHub Pages and how is it built? | [13-website.md](13-website.md) |
| What patterns, bugs, and platform traps should I avoid? | [14-conventions-and-gotchas.md](14-conventions-and-gotchas.md) |
| I need a concrete step-by-step change recipe | [15-task-recipes.md](15-task-recipes.md) |

---

## Critical facts at a glance

1. **One native process, one UI thread, one library loader, one tokio runtime.** `XurPorisoyApp` owns all UI state. Tag parsing runs on a `std::thread` named `library-loader`. LLM calls and iTunes artwork lookups are spawned onto a 2-worker tokio runtime whose `Handle` is cached in a `OnceLock`.
2. **The UI is immediate mode.** Every frame rebuilds the UI from `impl XurPorisoyApp` methods plus `GridView`, `DetailView`, and `SettingsView`. There is no retained widget tree. Deferred mutations are done by cloning indices and applying them after the UI closure.
3. **All state is in memory plus one JSON file.** `AppState` holds `Vec<Song>`, the current view, the texture caches, and settings. Settings live at `ProjectDirs::from("", "", "xur-porisoy")/settings.json`; the LLM API key is stored in plaintext.
4. **Tag writing is not a merge.** `MetadataService::write_metadata` rewrites the primary tag: empty UI fields delete their key, artwork changes delete all existing pictures, and a fixed list of MusicBrainz/ReplayGain/AcoustID/credits keys is always removed.
5. **Artwork is reference counted and capped.** `AlbumArt` stores `Arc<[u8]>` and a hash computed once. Two `TextureCache`s cap decoded textures at 512 library tiles and 8 detail previews; failed decodes are remembered so corrupt covers are not retried every frame.
6. **LLM support is "any OpenAI-compatible `/chat/completions`".** The base URL defaults to `https://api.openai.com/v1`. Local endpoints on `localhost`/`127.0.0.1` are allowed without an API key; all other endpoints require one.
7. **Cover art comes from the public iTunes Search API.** The 100×100 URL is rewritten to 600×600, the MIME type comes from `Content-Type`, and only the image header is validated before the bounded decoder runs.
8. **Saving is synchronous on the UI thread.** `save_current_song` calls `MetadataService::write_metadata` inline, so writing a large tag/artwork set blocks the frame.
9. **Releases are tag-driven.** The release workflow reads `0.0.1`-style versions from the first `version =` line in `Cargo.toml`. A `v*` tag attaches the verified bundle to a GitHub Release; ordinary runs publish the same bundle as a workflow artifact and republish the apt repository on Pages.
10. **Packaging is multi-architecture and script-driven.** The matrix covers Debian, RPM, tarball, AppImage, Flatpak, Arch, Windows, and a macOS universal DMG. All scripts read a prebuilt binary; none compile inside the packaging step.

---

## Repository entry points

- Application entry: `src/main.rs` → `ui::XurPorisoyApp::new` → `eframe::App::update` in `src/ui/mod.rs`.
- UI state and message handling: `src/ui/state.rs`.
- Tag reading/writing: `src/services/metadata.rs`.
- LLM client: `src/services/llm.rs`; artwork lookup: `src/services/artwork.rs`.
- Settings persistence: `src/services/settings.rs`.
- Release automation: `.github/workflows/{ci,audit,release}.yml` and `packaging/**`.

## Keeping this pack accurate

When behaviour changes, update the relevant file here **in the same PR**. Prefer function/module names and contracts over large code blocks; line numbers drift quickly. If `Cargo.toml`, `Cargo.lock`, `installer/xur-porisoy.iss`, `packaging/**`, or the workflows change, re-check [11-ci-cd.md](11-ci-cd.md) and [12-packaging-and-release.md](12-packaging-and-release.md).
