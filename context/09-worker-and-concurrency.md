# 09 — Worker and Concurrency Model

The app uses three kinds of execution: the UI thread, a short-lived `std::thread` per Add Songs batch, and a process-wide Tokio runtime for HTTP. Understanding which work runs where explains most UI freezes and stale-state bugs.

## 1. Thread inventory

| Execution context | Created by | Lifetime | Runs |
|---|---|---|---|
| Main / UI thread | `eframe::run_native` | Process | egui, all state mutation, texture creation, tag writes, settings I/O |
| `library-loader` | `AppState::add_songs` | Until the batch ends or the receiver drops | `MetadataService::load_song` per path |
| Tokio worker 1–2 | `#[tokio::main(worker_threads = 2)]` | Process | LLM fetch, iTunes fetch, connection test |
| `reqwest` runtime tasks | `reqwest` | Per request | DNS, TLS, I/O |

There is **no** dedicated save thread, no rayon pool, no database, and no watcher.

## 2. Tokio runtime and handle

`src/main.rs`:

```rust
#[tokio::main(worker_threads = 2)]
async fn main() -> Result<(), eframe::Error> {
    ...
    crate::services::llm::set_tokio_handle(tokio::runtime::Handle::current());
    eframe::run_native(...)
}
```

`services/llm.rs`:

```rust
static TOKIO_HANDLE: OnceLock<Handle> = OnceLock::new();
pub fn set_tokio_handle(handle: Handle) { let _ = TOKIO_HANDLE.set(handle); }
pub fn get_tokio_handle() -> Option<&'static Handle> { TOKIO_HANDLE.get() }
```

- The `OnceLock` is set once; later calls are ignored.
- UI code spawns with `handle.spawn(async move { ... })` and ignores the returned `JoinHandle`.
- Two workers are chosen to avoid the default "one per CPU" stack/allocator footprint. More concurrency would not help: the app rarely has more than two HTTP requests in flight, and only one fill is allowed anyway.
- `eframe` runs a blocking native event loop after `main` returns to `run_native`; the Tokio runtime remains alive because the handle/runtime is owned by the async `main` frame for the process duration.

## 3. Library loading

```rust
let sender = self.message_tx.clone();
std::thread::Builder::new()
    .name("library-loader".to_owned())
    .spawn(move || {
        for path in paths {
            let song = MetadataService::load_song(&path).map_err(|e| e.to_string());
            if sender.send(AppMessage::SongLoaded(song)).is_err() {
                break;
            }
        }
    });
```

- A new thread is created for **each** `add_songs` call. Two quick batches run two threads concurrently and interleave `SongLoaded` messages.
- Ordering across batches is therefore not guaranteed, though within one batch it is input order.
- `loading_count` is incremented by the batch size before spawn and decremented per received message. If the thread fails to spawn, the count is rolled back and an error is shown.
- If the receiver is dropped (window closed), the thread stops at the next send.
- A panic inside `load_song` would kill that thread; remaining paths in the batch would never send, leaving `loading_count > 0` forever. `lofty` is not expected to panic, but there is no catch-unwind.

## 4. Async fetches

### Fill Data

Spawned from `DetailView` with cloned values:

```rust
handle.spawn(async move {
    let result = llm_service.fetch_metadata(&settings, &title, &artist).await
        .map_err(|e| e.to_string());
    let result = match result {
        Ok(mut metadata) => { /* best-effort iTunes artwork */ Ok(metadata) }
        Err(error) => Err(error),
    };
    let _ = sender.send(AppMessage::MetadataFetched(idx, result));
});
```

- The `LlmService` clone shares the same `reqwest::Client` connection pool.
- The `idx` is captured at spawn; the result is applied to that index later. There is no generation counter.
- A panic inside the task (not expected) would drop without sending; `is_fetching` would stay `true` and the Fill button would remain disabled. There is no timeout guard at the app level, though `reqwest` has a 60 s total timeout.

### Test connection

Spawned from `SettingsView` with a cloned settings value, sends `AppMessage::LlmTestResult(Result<(), String>)`. It shares the same runtime and has no cancellation.

## 5. Message channel

