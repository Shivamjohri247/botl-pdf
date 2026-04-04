use crate::error::{BotlError, Result};

/// Decode ASCIIHex encoded data.
pub fn decode(data: &[u8]) -> Result<Vec<u8>> {
    let mut output = Vec::with_capacity(data.len() / 2);
    let mut high_nibble: Option<u8> = None;

    for &b in data {
        if b == b'>' {
            if let Some(h) = high_nibble {
                output.push(h << 4);
            }
            break;
        }

        let nibble = match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'f' => b - b'a' + 10,
            b'A'..=b'F' => b - b'A' + 10,
            b' ' | b'\t' | b'\r' | b'\n' | b'\x0c' => continue,
            _ => {
                return Err(BotlError::CodecError(format!(
                    "Invalid hex char: '{}'",
                    b as char
                )))
            }
        };

        match high_nibble {
            Some(h) => {
                output.push((h << 4) | nibble);
                high_nibble = None;
            }
            None => high_nibble = Some(nibble),
        }
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_basic() {
        assert_eq!(decode(b"48656C6C6F>").unwrap(), b"Hello");
    }

    #[test]
    fn test_decode_with_spaces() {
        assert_eq!(decode(b"48 65 6C 6C 6F>").unwrap(), b"Hello");
    }

    #[test]
    fn test_decode_odd_length() {
        assert_eq!(decode(b"4>").unwrap(), &[0x40]);
    }
}
