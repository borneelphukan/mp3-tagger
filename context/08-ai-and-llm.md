# 08 — AI and LLM Integration

The "AI" feature is a single OpenAI-compatible chat-completions call plus a best-effort iTunes cover lookup. There is no agent loop, tool calling, embedding, streaming, or local model runtime.

## 1. Configuration model

`LLMSettings` (`models/song.rs`):

```rust
pub struct LLMSettings {
    pub api_key: String,
    pub model_name: String,
    pub base_url: Option<String>,
    pub enabled: bool,
}
```

`is_configured()`:

```text
enabled
&& !model_name.is_empty()
&& ( !api_key.is_empty()
     || base_url starts with http://localhost
     || base_url starts with http://127.0.0.1
     || base_url starts with https://localhost
     || base_url starts with https://127.0.0.1 )
```

Consequences:

- A local server needs no API key, but the URL must start exactly with one of those four prefixes.
- A hosted provider **always** needs a key, even if the endpoint is open.
- `base_url = None` is not itself sufficient for a keyless local run.
- `enabled` is forced to `true` by the Save button and temporarily by Test connection.

The dialog exposes only `gpt-4o-mini`, `gpt-4o`, and a free-text field; any OpenAI-compatible model name works.

## 2. Endpoint construction

```rust
let base_url = settings.base_url.as_deref().unwrap_or("https://api.openai.com/v1");
let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
```

- A trailing slash on the base URL is tolerated.
- If the user enters `https://api.openai.com` without `/v1`, the app calls `https://api.openai.com/chat/completions` and fails. The UI hint/default includes `/v1`.
- If the user enters a full `/chat/completions` URL, the app appends another segment and fails.
- `Authorization: Bearer <api_key>` is added only when the key is non-empty.

## 3. HTTP client

All LLM and artwork requests share `services/http.rs::client()`:

```text
connect timeout 10 s
total request timeout 60 s
pool idle timeout 90 s
User-Agent xur-porisoy/<CARGO_PKG_VERSION>
rustls TLS with webpki roots
```

There is no per-request timeout except iTunes' explicit 20 s on each of its two requests. There is no retry, backoff, rate limiting, or proxy configuration.

## 4. Request payload

```json
{
  "model": "<settings.model_name>",
  "messages": [
    {
      "role": "system",
      "content": "You are a music metadata expert. Return only valid JSON."
    },
    {
      "role": "user",
      "content": "<build_prompt output>"
    }
  ],
  "temperature": 0.1,
  "max_tokens": 2000
}
```

`temperature = 0.1` is hard-coded for determinism; `max_tokens = 2000` bounds cost but can truncate long lyrics.

## 5. Prompt

`build_prompt(title, artist)` embeds the title and artist directly (no escaping beyond Rust formatting). It asks for exactly these 18 keys:

```text
title artist album album_artist year genre track_number total_tracks
disc_number total_discs composer lyricist remixer producer comment lyrics bpm isrc
```

Instructions:

- Return **only** a valid JSON object.
- Use `""` for unknown values.
- If the song cannot be found, return empty strings for all fields.

Artwork is deliberately absent from the prompt; it is fetched separately. The model is told it is a "music metadata expert"; there is no web search or date grounding, so "year" and credits are model guesses.

## 6. Response handling

1. Non-2xx status → `AppError::Llm("LLM API error (<status>): <body>")`.
2. JSON body → `LlmResponse { choices: Vec<LlmChoice> }`; only `choices[0].message.content` is used. Missing choices yield an empty string.
3. `parse_response` slices `response[first '{' .. last '}' + 1]`. This tolerates Markdown fences and prose around the object, but:
   - it fails if the object contains a `}` before the last one in surrounding prose,
   - it fails if the model returns multiple JSON objects,
   - it fails on truncated output (e.g. hit `max_tokens`).
4. The slice is deserialised as `SongMetadata`. `#[serde(default)]` means missing keys become empty strings/`None`.
5. If `metadata.is_empty()` → `AppError::SongNotFound("Song not found in LLM database")`.
6. Unknown keys are ignored by serde; wrong types (e.g. `"year": 1999`) fail the whole parse.

There is no JSON-schema enforcement, no repair pass, and no second attempt.

## 7. Artwork lookup (iTunes Search API)

`services/artwork.rs::fetch_cover_art(title, artist, album) -> Result<(Vec<u8>, String), String>`.

### Search

```text
GET https://itunes.apple.com/search
    ?term=<artist> <album> <title>
    &entity=song
    &limit=5
timeout: 20 s
```

- The term is space-joined even when album is empty (leading/trailing spaces).
- No `country`, `media`, `lang`, or explicit-content parameter.
- Any non-2xx or parse failure is a string error.

