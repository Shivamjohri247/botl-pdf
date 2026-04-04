use crate::error::{BotlError, Result};

/// Decode RunLength encoded data.
pub fn decode(data: &[u8]) -> Result<Vec<u8>> {
    let mut output = Vec::with_capacity(data.len());
    let mut i = 0;

    while i < data.len() {
        let length_byte = data[i] as i16;
        i += 1;

        if length_byte == 128 {
            break; // EOD
        }

        if length_byte >= 0 && length_byte <= 127 {
            let count = (length_byte + 1) as usize;
            if i + count > data.len() {
                return Err(BotlError::CodecError(
                    "Unexpected end of RunLength data".into(),
                ));
            }
            output.extend_from_slice(&data[i..i + count]);
            i += count;
        } else if length_byte >= 129 {
            let count = (257 - length_byte as i16) as usize;
            if i >= data.len() {
                return Err(BotlError::CodecError(
                    "Unexpected end of RunLength data".into(),
                ));
            }
            output.extend(std::iter::repeat(data[i]).take(count));
            i += 1;
        }
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_run() {
        let data = vec![2, b'a', b'b', b'c', 128];
        assert_eq!(decode(&data).unwrap(), b"abc");
    }

    #[test]
    fn test_repeated_run() {
        let data = vec![255, b'x', 128];
        assert_eq!(decode(&data).unwrap(), b"xx");
    }

    #[test]
    fn test_mixed() {
        let data = vec![1, b'a', b'b', 254, b'c', 128];
        assert_eq!(decode(&data).unwrap(), b"abccc");
    }
}
