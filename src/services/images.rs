use image::RgbaImage;
use std::io::Cursor;

/// Decode an image and scale it down to at most `max_side` pixels.
///
/// Album art is only ever displayed at 96px (library) or 320px (detail view),
/// so uploading the original 600–3000px image wastes CPU and GPU memory. The
/// decoder limits also reject decompression bombs coming from remote artwork.
pub fn decode_thumbnail(bytes: &[u8], max_side: u32) -> Option<RgbaImage> {
    let mut reader = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;

    let mut limits = image::Limits::default();
    limits.max_image_width = Some(4096);
    limits.max_image_height = Some(4096);
    limits.max_alloc = Some(256 * 1024 * 1024);
    reader.limits(limits);

    let image = reader.decode().ok()?;
    Some(image.thumbnail(max_side, max_side).to_rgba8())
}
