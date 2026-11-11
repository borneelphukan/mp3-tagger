# 05 — Metadata Pipeline

This document follows tags from "user selects files" through "bytes are written back to disk". All of it lives in `services/metadata.rs` and `ui/detail_view.rs`, with `lofty` doing the format work.

## 1. Scope

- Reads and writes **tags and embedded pictures only**.
- Audio properties (bitrate, sample rate, duration, channels) are deliberately never parsed: `ParseOptions::new().read_properties(false)`.
- Supported by the file dialog: `mp3`, `flac`, `ogg`, `m4a`, `wav`, `aiff`, `ape`. The actual capability comes from `lofty`; the extension list is duplicated in `ui/mod.rs` (File menu) and `ui/grid_view.rs` (toolbar).

## 2. Reading a song

`MetadataService::load_song(path)` → `read_metadata(path)` → `Song::new(path, metadata)`.

### 2.1 Open and parse

```rust
Probe::open(path)?
    .options(ParseOptions::new().read_properties(false))
    .read()?
```

`read_tagged_file` converts any failure into `AppError::AudioMetadataParse`.

### 2.2 Choose the tag

```rust
tagged_file.primary_tag().or_else(|| tagged_file.first_tag())
```

The **primary** tag for the format wins (ID3v2 for MP3, Vorbis Comments for FLAC/OGG, MP4 ilst for M4A, and so on). If no primary tag exists, the first available tag is used. A file with no tag at all simply yields an all-empty `SongMetadata`; that is not an error.

### 2.3 Extract text

Two passes over the same tag:

1. `lofty::tag::Accessor` convenience accessors fill `artist`, `title`, `album`, `genre`, `track_number`, and `comment`.
2. A loop over `tag.items()` matches `ItemKey` variants and fills the rest.

| `ItemKey` | Field |
|---|---|
| `TrackTitle` | `title` |
| `TrackArtist` | `artist` |
| `AlbumTitle` | `album` |
| `AlbumArtist` | `album_artist` |
| `RecordingDate`, `Year` | `year` |
| `Genre` | `genre` |
| `TrackNumber` | `track_number` |
| `TrackTotal` | `total_tracks` |
| `DiscNumber` | `disc_number` |
| `DiscTotal` | `total_discs` |
| `Composer` | `composer` |
| `Lyricist` | `lyricist` |
| `Remixer` | `remixer` |
| `Producer` | `producer` |
| `Comment` | `comment` |
| `UnsyncLyrics` | `lyrics` |
| `IntegerBpm` | `bpm` |
| `Isrc` | `isrc` |

Ordering note: the `ItemKey` loop runs **after** the `Accessor` pass, so if both expose a value the `ItemKey` value wins. Within the loop, later matching items overwrite earlier ones.

Only textual item values are read: `item.value().text().unwrap_or("")`. Binary values are ignored.

### 2.4 Extract artwork

Artwork is read from the chosen tag first (`tag.pictures().first()`). Only the **first** picture is used, regardless of its `PictureType`. If none was found:

- try `tagged_file.tag(TagType::Id3v2)`, else
- try `tagged_file.tag(TagType::Mp4Ilst)`.

This fallback covers formats where the primary tag is Vorbis Comments but a companion ID3v2/MP4 tag holds the cover. Bytes are wrapped in `AlbumArt::new`; MIME is stored as `picture.mime_type().to_string()` when present.

## 3. Loading a batch

`AppState::add_songs(paths)`:

1. Ignores an empty list.
2. `loading_count += paths.len()`, `pending_changes = true`.
3. Spawns `std::thread` named `library-loader` with a cloned `message_tx`.
4. For each path: `MetadataService::load_song` → `AppMessage::SongLoaded(Result<Song, String>)`; stops if the receiver is gone.

One message per path means progress is visible and one corrupt file cannot abort the batch. `load_songs` (the synchronous helper) exists but is `#[allow(dead_code)]` and unused by the UI.

## 4. Editing

- Detail-view fields call `AppState::update_metadata_field(index, field_index, value)`, which writes via `set_field`, calls `Song::mark_changed`, and sets `pending_changes`.
- `Clear` replaces the whole metadata with `SongMetadata::default()`, marks changed. On save this removes every exposed key and the artwork.
- `Load Artwork` reads a local file with `std::fs::read`, infers MIME from the extension (`png`, `gif`, `bmp`, else `image/jpeg`), stores the bytes and MIME, marks changed.
- `Remove Artwork` sets both `album_art` and `album_art_mime` to `None`, marks changed.
- `Fill Data` replaces the metadata, then preserves existing lyrics when the LLM returns blank lyrics and preserves existing artwork when no new artwork was fetched (see [08-ai-and-llm.md](08-ai-and-llm.md)).

## 5. Writing a song

`AppState::save_current_song` → `save_song_at(index)` → `MetadataService::write_metadata(song)`. This is **synchronous on the UI thread**.

### 5.1 Re-open and choose a writable tag

