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
    pub conductor: String,
    pub remixer: String,
    pub producer: String,
    pub arranger: String,
    pub engineer: String,
    pub mixer: String,
    pub comment: String,
    pub lyrics: String,
    pub bpm: String,
    pub isrc: String,
    pub musicbrainz_track_id: String,
    pub musicbrainz_artist_id: String,
    pub musicbrainz_album_id: String,
    pub musicbrainz_album_artist_id: String,
    pub musicbrainz_release_group_id: String,
    pub musicbrainz_work_id: String,
    pub acoustid_fingerprint: String,
    pub acoustid_id: String,
    pub replaygain_track_gain: String,
    pub replaygain_track_peak: String,
    pub replaygain_album_gain: String,
    pub replaygain_album_peak: String,
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
            && self.conductor.is_empty()
            && self.remixer.is_empty()
            && self.producer.is_empty()
            && self.arranger.is_empty()
            && self.engineer.is_empty()
            && self.mixer.is_empty()
            && self.comment.is_empty()
            && self.lyrics.is_empty()
            && self.bpm.is_empty()
            && self.isrc.is_empty()
            && self.musicbrainz_track_id.is_empty()
            && self.musicbrainz_artist_id.is_empty()
            && self.musicbrainz_album_id.is_empty()
            && self.musicbrainz_album_artist_id.is_empty()
            && self.musicbrainz_release_group_id.is_empty()
            && self.musicbrainz_work_id.is_empty()
            && self.acoustid_fingerprint.is_empty()
            && self.acoustid_id.is_empty()
            && self.replaygain_track_gain.is_empty()
            && self.replaygain_track_peak.is_empty()
            && self.replaygain_album_gain.is_empty()
            && self.replaygain_album_peak.is_empty()
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
            "Conductor",
            "Remixer",
            "Producer",
            "Arranger",
            "Engineer",
            "Mixer",
            "Comment",
            "Lyrics",
            "BPM",
            "ISRC",
            "MusicBrainz Track ID",
            "MusicBrainz Artist ID",
            "MusicBrainz Album ID",
            "MusicBrainz Album Artist ID",
            "MusicBrainz Release Group ID",
            "MusicBrainz Work ID",
            "AcoustID Fingerprint",
            "AcoustID ID",
            "ReplayGain Track Gain",
            "ReplayGain Track Peak",
            "ReplayGain Album Gain",
            "ReplayGain Album Peak",
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
            12 => &self.conductor,
            13 => &self.remixer,
            14 => &self.producer,
            15 => &self.arranger,
            16 => &self.engineer,
            17 => &self.mixer,
            18 => &self.comment,
            19 => &self.lyrics,
            20 => &self.bpm,
            21 => &self.isrc,
            22 => &self.musicbrainz_track_id,
            23 => &self.musicbrainz_artist_id,
            24 => &self.musicbrainz_album_id,
            25 => &self.musicbrainz_album_artist_id,
            26 => &self.musicbrainz_release_group_id,
            27 => &self.musicbrainz_work_id,
            28 => &self.acoustid_fingerprint,
            29 => &self.acoustid_id,
            30 => &self.replaygain_track_gain,
            31 => &self.replaygain_track_peak,
            32 => &self.replaygain_album_gain,
            33 => &self.replaygain_album_peak,
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
            12 => self.conductor = value,
            13 => self.remixer = value,
            14 => self.producer = value,
            15 => self.arranger = value,
            16 => self.engineer = value,
            17 => self.mixer = value,
            18 => self.comment = value,
            19 => self.lyrics = value,
            20 => self.bpm = value,
            21 => self.isrc = value,
            22 => self.musicbrainz_track_id = value,
            23 => self.musicbrainz_artist_id = value,
            24 => self.musicbrainz_album_id = value,
            25 => self.musicbrainz_album_artist_id = value,
            26 => self.musicbrainz_release_group_id = value,
            27 => self.musicbrainz_work_id = value,
            28 => self.acoustid_fingerprint = value,
            29 => self.acoustid_id = value,
            30 => self.replaygain_track_gain = value,
            31 => self.replaygain_track_peak = value,
            32 => self.replaygain_album_gain = value,
            33 => self.replaygain_album_peak = value,
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
        self.enabled && !self.api_key.is_empty() && !self.model_name.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    pub llm: LLMSettings,
    pub last_opened_folder: Option<PathBuf>,
    pub window_size: Option<(f32, f32)>,
}
