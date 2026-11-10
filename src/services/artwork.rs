use serde::Deserialize;
use std::time::Duration;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

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
    let client = crate::services::http::client();
    let search: ItunesSearchResponse = client
        .get("https://itunes.apple.com/search")
        .query(&[("term", term.as_str()), ("entity", "song"), ("limit", "5")])
        .timeout(REQUEST_TIMEOUT)
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
        .timeout(REQUEST_TIMEOUT)
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

    // Only inspect the header here. Fully decoding the image just to validate
    // it wasted CPU and memory; the texture loader decodes (and bounds) it once
    // when it is actually displayed.
    match image::guess_format(&bytes) {
        Ok(
            image::ImageFormat::Png
            | image::ImageFormat::Jpeg
            | image::ImageFormat::Gif
            | image::ImageFormat::Bmp,
        ) => {}
        _ => return Err("The cover art response was not a supported image".to_string()),
    }

    Ok((bytes, mime))
}
