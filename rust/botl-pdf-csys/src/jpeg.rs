//! JPEG (DCTDecode) decompression.
//!
//! Uses the `jpeg-decoder` crate for reliable JPEG decompression.

use crate::image::DecodedImage;

/// Errors that can occur during JPEG decompression.
#[derive(Debug)]
pub enum JpegError {
    /// The input data is not valid JPEG.
    InvalidData(String),
    /// An I/O error occurred.
    Io(String),
}

impl std::fmt::Display for JpegError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JpegError::InvalidData(msg) => write!(f, "Invalid JPEG data: {}", msg),
            JpegError::Io(msg) => write!(f, "JPEG I/O error: {}", msg),
        }
    }
}

impl std::error::Error for JpegError {}

/// Decompress JPEG data from a PDF DCTDecode stream.
///
/// Returns the decoded pixel data along with image dimensions and color space info.
pub fn decode_jpeg(data: &[u8]) -> Result<DecodedImage, JpegError> {
    let mut decoder = jpeg_decoder::Decoder::new(data);
    let pixels = decoder.decode().map_err(|e| match e {
        jpeg_decoder::Error::Format(msg) => JpegError::InvalidData(msg.to_string()),
        jpeg_decoder::Error::Io(msg) => JpegError::Io(msg.to_string()),
        jpeg_decoder::Error::Internal(msg) => JpegError::InvalidData(msg.to_string()),
        _ => JpegError::InvalidData("Unknown JPEG decoding error".into()),
    })?;

    let info = decoder
        .info()
        .ok_or_else(|| JpegError::InvalidData("No JPEG header information found".into()))?;

    let components = match info.pixel_format {
        jpeg_decoder::PixelFormat::L8 => 1,
        jpeg_decoder::PixelFormat::RGB24 => 3,
        jpeg_decoder::PixelFormat::CMYK32 => 4,
        _ => 3, // fallback for unknown formats
    };

    Ok(DecodedImage::new(
        pixels,
        info.width.into(),
        info.height.into(),
        components,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_invalid_jpeg_returns_error() {
        let result = decode_jpeg(b"not a jpeg");
        assert!(result.is_err());
        match result.unwrap_err() {
            JpegError::InvalidData(_) => {} // expected
            other => panic!("Expected InvalidData, got {:?}", other),
        }
    }

    #[test]
    fn test_decode_empty_data_returns_error() {
        let result = decode_jpeg(&[]);
        assert!(result.is_err());
    }

    /// Minimal valid JPEG: SOI + EOI markers only.
    /// This is a degenerate JPEG but should parse without error at the header level.
    #[test]
    fn test_decode_minimal_jpeg() {
        // SOI (Start of Image) + APP0 marker + EOI (End of Image)
        // A minimal JPEG needs at least a frame header
        let minimal = [
            0xFF, 0xD8, // SOI
            0xFF, 0xE0, // APP0
            0x00, 0x10, // Length
            b'J', b'F', b'I', b'F', 0x00, // JFIF identifier
            0x01, 0x01, // Version 1.1
            0x00, // Aspect ratio units
            0x00, 0x01, // X density
            0x00, 0x01, // Y density
            0x00, 0x00, // Thumbnail dimensions
            // No actual image data — this should fail on decode
        ];
        let result = decode_jpeg(&minimal);
        // Should fail because there's no image data, but parsing starts correctly
        assert!(result.is_err());
    }
}
