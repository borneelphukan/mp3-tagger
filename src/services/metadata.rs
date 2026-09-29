use crate::models::{AppError, Result, Song, SongMetadata};
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::picture::{MimeType, Picture, PictureType};
use lofty::probe::Probe;
use lofty::tag::{Accessor, ItemKey, Tag, TagType};
use std::path::Path;

pub struct MetadataService;

impl MetadataService {
    pub fn read_metadata(path: &Path) -> Result<SongMetadata> {
        let tagged_file = Probe::open(path)
            .map_err(|e| AppError::AudioMetadataParse(e.to_string()))?
            .read()
            .map_err(|e| AppError::AudioMetadataParse(e.to_string()))?;

        let mut metadata = SongMetadata::default();

        if let Some(tag) = tagged_file.primary_tag() {
            Self::extract_tag_data(tag, &mut metadata);
        } else if let Some(tag) = tagged_file.first_tag() {
            Self::extract_tag_data(tag, &mut metadata);
        }

        if metadata.album_art.is_none() {
            if let Some(tag) = tagged_file.tag(TagType::Id3v2) {
                Self::extract_artwork(tag, &mut metadata);
            } else if let Some(tag) = tagged_file.tag(TagType::Mp4Ilst) {
                Self::extract_artwork(tag, &mut metadata);
            }
        }

        Ok(metadata)
    }

    fn extract_tag_data(tag: &Tag, metadata: &mut SongMetadata) {
        // Use Accessor trait for common fields
        if let Some(artist) = tag.artist() {
            metadata.artist = artist.to_string();
        }
        if let Some(title) = tag.title() {
            metadata.title = title.to_string();
        }
        if let Some(album) = tag.album() {
            metadata.album = album.to_string();
        }
        if let Some(genre) = tag.genre() {
            metadata.genre = genre.to_string();
        }
        if let Some(track) = tag.track() {
            metadata.track_number = track.to_string();
        }
        if let Some(comment) = tag.comment() {
            metadata.comment = comment.to_string();
        }

        // Use ItemKey for other fields
        for item in tag.items() {
            let value = item.value().text().unwrap_or("");
            match item.key() {
                ItemKey::TrackTitle => metadata.title = value.to_string(),
                ItemKey::TrackArtist => metadata.artist = value.to_string(),
                ItemKey::AlbumTitle => metadata.album = value.to_string(),
                ItemKey::AlbumArtist => metadata.album_artist = value.to_string(),
                ItemKey::RecordingDate => metadata.year = value.to_string(),
                ItemKey::Year => metadata.year = value.to_string(),
                ItemKey::Genre => metadata.genre = value.to_string(),
                ItemKey::TrackNumber => metadata.track_number = value.to_string(),
                ItemKey::TrackTotal => metadata.total_tracks = value.to_string(),
                ItemKey::DiscNumber => metadata.disc_number = value.to_string(),
                ItemKey::DiscTotal => metadata.total_discs = value.to_string(),
                ItemKey::Composer => metadata.composer = value.to_string(),
                ItemKey::Lyricist => metadata.lyricist = value.to_string(),
                ItemKey::Conductor => metadata.conductor = value.to_string(),
                ItemKey::Remixer => metadata.remixer = value.to_string(),
                ItemKey::Producer => metadata.producer = value.to_string(),
                ItemKey::Arranger => metadata.arranger = value.to_string(),
                ItemKey::Engineer => metadata.engineer = value.to_string(),
                ItemKey::MixEngineer => metadata.mixer = value.to_string(),
                ItemKey::Comment => metadata.comment = value.to_string(),
                ItemKey::UnsyncLyrics => metadata.lyrics = value.to_string(),
                ItemKey::IntegerBpm => metadata.bpm = value.to_string(),
                ItemKey::Isrc => metadata.isrc = value.to_string(),
                ItemKey::MusicBrainzRecordingId => {
                    metadata.musicbrainz_track_id = value.to_string()
                }
                ItemKey::MusicBrainzTrackId => metadata.musicbrainz_track_id = value.to_string(),
                ItemKey::MusicBrainzArtistId => metadata.musicbrainz_artist_id = value.to_string(),
                ItemKey::MusicBrainzReleaseId => metadata.musicbrainz_album_id = value.to_string(),
                ItemKey::MusicBrainzReleaseArtistId => {
                    metadata.musicbrainz_album_artist_id = value.to_string()
                }
                ItemKey::MusicBrainzReleaseGroupId => {
                    metadata.musicbrainz_release_group_id = value.to_string()
                }
                ItemKey::MusicBrainzWorkId => metadata.musicbrainz_work_id = value.to_string(),
                ItemKey::AcoustIdFingerprint => metadata.acoustid_fingerprint = value.to_string(),
                ItemKey::AcoustId => metadata.acoustid_id = value.to_string(),
                ItemKey::ReplayGainTrackGain => metadata.replaygain_track_gain = value.to_string(),
                ItemKey::ReplayGainTrackPeak => metadata.replaygain_track_peak = value.to_string(),
                ItemKey::ReplayGainAlbumGain => metadata.replaygain_album_gain = value.to_string(),
                ItemKey::ReplayGainAlbumPeak => metadata.replaygain_album_peak = value.to_string(),
                _ => {}
            }
        }
    }

