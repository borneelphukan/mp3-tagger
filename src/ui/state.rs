use crate::models::{AppSettings, Song, SongMetadata};
use crate::services::llm::LlmService;
use crate::services::metadata::MetadataService;
use crate::services::settings::SettingsService;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Grid,
    Detail(usize),
    Settings,
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
            Self::UpdateMetadata(idx, field, value) => Self::UpdateMetadata(*idx, *field, value.clone()),
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

    #[allow(dead_code)]
    pub fn can_go_back(&self) -> bool {
        matches!(self.current_view, View::Detail(_)) && (!self.pending_changes || self.has_unsaved_changes())
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
        if let View::Detail(idx) = self.current_view {
            if idx < self.songs.len() && self.songs[idx].has_changes {
                self.songs[idx].reset_changes();
            }
        }
        self.current_view = View::Grid;
        self.selected_index = None;
    }

    pub fn open_song_detail(&mut self, index: usize) {
        if index < self.songs.len() {
            self.selected_index = Some(index);
            self.current_view = View::Detail(index);
        }
    }

    pub fn update_metadata_field(&mut self, song_index: usize, field_index: usize, value: String) {
        if song_index < self.songs.len() {
            self.songs[song_index].metadata.set_field(field_index, value);
            self.songs[song_index].mark_changed();
            self.pending_changes = true;
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
                    self.songs[index].metadata = metadata;
                    self.songs[index].mark_changed();
                    self.pending_changes = true;
                }
            }
            Err(e) => {
                self.error_message = Some(e);
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
        self.settings_service.save(&self.settings).map_err(|e| e.to_string())?;
        Ok(())
    }
}