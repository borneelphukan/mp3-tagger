# 06 — UI and Interactions

The UI is pure `egui` immediate mode. There is no retained widget tree, no CSS, and no asset pipeline beyond the embedded window icon. Every frame rebuilds panels from `AppState`.

## 1. Frame skeleton

`impl eframe::App for XurPorisoyApp::update`:

```text
sync theme (if settings.theme changed)
drain all queued AppMessage values
TopBottomPanel::top("menu_bar")        File | AI | View | Help, right-aligned "📚 Library"
match current_view {
    Grid   => GridView::show(ctx, &mut state)
    Detail => DetailView::show(ctx, &mut state)
}
optional SettingsView popup
optional About window
optional Error window
request_repaint_after(100 ms) if is_fetching or loading_count > 0
```

Dialogs are checked after the main content so they float above it. egui handles window ordering and focus.

## 2. Menu bar (`ui/mod.rs`)

| Menu | Item | Behaviour |
|---|---|---|
| File | Add Songs | `rfd::FileDialog` multi-select, filter `mp3 flac ogg m4a wav aiff ape` → `state.add_songs`. |
| File | Quit | `ViewportCommand::Close`. |
| AI | LLM Settings | Re-clones `settings.llm` into `ai_settings_draft`, derives local mode from the base URL, opens the popup. |
| View | Theme → Default / Dark / Light | `selectable_value` on `settings.theme`, saves immediately. The submenu is widened to 150 px. |
| Help | About | Opens the About window. |

The right side of the bar shows `📚 Library` only in `View::Grid`.

## 3. Library view (`ui/grid_view.rs`)

### Toolbar

- `➕ Add Songs` (same dialog as the menu).
- `🗑 Clear All` — `songs.clear()` and `pending_changes = true`. It does **not** reset `is_fetching`, `fetch_index`, selection, or texture caches.
- `View:` label plus `📋 List` / `🔲 Grid` `selectable_label`s; changes `library_view_mode` for the session only.
- Right-aligned count: `"N songs"` or `"N songs · loading M songs…"`.

### Empty state

When `songs.is_empty()`: "Reading tags…" if `loading_count > 0`, otherwise "No songs added yet." / "Click 'Add Songs' to get started." The function returns early, so no table/grid is built.

### List mode

`egui_extras::TableBuilder` with seven auto columns:

| Column | Minimum width | Content |
|---|---|---|
| `#` | 40 | `index + 1` |
| Title | 200 | `metadata.title`, or the file name if empty |
| Artist | 150 | `metadata.artist` |
| Album | 150 | `metadata.album` |
| Year | 60 | `metadata.year` |
| Genre | 100 | `metadata.genre` |
| Track | 80 | `"—"`, `track`, or `track/total` |

`body.rows(28.0, songs.len(), ...)` virtualises rows, so only visible rows allocate widgets. Clicking **any cell** opens the song: each cell draws its label and then calls `track_list_row_click`, which allocates a click-sensing rect with id `("song_list_cell", song_index, column_index)`.

### Grid mode

Constants: tile width 112, spacing 12, row height 126, cover display 72×72.

- `columns = floor((available_width + 12) / (112 + 12)).max(1)`.
- `ScrollArea::vertical().show_rows(...)` virtualises entire rows.
- The grid splits borrows into `let songs = &state.songs; let textures = &mut state.album_art_textures;` so tiles never clone artwork or titles.
- Cover: `textures.get_or_load(ctx, art, 96)` → 72×72 image; on failure or no art → a 72×72 button labelled `♫`.
- Title label is truncated/wrapped into 112×36 and is also clickable.
- Clicking the cover or label records `clicked_idx`; `GridView::show` applies `open_song_detail` after rendering to avoid borrow conflicts.

## 4. Detail view (`ui/detail_view.rs`)

Layout is three fixed regions plus an optional dialog.

### Top bar

- `‹ Back to Library`: with unsaved changes, opens the discard dialog; otherwise `go_back()`.
- Display name and file name.
- Right side: `🔮 Fill Data` (or `⏳ Fetching...` while this song is being fetched).
- Inline hints: "⚠ Configure LLM in Settings" or "⚠ Enter Title and Artist first" when the button is disabled.

Enable condition (`can_fill`): title and artist non-blank, LLM configured, and `!is_fetching`. The button adds `!is_fetching` again.

### Bottom bar

- `Clear` — resets all metadata.
- `💾 Save` — enabled only when `has_changes`. On success navigates back; on failure sets `error_message`.

