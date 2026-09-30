use crate::models::{AppError, LLMSettings, Result, SongMetadata};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::time::Duration;
use tokio::runtime::Handle;

static TOKIO_HANDLE: OnceLock<Handle> = OnceLock::new();

pub fn set_tokio_handle(handle: Handle) {
    let _ = TOKIO_HANDLE.set(handle);
}

pub fn get_tokio_handle() -> Option<&'static Handle> {
    TOKIO_HANDLE.get()
}

#[derive(Debug, Serialize)]
struct LlmRequest {
    model: String,
    messages: Vec<LlmMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Serialize)]
struct LlmMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct LlmResponse {
    choices: Vec<LlmChoice>,
}

#[derive(Debug, Deserialize)]
struct LlmChoice {
    message: LlmResponseMessage,
}

#[derive(Debug, Deserialize)]
struct LlmResponseMessage {
    content: String,
}

#[derive(Clone)]
pub struct LlmService {
    client: Client,
}

impl LlmService {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .unwrap(),
        }
    }

    pub async fn fetch_metadata(
        &self,
        settings: &LLMSettings,
        title: &str,
        artist: &str,
    ) -> Result<SongMetadata> {
        if !settings.is_configured() {
            return Err(AppError::Llm("LLM not configured".to_string()));
        }

        let prompt = self.build_prompt(title, artist);
        let response: String = self.call_llm(settings, &prompt).await?;
        self.parse_response(&response)
    }

    fn build_prompt(&self, title: &str, artist: &str) -> String {
        format!(
            r#"You are a music metadata expert. Given a song title and artist, provide comprehensive metadata in JSON format.
Return ONLY a valid JSON object with these fields (use empty string "" for unknown values):
{{
  "title": "",
  "artist": "",
  "album": "",
  "album_artist": "",
  "year": "",
  "genre": "",
  "track_number": "",
  "total_tracks": "",
  "disc_number": "",
  "total_discs": "",
  "composer": "",
  "lyricist": "",
  "conductor": "",
  "remixer": "",
  "producer": "",
  "arranger": "",
  "engineer": "",
  "mixer": "",
  "comment": "",
  "lyrics": "",
  "bpm": "",
  "isrc": "",
  "acoustid_fingerprint": "",
  "acoustid_id": "",
  "replaygain_track_gain": "",
  "replaygain_track_peak": "",
  "replaygain_album_gain": "",
  "replaygain_album_peak": ""
}}

Song: "{title}"
Artist: "{artist}"

Provide accurate metadata. If you cannot find the song, return empty strings for all fields."#
        )
    }

    async fn call_llm(&self, settings: &LLMSettings, prompt: &str) -> Result<String> {
        let base_url = settings
            .base_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1");
        let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

        let request = LlmRequest {
            model: settings.model_name.clone(),
            messages: vec![
                LlmMessage {
                    role: "system".to_string(),
                    content: "You are a music metadata expert. Return only valid JSON.".to_string(),
                },
                LlmMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            temperature: 0.1,
            max_tokens: 2000,
        };

        let mut req_builder = self.client.post(&url).json(&request);

        if !settings.api_key.is_empty() {
            req_builder = req_builder.bearer_auth(&settings.api_key);
        }

        let response = req_builder.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::Llm(format!(
                "LLM API error ({}): {}",
                status, error_text
            )));
        }

        let llm_response: LlmResponse = response.json().await?;

        Ok(llm_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default())
    }

    fn parse_response(&self, response: &str) -> Result<SongMetadata> {
        let json_start = response.find('{').unwrap_or(0);
        let json_end = response.rfind('}').map(|i| i + 1).unwrap_or(response.len());
        let json_str = &response[json_start..json_end];

        let metadata: SongMetadata = serde_json::from_str(json_str)
            .map_err(|e| AppError::Llm(format!("Failed to parse LLM response: {}", e)))?;

        if metadata.is_empty() {
            return Err(AppError::SongNotFound(
                "Song not found in LLM database".to_string(),
            ));
        }

        Ok(metadata)
    }

    pub async fn test_connection(&self, settings: &LLMSettings) -> Result<()> {
        if !settings.is_configured() {
            return Err(AppError::Llm("LLM not configured".to_string()));
        }

        let prompt = "Test connection. Reply with 'OK'.";
        let _ = self.call_llm(settings, prompt).await?;
        Ok(())
    }
}

impl Default for LlmService {
    fn default() -> Self {
        Self::new()
    }
}
