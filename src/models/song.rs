use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SongMetadata {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: String,
    pub year: String,
    pub genre: String,
    pub track_number: String,
    pub total_tracks: String,
    pub disc_number: String,
    pub total_discs: String,
    pub composer: String,
    pub lyricist: String,
    pub remixer: String,
    pub producer: String,
    pub comment: String,
    pub lyrics: String,
    pub bpm: String,
    pub isrc: String,
    pub album_art: Option<Vec<u8>>,
    pub album_art_mime: Option<String>,
}

impl SongMetadata {
    pub fn is_empty(&self) -> bool {
        self.title.is_empty()
            && self.artist.is_empty()
            && self.album.is_empty()
            && self.album_artist.is_empty()
            && self.year.is_empty()
            && self.genre.is_empty()
            && self.track_number.is_empty()
            && self.total_tracks.is_empty()
            && self.disc_number.is_empty()
            && self.total_discs.is_empty()
            && self.composer.is_empty()
            && self.lyricist.is_empty()
            && self.remixer.is_empty()
            && self.producer.is_empty()
            && self.comment.is_empty()
            && self.lyrics.is_empty()
            && self.bpm.is_empty()
            && self.isrc.is_empty()
            && self.album_art.is_none()
    }

    pub fn has_required_for_llm(&self) -> bool {
        !self.title.trim().is_empty() && !self.artist.trim().is_empty()
    }

    pub fn field_names() -> &'static [&'static str] {
        &[
            "Title",
            "Artist",
            "Album",
            "Album Artist",
            "Year",
            "Genre",
            "Track Number",
            "Total Tracks",
            "Disc Number",
            "Total Discs",
            "Composer",
            "Lyricist",
            "Remixer",
            "Producer",
            "Comment",
            "Lyrics",
            "BPM",
            "ISRC",
        ]
    }

    pub fn get_field(&self, index: usize) -> &str {
        match index {
            0 => &self.title,
            1 => &self.artist,
            2 => &self.album,
            3 => &self.album_artist,
            4 => &self.year,
            5 => &self.genre,
            6 => &self.track_number,
            7 => &self.total_tracks,
            8 => &self.disc_number,
            9 => &self.total_discs,
            10 => &self.composer,
            11 => &self.lyricist,
            12 => &self.remixer,
            13 => &self.producer,
            14 => &self.comment,
            15 => &self.lyrics,
            16 => &self.bpm,
            17 => &self.isrc,
            _ => "",
        }
    }

    pub fn set_field(&mut self, index: usize, value: String) {
        match index {
            0 => self.title = value,
            1 => self.artist = value,
            2 => self.album = value,
            3 => self.album_artist = value,
            4 => self.year = value,
            5 => self.genre = value,
            6 => self.track_number = value,
            7 => self.total_tracks = value,
            8 => self.disc_number = value,
            9 => self.total_discs = value,
            10 => self.composer = value,
            11 => self.lyricist = value,
            12 => self.remixer = value,
            13 => self.producer = value,
            14 => self.comment = value,
            15 => self.lyrics = value,
            16 => self.bpm = value,
            17 => self.isrc = value,
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    pub path: PathBuf,
    pub metadata: SongMetadata,
    pub original_metadata: SongMetadata,
    pub has_changes: bool,
    pub last_modified: DateTime<Utc>,
}

impl Song {
    pub fn new(path: PathBuf, metadata: SongMetadata) -> Self {
        let original = metadata.clone();
        Self {
            path,
            metadata,
            original_metadata: original,
            has_changes: false,
            last_modified: Utc::now(),
        }
    }

    pub fn mark_changed(&mut self) {
        self.has_changes = true;
        self.last_modified = Utc::now();
    }

    pub fn save_changes(&mut self) {
        self.original_metadata = self.metadata.clone();
        self.has_changes = false;
    }

    pub fn file_name(&self) -> String {
        self.path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string()
    }

    pub fn display_name(&self) -> String {
        if !self.metadata.title.is_empty() {
            format!("{} - {}", self.metadata.artist, self.metadata.title)
        } else {
            self.file_name()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LLMSettings {
    pub api_key: String,
    pub model_name: String,
    pub base_url: Option<String>,
    pub enabled: bool,
}

impl LLMSettings {
    pub fn is_configured(&self) -> bool {
        self.enabled
            && !self.model_name.is_empty()
            && (!self.api_key.is_empty()
                || self.base_url.as_deref().is_some_and(|url| {
                    url.starts_with("http://localhost")
                        || url.starts_with("http://127.0.0.1")
                        || url.starts_with("https://localhost")
                        || url.starts_with("https://127.0.0.1")
                }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    pub llm: LLMSettings,
    pub last_opened_folder: Option<PathBuf>,
    pub window_size: Option<(f32, f32)>,
    #[serde(default)]
    pub theme: ThemePreference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThemePreference {
    Light,
    Dark,
    #[default]
    System,
}
