//! JPXDecode (JPEG2000) decompression via the csys crate.

use crate::error::{BotlError, Result};

/// Decompress JPEG2000 data from a PDF JPXDecode stream.
pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    let image = botl_pdf_csys::jpx::decode_jpeg2000(data)
        .map_err(|e| BotlError::CodecError(format!("JPEG2000 decompression failed: {}", e)))?;

    Ok(image.data)
}
