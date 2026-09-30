pub mod errors;
pub mod song;

pub use errors::{AppError, Result};
pub use song::{AppSettings, LLMSettings, Song, SongMetadata, ThemePreference};
