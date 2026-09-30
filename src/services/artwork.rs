use serde::Deserialize;
use std::time::Duration;

#[derive(Deserialize)]
struct ItunesSearchResponse {
    results: Vec<ItunesSearchResult>,
}

#[derive(Deserialize)]
struct ItunesSearchResult {
    #[serde(rename = "artworkUrl100")]
    artwork_url: Option<String>,
}

pub async fn fetch_cover_art(
    title: &str,
    artist: &str,
    album: &str,
) -> Result<(Vec<u8>, String), String> {
    let term = format!("{} {} {}", artist, album, title);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;
    let search: ItunesSearchResponse = client
        .get("https://itunes.apple.com/search")
        .query(&[("term", term.as_str()), ("entity", "song"), ("limit", "5")])
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let url = search
        .results
        .into_iter()
        .find_map(|result| result.artwork_url)
        .ok_or_else(|| "No cover art was found for this song".to_string())?
        .replace("100x100bb", "600x600bb");
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;
    let mime = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .filter(|value| value.starts_with("image/"))
        .unwrap_or("image/jpeg")
        .to_string();
    let bytes = response.bytes().await.map_err(|e| e.to_string())?.to_vec();
    image::load_from_memory(&bytes)
        .map_err(|_| "The cover art response was not a supported image".to_string())?;
    Ok((bytes, mime))
}
