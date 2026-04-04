//! DCTDecode (JPEG) decompression via the csys crate.

use crate::error::{BotlError, Result};

/// Decompress JPEG data from a PDF DCTDecode stream.
pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    let image = botl_pdf_csys::jpeg::decode_jpeg(data)
        .map_err(|e| BotlError::CodecError(format!("JPEG decompression failed: {}", e)))?;

    // Convert from planar/interleaved to raw pixel data
    // PDF expects the raw decoded pixel bytes
    Ok(image.data)
}
