use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Audio metadata parse error: {0}")]
    AudioMetadataParse(String),

    #[error("Audio metadata encode error: {0}")]
    AudioMetadataEncode(String),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),

    #[error("LLM error: {0}")]
    Llm(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[allow(dead_code)]
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[allow(dead_code)]
    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),

    #[allow(dead_code)]
    #[error("No internet connection")]
    NoInternet,

    #[error("Song not found: {0}")]
    SongNotFound(String),

    #[error("Invalid metadata: {0}")]
    InvalidMetadata(String),
}

pub type Result<T> = std::result::Result<T, AppError>;