# XurPorisoy

XurPorisoy is a free desktop app for organizing songs and editing the information saved in their audio files. You can update details such as the title, artist, album, and cover art, or use an AI service to help fill in missing information.

[Download XurPorisoy](https://github.com/borneelphukan/xur-porisoy/releases) · [Get help or report a problem](https://github.com/borneelphukan/xur-porisoy/issues)

## What you can do

- Browse your music in a list or grid.
- Edit song details and view album artwork.
- Add one song or several songs at once.
- Save changes directly to your audio files.
- Use an AI service to suggest song information.
- Work with MP3, FLAC, OGG, M4A/MP4, WAV, AIFF, and APE files.

## Get started

1. [Download XurPorisoy](https://github.com/borneelphukan/xur-porisoy/releases) for your computer.
2. Install and open the app.
3. Select **Add Songs** and choose one or more audio files.
4. Select a song to view or edit its information.
5. Select **Save** to write your changes to the audio file.

## Everyday use

| To… | Do this |
|---|---|
| Add music | Select **Add Songs** and choose audio files |
| View or edit a song | Select its title; double-clicking also opens its details |
| Change song information | Edit the fields in the song details |
| Add cover art | Use the artwork controls in the song details |
| Ask AI to fill in information | Enter the song's title and artist, then select **Fill Data** |
| Save your changes | Select **Save** (or press **Enter** in the details view) |
| Return to your music library | Select **Back to Grid** or press **Escape** |

## Optional: Set up AI suggestions

AI suggestions require an API key from an OpenAI-compatible service. The service may charge for using its API; check its pricing before adding a key.

1. Open **AI → LLM Settings** from the menu.
2. Enter your API key and model name. The model name depends on your service.
3. If your service uses a custom address, enter its base URL too.
4. Select **Save**.

To use **Fill Data**, make sure the song has both a title and an artist. You can still edit and save song information without configuring AI.

## Supported audio files

XurPorisoy can read and write metadata in MP3, FLAC, OGG, M4A/MP4, WAV, AIFF, and APE files. The kinds of information available can vary by file format. Common fields include title, artist, album, year, genre, track and disc numbers, credits, comments, lyrics, BPM, and embedded cover art.

## If something goes wrong

- **AI suggestions are unavailable:** Add an API key and model in **AI → LLM Settings**, and make sure the song has a title and artist.
- **Changes are not showing in another music app:** Confirm that you selected **Save**. Some players may need to refresh their library or reopen the file.
- **A file or field does not behave as expected:** Metadata support varies by format. [Report a problem](https://github.com/borneelphukan/xur-porisoy/issues) and include the file format and what you were trying to do.

## About the project

XurPorisoy is free and open source under the [MIT License](LICENSE). You can [browse the source code](https://github.com/borneelphukan/xur-porisoy) or [report a problem](https://github.com/borneelphukan/xur-porisoy/issues).

### For developers

To build and run the app locally, install [Rust](https://www.rust-lang.org/tools/install), then run `cargo run` from the project directory. Linux builds may also require system development libraries.
