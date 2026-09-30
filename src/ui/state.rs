use crate::models::{AppSettings, Song, SongMetadata};
use crate::services::llm::LlmService;
use crate::services::metadata::MetadataService;
use crate::services::settings::SettingsService;
use eframe::egui;
use std::collections::HashMap;
use std::path::PathBuf;

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
    LoadSongs(Vec<PathBuf>),
    SongLoaded(usize, Result<Song>),
    MetadataFetched(usize, Result<SongMetadata>),
    SaveSong(usize),
    DeleteSong(usize),
    ChangeView(View),
    UpdateMetadata(usize, usize, String),
    ToggleFillDataButton(usize),
    SetError(String),
    ClearError,
    SettingsSaved,
    TestLlmConnection,
    LlmTestResult(Result<()>),
}

impl Clone for AppMessage {
    fn clone(&self) -> Self {
        match self {
            Self::LoadSongs(paths) => Self::LoadSongs(paths.clone()),
            Self::SongLoaded(idx, result) => Self::SongLoaded(*idx, result.clone()),
            Self::MetadataFetched(idx, result) => Self::MetadataFetched(*idx, result.clone()),
            Self::SaveSong(idx) => Self::SaveSong(*idx),
            Self::DeleteSong(idx) => Self::DeleteSong(*idx),
            Self::ChangeView(view) => Self::ChangeView(view.clone()),
            Self::UpdateMetadata(idx, field, value) => {
                Self::UpdateMetadata(*idx, *field, value.clone())
            }
            Self::ToggleFillDataButton(idx) => Self::ToggleFillDataButton(*idx),
            Self::SetError(msg) => Self::SetError(msg.clone()),
            Self::ClearError => Self::ClearError,
            Self::SettingsSaved => Self::SettingsSaved,
            Self::TestLlmConnection => Self::TestLlmConnection,
            Self::LlmTestResult(result) => Self::LlmTestResult(result.clone()),
        }
    }
}

pub type Result<T> = std::result::Result<T, String>;

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
    pub album_art_textures: HashMap<String, egui::TextureHandle>,
    pub confirm_discard_changes: bool,
}

impl AppState {
    pub fn new() -> Result<Self> {
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
            album_art_textures: HashMap::new(),
            confirm_discard_changes: false,
        })
    }

    pub fn add_songs(&mut self, paths: Vec<PathBuf>) {
        for path in paths {
            if let Ok(song) = MetadataService::load_song(&path) {
                self.songs.push(song);
            }
        }
        self.pending_changes = true;
    }

    pub fn can_go_back(&self) -> bool {
        matches!(self.current_view, View::Detail(_)) && !self.has_unsaved_changes()
    }

    #[allow(dead_code)]
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

    pub fn album_art_texture(
        &mut self,
        ctx: &egui::Context,
        path: &str,
        bytes: &[u8],
    ) -> Option<egui::TextureHandle> {
        use std::hash::{Hash, Hasher};

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        path.hash(&mut hasher);
        bytes.hash(&mut hasher);
        let key = format!("album-art-{}", hasher.finish());
        if !self.album_art_textures.contains_key(&key) {
            let image = image::load_from_memory(bytes).ok()?;
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [image.width() as usize, image.height() as usize],
                image.to_rgba8().as_flat_samples().as_slice(),
            );
            let texture = ctx.load_texture(&key, color_image, Default::default());
            self.album_art_textures.insert(key.clone(), texture);
        }
        self.album_art_textures.get(&key).cloned()
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
            if idx < self.songs.len() {
                MetadataService::write_metadata(&mut self.songs[idx]).map_err(|e| e.to_string())?;
                self.pending_changes = self.songs.iter().any(|s| s.has_changes);
            }
        }
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

    pub fn delete_song(&mut self, index: usize) {
        if index < self.songs.len() {
            self.songs.remove(index);
            self.pending_changes = true;
            if let View::Detail(idx) = self.current_view {
                if idx >= index {
                    self.current_view = View::Grid;
                    self.selected_index = None;
                }
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
