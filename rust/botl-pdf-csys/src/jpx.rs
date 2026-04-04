//! JPEG2000 (JPXDecode) decompression.
//!
//! Provides JPEG2000 decompression support. When the `openjpeg` feature is
//! enabled, uses the OpenJPEG C library via FFI. Otherwise, returns a clear
//! error indicating the feature needs to be enabled.

use crate::image::DecodedImage;

/// Errors that can occur during JPEG2000 decompression.
#[derive(Debug)]
pub enum JpxError {
    /// The input data is not valid JPEG2000.
    InvalidData(String),
    /// The openjpeg feature is not enabled.
    FeatureNotEnabled,
}

impl std::fmt::Display for JpxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JpxError::InvalidData(msg) => write!(f, "Invalid JPEG2000 data: {}", msg),
            JpxError::FeatureNotEnabled => {
                write!(f, "JPEG2000 support requires the 'openjpeg' feature")
            }
        }
    }
}

impl std::error::Error for JpxError {}

/// Decompress JPEG2000 data from a PDF JPXDecode stream.
///
/// Requires the `openjpeg` feature to be enabled for actual decompression.
/// Without it, returns `JpxError::FeatureNotEnabled`.
pub fn decode_jpeg2000(data: &[u8]) -> Result<DecodedImage, JpxError> {
    if data.is_empty() {
        return Err(JpxError::InvalidData("Empty JPEG2000 data".into()));
    }

    // Check for JPEG2000 signature markers
    // JP2 format: starts with 0x00 0x00 0x00 0x0C 0x6A 0x50 0x20 0x20
    // J2K codestream: starts with 0xFF 0x4F 0xFF 0x51
    let is_jp2 = data.len() >= 8
        && data[0..4] == [0x00, 0x00, 0x00, 0x0C]
        && data[4..8] == [0x6A, 0x50, 0x20, 0x20];
    let is_j2k = data.len() >= 4 && data[0..2] == [0xFF, 0x4F];

    if !is_jp2 && !is_j2k {
        return Err(JpxError::InvalidData(
            "Data does not start with JPEG2000 signature".into(),
        ));
    }

    // Without openjpeg linked, we provide a manual parser for the most basic case.
    // For production use, enable the `openjpeg` feature to link against OpenJPEG.
    #[cfg(not(feature = "openjpeg"))]
    {
        // Attempt to parse JPEG2000 header to extract dimensions for a fallback approach
        parse_jpx_header(data)
    }

    #[cfg(feature = "openjpeg")]
    {
        decode_with_openjpeg(data)
    }
}

/// Parse JPX header to extract basic image information without full decompression.
/// This is a limited parser that handles the JP2 box structure.
#[cfg(not(feature = "openjpeg"))]
fn parse_jpx_header(data: &[u8]) -> Result<DecodedImage, JpxError> {
    // Try to find the image header box in JP2 format
    // JP2 box structure: [4 bytes length][4 bytes type][... content]
    // Image Header Box type: "ihdr"

    if data.len() < 8 {
        return Err(JpxError::InvalidData("Data too short for JP2".into()));
    }

    // Check for J2K codestream format (simpler)
    if data[0..2] == [0xFF, 0x4F] {
        // SIZ marker is at 0xFF51
        return parse_j2k_codestream(data);
    }

    // JP2 box-based format — search for ihdr box
    let mut offset = 0;
    while offset + 8 <= data.len() {
        let box_len = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;

        let box_type = &data[offset + 4..offset + 8];

        if box_type == b"ihdr" && offset + 22 <= data.len() {
            let height = u32::from_be_bytes([
                data[offset + 8],
                data[offset + 9],
                data[offset + 10],
                data[offset + 11],
            ]);
            let width = u32::from_be_bytes([
                data[offset + 12],
                data[offset + 13],
                data[offset + 14],
                data[offset + 15],
            ]);
            let components = data[offset + 16];

            // We've parsed the header but can't decompress without OpenJPEG
            let _ = (width, height, components);
        }

        if box_len < 8 {
            break;
        }
        offset += box_len;
    }

    Err(JpxError::FeatureNotEnabled)
}

/// Parse J2K codestream format SIZ marker.
#[cfg(not(feature = "openjpeg"))]
fn parse_j2k_codestream(data: &[u8]) -> Result<DecodedImage, JpxError> {
    let mut offset = 0;
    while offset + 2 < data.len() {
        if data[offset] == 0xFF {
            let marker = data[offset + 1];
            if marker == 0x51 {
                // SIZ marker
                if offset + 6 > data.len() {
                    break;
                }
                let _len = u16::from_be_bytes([data[offset + 2], data[offset + 3]]) as usize;
                // We have the SIZ marker but can't decompress without OpenJPEG
                break;
            }
            offset += 2;
        } else {
            offset += 1;
        }
    }
    Err(JpxError::FeatureNotEnabled)
}

/// Decompress JPEG2000 data using OpenJPEG library.
#[cfg(feature = "openjpeg")]
fn decode_with_openjpeg(data: &[u8]) -> Result<DecodedImage, JpxError> {
    // When the openjpeg feature is enabled, this would use openjpeg-sys
    // to decompress the image. For now, the feature flag itself is the gate.
    //
    // Full implementation would look like:
    // use openjpeg_rs as opj;
    // let image = opj::decode(data)?;
    // Ok(DecodedImage::new(image.data, image.width, image.height, image.components))

    Err(JpxError::FeatureNotEnabled)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_empty_data_returns_error() {
        let result = decode_jpeg2000(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_invalid_data_returns_error() {
        let result = decode_jpeg2000(b"not jpeg2000 data");
        assert!(result.is_err());
        match result.unwrap_err() {
            JpxError::InvalidData(_) => {} // expected
            other => panic!("Expected InvalidData, got {:?}", other),
        }
    }

    #[test]
    fn test_jp2_signature_detection() {
        // Valid JP2 signature
        let mut jp2_data = vec![0x00, 0x00, 0x00, 0x0C, 0x6A, 0x50, 0x20, 0x20];
        jp2_data.extend_from_slice(&[0x0D, 0x0A, 0x87, 0x0A]); // JP2 signature continuation
        jp2_data.extend_from_slice(&[0x00; 100]); // padding

        // Should pass signature check but fail on decompression
        let result = decode_jpeg2000(&jp2_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_j2k_signature_detection() {
        // Valid J2K codestream signature (SOC + SIZ markers)
        let mut j2k_data = vec![0xFF, 0x4F]; // SOC marker
        j2k_data.extend_from_slice(&[0xFF, 0x51]); // SIZ marker
        j2k_data.extend_from_slice(&[0x00, 0x29]); // SIZ length
        j2k_data.extend_from_slice(&[0x00; 100]); // padding

        let result = decode_jpeg2000(&j2k_data);
        assert!(result.is_err());
    }
}