### Central panel: 65 / 35 split

Left (65% width, vertical scroll):

- Every `field_names()` entry except index 15, rendered as a bold label + full-width single-line `TextEdit`.
- Edits are applied on `response.changed()` **and** when the value differs from the frame's snapshot.
- Title/Artist show "Required for LLM lookup" while empty.
- Lyrics label + 5-row multiline `TextEdit`.

Right (35%, centered):

- "Album Artwork" heading.
- 300×300 preview from `detail_album_art` (decoded at 320 px).
- "Failed to load artwork image" when bytes exist but decode failed; "No album artwork" when absent.
- `📁 Load Artwork` — image filter `png jpg jpeg gif bmp`; reads bytes, infers MIME from extension, marks changed.
- `Remove Artwork` — enabled only when art exists.

Because the panes are `allocate_ui_with_layout` calls inside a horizontal layout, the split is proportional, not draggable.

### Unsaved-changes dialog

Centered, non-resizable, shown when `state.confirm_discard_changes`:
"You have unsaved changes. Any changes made will be lost. Go back?" → **Yes** restores `original_metadata` and returns to the grid; **No** just closes the dialog.

## 5. AI Settings popup (`ui/settings_view.rs`)

Fixed 460 px wide, 100 px high (egui may grow it), non-collapsible.

- Header "LLM Service Configuration".
- Group with a right-to-left toggle: `Local LLM` (true) / `API Key` (false).
- Model combo: `gpt-4o-mini`, `gpt-4o`, `Custom` (selecting Custom clears the model only if it was empty).
- "Custom Model Name" text field bound directly to `draft.model_name`.
- Local mode: "Endpoint URL" defaulting to `http://localhost:1234/v1`.
- API mode: password-masked "API Key" (`sk-...`) and "Endpoint URL" defaulting to `https://api.openai.com/v1`.
- Inline error only if `error_message` contains `"Connection"`, `"connection"`, or `"LLM"`.
- `Cancel` closes without committing.
- `Save` is enabled when model is non-blank and (local mode or a non-blank key). It forces `enabled = true`, copies the draft into `settings.llm`, persists, and closes on success.
- `Test connection` uses the same enablement gate. It temporarily sets `draft.enabled = true`, clones the settings, restores the previous enabled value, and spawns the test.

The `draft` is reset from `settings.llm` every time the dialog is opened from the menu, so Cancel discards edits.

## 6. About window

Centered, default 760×560, minimum 600×460, resizable. Content is static: purple `♫` badge, title, description, author/copyright, fake "Credits / Website / Release Notes" labels (they are plain labels, not links), and a version grid using `CARGO_PKG_VERSION`, OS/arch, developer. A bottom-up `Close` button closes it.

## 7. Error window

A centered, non-resizable window titled "Error" whenever `error_message` is `Some`. The message is red; `OK` clears it.

There is no severity level and no queue: a new error overwrites the previous one. Success text from a connection test also goes through this window.

## 8. Theme and styling

- `setup_style` applies to both light and dark styles: item spacing `(8, 6)`, button padding `(12, 6)`, interact size `(100, 28)`.
- `apply_theme` maps `ThemePreference` onto `egui::ThemePreference` and calls `ctx.set_theme`.
- The theme is re-applied when `settings.theme` differs from `applied_theme`, then persisted by the menu action.
- System theme follows the OS.

## 9. What is *not* implemented

Despite the root README:

- **No double-click to open.** List rows open on a single click; grid tiles open on a single click.
- **No Enter-to-save** and **no Escape-to-go-back**. There are no keyboard handlers anywhere.
- **No drag-and-drop** file opening.
- **No column sorting or resizing** (`resizable(false)`).
- **No search/filter**.
- **No multi-select** in the library, despite the file dialog supporting multi-file add.

See [14-conventions-and-gotchas.md](14-conventions-and-gotchas.md).

## 10. Accessibility and i18n

`accesskit` is disabled, so there is no screen-reader tree. All strings are hard-coded English literals. Emoji are used as icons.

## 11. Interaction state machine

```text
Grid ──click row/tile──► Detail(i)
Detail(i) ──Back, clean──► Grid
Detail(i) ──Back, dirty──► discard dialog ──Yes──► Grid (metadata restored)
                                          └─No───► Detail(i)
Detail(i) ──Save success──► Grid
Detail(i) ──index out of range──► Grid
```

`selected_index` is set when opening detail but is otherwise unused by the views. `library_view_mode` survives navigation but not restart.
