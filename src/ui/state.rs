use crate::models::{AlbumArt, AppSettings, Song, SongMetadata};
use crate::services::llm::LlmService;
use crate::services::metadata::MetadataService;
use crate::services::settings::SettingsService;
use eframe::egui;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::mpsc::Sender;

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Grid,
    Detail(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum LibraryViewMode {
    #[default]
    List,
    Grid,
}

#[derive(Debug)]
pub enum AppMessage {
    /// A song finished loading on the background library thread.
    SongLoaded(Result<Song>),
    MetadataFetched(usize, Result<SongMetadata>),
    LlmTestResult(Result<()>),
}

pub type Result<T> = std::result::Result<T, String>;

/// Bounded texture cache.
///
/// Previously every decoded cover art image was kept for the lifetime of the
/// process, so browsing a large library could consume gigabytes of GPU memory.
/// Entries are now inserted in order and the oldest are evicted once the cap is
/// reached.
pub struct TextureCache {
    textures: HashMap<u64, egui::TextureHandle>,
    order: VecDeque<u64>,
    /// Images that failed to decode. Without this, a corrupt cover would be
    /// re-decoded on every frame.
    failed: HashSet<u64>,
    max_entries: usize,
}

impl TextureCache {
    fn new(max_entries: usize) -> Self {
        Self {
            textures: HashMap::with_capacity(max_entries.min(64)),
            order: VecDeque::with_capacity(max_entries.min(64)),
            failed: HashSet::new(),
            max_entries,
        }
    }

    pub fn get_or_load(
        &mut self,
        ctx: &egui::Context,
        art: &AlbumArt,
        max_side: u32,
    ) -> Option<egui::TextureHandle> {
        // `AlbumArt` fingerprints its bytes once, so this lookup is a single
        // map probe instead of hashing the whole image on every frame.
        let key = art.hash() ^ (u64::from(max_side) << 32);

        if let Some(texture) = self.textures.get(&key) {
            return Some(texture.clone());
        }
        if self.failed.contains(&key) {
            return None;
        }

        let Some(image) = crate::services::images::decode_thumbnail(art.bytes(), max_side) else {
            self.failed.insert(key);
            return None;
        };
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            [image.width() as usize, image.height() as usize],
            image.as_raw(),
        );
        let texture = ctx.load_texture(
            format!("album-art-{key}"),
            color_image,
            egui::TextureOptions::LINEAR,
        );

        while self.textures.len() >= self.max_entries {
            let Some(oldest) = self.order.pop_front() else {
                break;
            };
            self.textures.remove(&oldest);
        }

        self.textures.insert(key, texture.clone());
        self.order.push_back(key);
        Some(texture)
    }
}

pub struct AppState {
    pub songs: Vec<Song>,
    pub current_view: View,
    pub selected_index: Option<usize>,
    pub settings: AppSettings,
    pub error_message: Option<String>,
    pub is_fetching: bool,
    pub fetch_index: Option<usize>,
    pub llm_service: LlmService,
    pub settings_service: SettingsService,
    pub pending_changes: bool,
    pub library_view_mode: LibraryViewMode,
    /// Textures for the small library tiles.
    pub album_art_textures: TextureCache,
    /// Textures for the larger detail-view preview, capped separately so
    /// browsing many songs cannot grow this without bound.
    pub detail_album_art_textures: TextureCache,
    pub loading_count: usize,
    pub message_tx: Sender<AppMessage>,
    pub confirm_discard_changes: bool,
}

impl AppState {
    pub fn new(message_tx: Sender<AppMessage>) -> Result<Self> {
        let settings_service = SettingsService::new().map_err(|e| e.to_string())?;
        let settings = settings_service.load().map_err(|e| e.to_string())?;

        Ok(Self {
            songs: Vec::new(),
            current_view: View::Grid,
            selected_index: None,
            settings,
            error_message: None,
            is_fetching: false,
            fetch_index: None,
            llm_service: LlmService::new(),
            settings_service,
            pending_changes: false,
            library_view_mode: LibraryViewMode::List,
            album_art_textures: TextureCache::new(512),
            detail_album_art_textures: TextureCache::new(8),
            loading_count: 0,
            message_tx,
            confirm_discard_changes: false,
        })
    }