    fn extract_artwork(tag: &Tag, metadata: &mut SongMetadata) {
        for picture in tag.pictures() {
            metadata.album_art = Some(picture.data().to_vec());
            if let Some(mime) = picture.mime_type() {
                metadata.album_art_mime = Some(mime.to_string());
            }
            break;
        }
    }

    pub fn write_metadata(song: &mut Song) -> Result<()> {
        let tagged_file = Probe::open(&song.path)
            .map_err(|e| AppError::AudioMetadataParse(e.to_string()))?
            .read()
            .map_err(|e| AppError::AudioMetadataParse(e.to_string()))?;
        let mut tagged_file = tagged_file;

        let primary_tag_type = tagged_file.primary_tag_type();
        let tag = match tagged_file.tag_mut(primary_tag_type) {
            Some(tag) => tag,
            None => tagged_file
                .first_tag_mut()
                .ok_or_else(|| AppError::InvalidMetadata("No writable tag found".to_string()))?,
        };

        Self::write_tag_data(tag, &song.metadata);

        let replace_artwork = song.metadata.album_art != song.original_metadata.album_art;
        if replace_artwork {
            // A cleared artwork field must also remove the embedded cover.
            let picture_count = tag.pictures().len();
            for _ in 0..picture_count {
                tag.remove_picture(0);
            }
        }

        if let Some(art_data) = &song.metadata.album_art {
            let mime = song
                .metadata
                .album_art_mime
                .as_deref()
                .unwrap_or("image/jpeg");
            let mime_type = match mime {
                "image/png" => MimeType::Png,
                "image/gif" => MimeType::Gif,
                "image/bmp" => MimeType::Bmp,
                _ => MimeType::Jpeg,
            };
            let picture = Picture::unchecked(art_data.clone())
                .pic_type(PictureType::CoverFront)
                .mime_type(mime_type)
                .build();
            tag.push_picture(picture);
        }

        tagged_file
            .save_to_path(&song.path, Default::default())
            .map_err(|e| AppError::AudioMetadataEncode(e.to_string()))?;
        song.save_changes();
        Ok(())
    }

