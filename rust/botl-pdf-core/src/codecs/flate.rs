use crate::error::{BotlError, Result};
use flate2::read::ZlibDecoder;
use std::io::Read;

/// Decompress FlateDecode (zlib) data.
pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(data);
    let mut output = Vec::with_capacity(data.len() * 2);
    decoder
        .read_to_end(&mut output)
        .map_err(|e| BotlError::CodecError(format!("FlateDecode error: {}", e)))?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;

    #[test]
    fn test_roundtrip() {
        let original = b"Hello, World! This is a test of FlateDecode compression.";
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(original).unwrap();
        let compressed = encoder.finish().unwrap();

        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_empty() {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(b"").unwrap();
        let compressed = encoder.finish().unwrap();
        let decompressed = decompress(&compressed).unwrap();
        assert!(decompressed.is_empty());
    }
}