    /// Read tags on a background thread and stream results back to the UI.
    ///
    /// Tag parsing (especially embedded artwork) is I/O and CPU heavy. Doing it
    /// inline froze the window while a folder was added.
    pub fn add_songs(&mut self, paths: Vec<PathBuf>) {
        if paths.is_empty() {
            return;
        }

        let count = paths.len();
        self.loading_count += count;
        self.pending_changes = true;

        let sender = self.message_tx.clone();
        let spawned = std::thread::Builder::new()
            .name("library-loader".to_owned())
            .spawn(move || {
                for path in paths {
                    let song = MetadataService::load_song(&path).map_err(|e| e.to_string());
                    if sender.send(AppMessage::SongLoaded(song)).is_err() {
                        break;
                    }
                }
            });

        if let Err(error) = spawned {
            self.loading_count = self.loading_count.saturating_sub(count);
            self.error_message = Some(format!("Could not start loading songs: {error}"));
        }
    }

    pub fn song_loaded(&mut self, result: Result<Song>) {
        self.loading_count = self.loading_count.saturating_sub(1);
        match result {
            Ok(song) => self.songs.push(song),
            Err(error) => {
                self.error_message = Some(format!("Could not read a song's metadata: {error}"))
            }
        }
    }

    pub fn can_go_back(&self) -> bool {
        matches!(self.current_view, View::Detail(_)) && !self.has_unsaved_changes()
    }

    pub fn has_unsaved_changes(&self) -> bool {
        if let View::Detail(idx) = self.current_view {
            if idx < self.songs.len() {
                return self.songs[idx].has_changes;
            }
        }
        false
    }

    pub fn go_back(&mut self) {
        if !self.can_go_back() {
            return;
        }
        self.current_view = View::Grid;
        self.selected_index = None;
    }

    pub fn confirm_discard_and_go_back(&mut self) {
        if let View::Detail(index) = self.current_view {
            if let Some(song) = self.songs.get_mut(index) {
                song.metadata = song.original_metadata.clone();
                song.has_changes = false;
            }
        }
        self.confirm_discard_changes = false;
        self.current_view = View::Grid;
        self.selected_index = None;
    }

    pub fn detail_album_art(
        &mut self,
        ctx: &egui::Context,
        art: &AlbumArt,
    ) -> Option<egui::TextureHandle> {
        // 320px covers the 300px detail preview with a little headroom.
        self.detail_album_art_textures.get_or_load(ctx, art, 320)
    }

    pub fn open_song_detail(&mut self, index: usize) {
        if index < self.songs.len() {
            self.selected_index = Some(index);
            self.current_view = View::Detail(index);
        }
    }

    pub fn update_metadata_field(&mut self, song_index: usize, field_index: usize, value: String) {
        if song_index < self.songs.len() {
            self.songs[song_index]
                .metadata
                .set_field(field_index, value);
            self.songs[song_index].mark_changed();
            self.pending_changes = true;
        }
    }

    pub fn clear_current_metadata(&mut self) {
        if let View::Detail(idx) = self.current_view {
            if let Some(song) = self.songs.get_mut(idx) {
                song.metadata = SongMetadata::default();
                song.mark_changed();
                self.pending_changes = true;
            }
        }
    }

    pub fn save_current_song(&mut self) -> Result<()> {
        if let View::Detail(idx) = self.current_view {
            self.save_song_at(idx)?;
        }
        Ok(())
    }

    pub fn save_song_at(&mut self, index: usize) -> Result<()> {
        let song = self
            .songs
            .get_mut(index)
            .ok_or_else(|| "That song is no longer in the library".to_string())?;
        MetadataService::write_metadata(song).map_err(|e| e.to_string())?;
        self.pending_changes = self.songs.iter().any(|s| s.has_changes);
        Ok(())
    }

    pub fn start_metadata_fetch(&mut self, index: usize) {
        if index < self.songs.len() {
            let song = &self.songs[index];
            if song.metadata.has_required_for_llm() && self.settings.llm.is_configured() {
                self.is_fetching = true;
                self.fetch_index = Some(index);
            }
        }
    }

    pub fn complete_metadata_fetch(&mut self, index: usize, result: Result<SongMetadata>) {
        self.is_fetching = false;
        self.fetch_index = None;

        match result {
            Ok(metadata) => {
                if index < self.songs.len() {
                    let existing = &self.songs[index].metadata;
                    let mut metadata = metadata;
                    if metadata.lyrics.trim().is_empty() {
                        metadata.lyrics = existing.lyrics.clone();
                    }
                    if metadata.album_art.is_none() {
                        metadata.album_art = existing.album_art.clone();
                        metadata.album_art_mime = existing.album_art_mime.clone();
                    }
                    self.songs[index].metadata = metadata;
                    self.songs[index].mark_changed();
                    self.pending_changes = true;
                }
            }
            Err(_) => {
                self.error_message = Some(
                    "Could not fetch the song’s metadata. Please enter the metadata manually"
                        .to_string(),
                );
            }
        }
    }

    pub fn save_settings(&mut self) -> Result<()> {
        self.settings_service
            .save(&self.settings)
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