### Choose and upscale the URL

- Iterate results and take the **first** `artworkUrl100`.
- Replace the literal substring `100x100bb` with `600x600bb`. If Apple changes the URL pattern, the replacement silently does nothing and the 100×100 image is embedded.
- Download that URL with a 20 s timeout.

### Validate

- MIME comes from the response `Content-Type` only if it starts with `image/`; otherwise `image/jpeg`.
- `image::guess_format` must return `Png`, `Jpeg`, `Gif`, or `Bmp`; anything else is rejected.
- Only the header is inspected. The texture loader later bounds decoding to 4096×4096 and 256 MiB.

### Caller behaviour

Artwork is **best effort** inside the fetch task:

```rust
if let Ok((bytes, mime)) = fetch_cover_art(...).await {
    metadata.album_art = Some(AlbumArt::new(bytes));
    metadata.album_art_mime = Some(mime);
}
```

If the lookup fails, the metadata is still delivered. If the user already had artwork and the lookup fails, `complete_metadata_fetch` keeps the old artwork. If the lookup succeeds, it **replaces** the user's artwork without asking.

## 8. UI gating

| Condition | Effect |
|---|---|
| LLM not configured | Fill Data disabled; hint "⚠ Configure LLM in Settings". |
| Missing title or artist | Disabled; hint "⚠ Enter Title and Artist first". |
| `is_fetching` | Disabled everywhere and labelled `⏳ Fetching...` on the active song. |
| Tokio handle missing | Error "The async runtime is not available"; flags reset. |

Only one fill can run at a time because `is_fetching` is global, not per song. During a fill the user can still navigate and edit; the result is applied to `fetch_index`, which is the index captured at start. If songs are reordered or cleared in the meantime, the index may refer to a different song or be out of range; `complete_metadata_fetch` bounds-checks the index.

## 9. Test connection

`test_connection` requires `is_configured()`, then performs a normal `call_llm` with "Test connection. Reply with 'OK'." and discards the content. It therefore validates endpoint, key, model, and JSON envelope, but not metadata quality.

The settings dialog temporarily forces `enabled = true` for the clone it sends, restoring the draft's previous value, so an unsaved enabled=false state can still be tested.

## 10. Security and privacy notes

- The API key is stored **in plaintext** in `settings.json` (`~/.config/xur-porisoy/` or `%APPDATA%\xur-porisoy\`). File permissions are whatever the OS default is; the app does not tighten them.
- The key is never logged, but a non-2xx response body is surfaced verbatim and could contain provider diagnostics.
- Title/artist (and optionally album) are sent to the LLM provider and to Apple's public iTunes API. There is no consent prompt beyond pressing Fill Data.
- No certificate pinning; rustls uses webpki roots.
- There is no prompt-injection surface beyond song names being embedded in the prompt, but a crafted title could add instructions. The model's output is only deserialised into fixed string fields, so the blast radius is metadata content.

## 11. Known limitations

| Limitation | Consequence |
|---|---|
| No streaming | The UI shows an indeterminate "Fetching…" state until the full response arrives. |
| No cancellation | Navigating away does not stop the request; the result still applies. |
| No retry | A transient 429/5xx surfaces as an error. |
| `max_tokens = 2000` | Long lyrics can truncate the JSON and fail parsing. |
| First-`{`/last-`}` slicing | Prose containing braces or multiple objects breaks parsing. |
| One global fetch flag | Cannot fill two songs in parallel. |
| iTunes first result | No scoring by title/artist/album; the first of five results wins. |
| Hard-coded `100x100bb` | A changed Apple URL scheme silently yields low-resolution art. |
| Artwork always replaced | A successful lookup overwrites existing user artwork on Fill Data. |
| `base_url` prefix test | `http://localhost.evil.example` would be treated as local for the key check (it would fail to connect, but the keyless rule is prefix-based). |

## 12. Where to change behaviour

| Change | Location |
|---|---|
| Prompt wording/fields | `LlmService::build_prompt` and the `SongMetadata` field list. |
| Request parameters | `LlmService::call_llm` (`temperature`, `max_tokens`, messages). |
| Response parsing | `LlmService::parse_response`. |
| Add a provider preset | `ui/settings_view.rs` model list. |
| Change default endpoint | `call_llm` fallback and `settings_view` default. |
| Change artwork source | `services/artwork.rs`; update this doc and [07-features-and-flows.md](07-features-and-flows.md). |
| Allow keyless remote endpoints | `LLMSettings::is_configured`. |

If you add a real JSON-schema request/response, update this file and the "Wire contracts" section of [04-data-model.md](04-data-model.md) together.