`std::sync::mpsc::channel::<AppMessage>()`, created in `XurPorisoyApp::new`. Sender clones live in `AppState` (and are cloned into threads/tasks); the single receiver is owned by `XurPorisoyApp`.

Delivery model:

- Producers **push and forget**.
- The UI **pulls** with `try_recv()` in a loop at the start of every frame.
- There is no wake-up callback: `egui` is not notified when a message arrives. Instead, while `is_fetching || loading_count > 0`, the frame calls `ctx.request_repaint_after(Duration::from_millis(100))`. This keeps frames coming until the work finishes.
- Consequence: if a producer sends after the last repaint request (e.g. a late message after `is_fetching` was cleared by another path), the message may sit until the next user interaction. In normal flows the flags stay set until the corresponding message is drained, so this does not happen.

Draining all messages per frame matters for batch loads: dozens of `SongLoaded` messages can accumulate between frames, and processing only one per frame would make a large batch appear one song per 100 ms.

`AppMessage` payloads are `Result<_, String>`; errors are stringified at the producer boundary. There is no structured error type across threads.

## 6. UI-thread-only work

These intentionally (or accidentally) run inside the frame:

| Work | Why it matters |
|---|---|
| `ctx.load_texture` | egui textures must be created on the UI thread. Decoding itself also happens here. |
| `std::fs::read` for Load Artwork | A large image blocks the frame. |
| `MetadataService::write_metadata` on Save | File I/O plus tag encoding blocks the frame; large artwork makes it noticeable. |
| `SettingsService::save` | Small JSON write; called on theme change, dialog Save, and exit. |
| `rfd` file dialogs | Native modal dialogs; egui waits. |
| `image::guess_format` / `decode_thumbnail` | Bounded, but still CPU work on the UI thread; 512 cached textures can mean repeated decoding during fast scrolling. |

## 7. Shared state and synchronization

- `AppState` is **not** shared: background threads receive only cloned values (paths, settings, title/artist, sender). Results return through the channel.
- `AlbumArt` uses `Arc<[u8]>` so cloning artwork into a task is cheap and immutable.
- `reqwest::Client` is a `OnceLock` singleton, safe to share.
- `TOKIO_HANDLE` is a `OnceLock`.
- Texture caches live only on the UI thread and need no locks.
- There are no `Mutex`, `RwLock`, `Atomic*`, or channels other than the single `mpsc`.

## 8. Concurrency invariants

1. Only one metadata fetch runs at a time (`is_fetching` global).
2. `fetch_index` is `Some(i)` exactly while a fetch is active; it is cleared on completion regardless of result.
3. `loading_count` reaches zero only after every spawned path has produced a message (assuming no panic).
4. Background code never touches `AppState`, `Song`, or egui.
5. Saving and loading the same file concurrently is possible: Add Songs reading a file while the user saves it from a detail view of an earlier copy. There is no file lock.

## 9. Failure and stuck-state behaviour

| Event | State after |
|---|---|
| Handler missing (no Tokio handle) | `is_fetching`/`fetch_index` reset; error banner. |
| LLM error | `MetadataFetched(Err)` → flags cleared, error banner. |
| iTunes error | Ignored; metadata succeeds. |
| Task panic | No message; `is_fetching` stays true (Fill disabled) until restart. |
| Library thread panic | `loading_count` stays > 0 (spinner/polling forever). |
| Window closed mid-load | Sends fail; threads exit; process ends. |

## 10. How to add background work safely

1. Define a new `AppMessage` variant with owned data and `Result<_, String>`.
2. Clone only `Send + 'static` values (paths, settings, `AlbumArt`, sender).
3. Spawn on `library-loader` style `std::thread` for filesystem/tag work, or `get_tokio_handle()` for HTTP.
4. Guard the spawn: if it can fail, roll back any counters and set an error.
5. Handle the message in `XurPorisoyApp::handle_message` and clear every "busy" flag on **both** success and error.
6. Make the UI poll while busy (`is_fetching || loading_count > 0`, or add a new counter/flag to that condition).
7. Never capture `&AppState`, `&mut Song`, or `egui::Context` in a background thread (the context is `Send`/`Sync`, but using it off-thread for textures is invalid).
8. If you need cancellation, add a generation/request id and ignore stale results rather than trying to abort the task.