    fn write_tag_data(tag: &mut Tag, metadata: &SongMetadata) {
        macro_rules! set_if_not_empty {
            ($tag:expr, $key:expr, $value:expr) => {
                if !$value.is_empty() {
                    $tag.insert_text($key, $value.clone());
                } else {
                    $tag.remove_key($key);
                }
            };
        }

        set_if_not_empty!(tag, ItemKey::TrackTitle, metadata.title);
        set_if_not_empty!(tag, ItemKey::TrackArtist, metadata.artist);
        set_if_not_empty!(tag, ItemKey::AlbumTitle, metadata.album);
        set_if_not_empty!(tag, ItemKey::AlbumArtist, metadata.album_artist);
        set_if_not_empty!(tag, ItemKey::RecordingDate, metadata.year);
        set_if_not_empty!(tag, ItemKey::Year, metadata.year);
        set_if_not_empty!(tag, ItemKey::Genre, metadata.genre);
        set_if_not_empty!(tag, ItemKey::TrackNumber, metadata.track_number);
        set_if_not_empty!(tag, ItemKey::TrackTotal, metadata.total_tracks);
        set_if_not_empty!(tag, ItemKey::DiscNumber, metadata.disc_number);
        set_if_not_empty!(tag, ItemKey::DiscTotal, metadata.total_discs);
        set_if_not_empty!(tag, ItemKey::Composer, metadata.composer);
        set_if_not_empty!(tag, ItemKey::Lyricist, metadata.lyricist);
        set_if_not_empty!(tag, ItemKey::Conductor, metadata.conductor);
        set_if_not_empty!(tag, ItemKey::Remixer, metadata.remixer);
        set_if_not_empty!(tag, ItemKey::Producer, metadata.producer);
        set_if_not_empty!(tag, ItemKey::Arranger, metadata.arranger);
        set_if_not_empty!(tag, ItemKey::Engineer, metadata.engineer);
        set_if_not_empty!(tag, ItemKey::MixEngineer, metadata.mixer);
        set_if_not_empty!(tag, ItemKey::Comment, metadata.comment);
        set_if_not_empty!(tag, ItemKey::UnsyncLyrics, metadata.lyrics);
        set_if_not_empty!(tag, ItemKey::IntegerBpm, metadata.bpm);
        set_if_not_empty!(tag, ItemKey::Isrc, metadata.isrc);
        set_if_not_empty!(
            tag,
            ItemKey::MusicBrainzRecordingId,
            metadata.musicbrainz_track_id
        );
        set_if_not_empty!(
            tag,
            ItemKey::MusicBrainzTrackId,
            metadata.musicbrainz_track_id
        );
        set_if_not_empty!(
            tag,
            ItemKey::MusicBrainzArtistId,
            metadata.musicbrainz_artist_id
        );
        set_if_not_empty!(
            tag,
            ItemKey::MusicBrainzReleaseId,
            metadata.musicbrainz_album_id
        );
        set_if_not_empty!(
            tag,
            ItemKey::MusicBrainzReleaseArtistId,
            metadata.musicbrainz_album_artist_id
        );
        set_if_not_empty!(
            tag,
            ItemKey::MusicBrainzReleaseGroupId,
            metadata.musicbrainz_release_group_id
        );
        set_if_not_empty!(
            tag,
            ItemKey::MusicBrainzWorkId,
            metadata.musicbrainz_work_id
        );
        set_if_not_empty!(
            tag,
            ItemKey::AcoustIdFingerprint,
            metadata.acoustid_fingerprint
        );
        set_if_not_empty!(tag, ItemKey::AcoustId, metadata.acoustid_id);
        set_if_not_empty!(
            tag,
            ItemKey::ReplayGainTrackGain,
            metadata.replaygain_track_gain
        );
        set_if_not_empty!(
            tag,
            ItemKey::ReplayGainTrackPeak,
            metadata.replaygain_track_peak
        );
        set_if_not_empty!(
            tag,
            ItemKey::ReplayGainAlbumGain,
            metadata.replaygain_album_gain
        );
        set_if_not_empty!(
            tag,
            ItemKey::ReplayGainAlbumPeak,
            metadata.replaygain_album_peak
        );
    }

    pub fn load_song(path: &Path) -> Result<Song> {
        let metadata = Self::read_metadata(path)?;
        Ok(Song::new(path.to_path_buf(), metadata))
    }

    #[allow(dead_code)]
    pub fn load_songs(paths: &[impl AsRef<Path>]) -> Vec<Result<Song>> {
        paths.iter().map(|p| Self::load_song(p.as_ref())).collect()
    }
}