The file is re-read with `read_tagged_file` (properties still off). The target is `tagged_file.tag_mut(primary_tag_type)`; if that returns `None`, `first_tag_mut()` is used. If neither exists, the save fails with `AppError::InvalidMetadata("No writable tag found")` — the app does **not** create a new tag.

### 5.2 Text write is a replace, not a merge

`write_tag_data` uses a macro around `Tag::insert_text` / `Tag::remove_key`:

- Non-empty UI value → `insert_text(key, value)`.
- Empty UI value → `remove_key(key)` (deletes the key entirely).

For every exposed field:

| Field | Keys written |
|---|---|
| title | `TrackTitle` |
| artist | `TrackArtist` |
| album | `AlbumTitle` |
| album_artist | `AlbumArtist` |
| year | **both** `RecordingDate` and `Year` |
| genre | `Genre` |
| track_number | `TrackNumber` |
| total_tracks | `TrackTotal` |
| disc_number | `DiscNumber` |
| total_discs | `DiscTotal` |
| composer | `Composer` |
| lyricist | `Lyricist` |
| remixer | `Remixer` |
| producer | `Producer` |
| comment | `Comment` |
| lyrics | `UnsyncLyrics` |
| bpm | `IntegerBpm` |
| isrc | `Isrc` |

This means saving a song can **delete** tags the editor does not show.

### 5.3 Fixed key cleanup

After the exposed fields, these keys are always removed, even if they had values:

```text
MusicBrainzRecordingId, MusicBrainzTrackId, MusicBrainzArtistId,
MusicBrainzReleaseId, MusicBrainzReleaseArtistId, MusicBrainzReleaseGroupId,
MusicBrainzWorkId,
Conductor, Arranger, Engineer, MixEngineer,
AcoustId, AcoustIdFingerprint,
ReplayGainTrackGain, ReplayGainTrackPeak,
ReplayGainAlbumGain, ReplayGainAlbumPeak
```

This is intentional lossy behaviour, not a bug: the app presents itself as a simple editor and strips machine-generated/acoustid/credit metadata on every save. Any code change that needs to preserve them must remove the corresponding loop.

### 5.4 Artwork write

Artwork is handled separately from text:

```rust
let replace_artwork = song.metadata.album_art != song.original_metadata.album_art;
```

- If the artwork differs (including "removed", `Some` → `None`), **all** existing pictures are removed by repeatedly calling `tag.remove_picture(0)` `pictures().len()` times.
- If the new metadata has artwork, one `Picture` is pushed with `PictureType::CoverFront`.
- MIME mapping: `image/png` → `Png`, `image/gif` → `Gif`, `image/bmp` → `Bmp`, anything else (including `image/jpeg` and `None`) → `Jpeg`.
- `Picture::unchecked` is used; the bytes are not re-decoded here.

If the artwork is unchanged, existing pictures are left untouched.

### 5.5 Commit

`tagged_file.save_to_path(&song.path, Default::default())` writes in place. On success:

- `Song::save_changes()` copies `metadata` into `original_metadata` and clears `has_changes`;
- `AppState` recomputes `pending_changes = songs.iter().any(|s| s.has_changes)`;
- the detail view navigates back to the library.

On failure the file may be partially written (there is no atomic temp-file rename), the error is shown, and the song stays dirty.

## 6. Unsaved changes and navigation

- `can_go_back()` = currently in `Detail` and the song is clean.
- Back button with `has_changes` sets `confirm_discard_changes`; the dialog offers Yes (`confirm_discard_and_go_back`, restores `original_metadata`) or No.
- Saving also navigates back, but only after a successful write.
- There is no autosave and no "save all".

## 7. Failure matrix

| Failure | Result |
|---|---|
| Probe/open fails | `AudioMetadataParse`; a per-song error banner for batch loads. |
| No tag on read | All fields empty; song still added. |
| No tag on write | `InvalidMetadata`; nothing written. |
| `save_to_path` fails | `AudioMetadataEncode`; UI error; song remains dirty. |
| Artwork file unreadable | "Could not load artwork"; metadata unchanged. |
| Corrupt embedded art | Read succeeds; the texture cache marks the key failed and shows a placeholder. |

## 8. Where to change behaviour

| Change | Location |
|---|---|
| Add/remove an exposed field | `SongMetadata`, `field_names`, `get_field`/`set_field`, `extract_tag_data`, `write_tag_data`, then the LLM prompt. |
| Stop deleting a fixed key | Remove it from the two arrays in `write_tag_data`. |
| Write extra keys | Add `set_if_not_empty!` calls in `write_tag_data`. |
| Change artwork MIME handling | `write_tag_data` MIME match and `show_album_art` extension match. |
| Make saves atomic | Wrap `save_to_path` with a temp file + rename (format-aware; not trivial). |
| Move saves off the UI thread | `AppState::save_song_at` and the `Save` button, plus a new `AppMessage` variant. |
