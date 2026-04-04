use crate::error::{BotlError, Result};

/// Decode ASCII85 (Base85) encoded data.
///
/// Groups of 5 ASCII characters (33-117) encode 4 binary bytes.
/// `z` is a special case representing four zero bytes.
pub fn decode(data: &[u8]) -> Result<Vec<u8>> {
    let mut output = Vec::with_capacity(data.len() * 4 / 5);
    let mut group = [0u32; 5];
    let mut group_len = 0;

    let mut i = 0;
    while i < data.len() {
        let b = data[i];

        if matches!(b, b' ' | b'\t' | b'\r' | b'\n' | b'\x0c') {
            i += 1;
            continue;
        }

        if b == b'~' && i + 1 < data.len() && data[i + 1] == b'>' {
            if group_len > 0 {
                let decoded = decode_group(&group[..group_len])?;
                output.extend_from_slice(&decoded);
            }
            break;
        }

        if b == b'z' {
            if group_len != 0 {
                return Err(BotlError::CodecError(
                    "'z' in middle of ASCII85 group".into(),
                ));
            }
            output.extend_from_slice(&[0, 0, 0, 0]);
            i += 1;
            continue;
        }

        if !(33..=117).contains(&b) {
            return Err(BotlError::CodecError(format!(
                "Invalid ASCII85 char: {}",
                b
            )));
        }

        group[group_len] = (b - 33) as u32;
        group_len += 1;

        if group_len == 5 {
            let decoded = decode_full_group(group)?;
            output.extend_from_slice(&decoded);
            group_len = 0;
        }

        i += 1;
    }

    Ok(output)
}

fn decode_full_group(group: [u32; 5]) -> Result<[u8; 4]> {
    let value = group[0] * 85u32.pow(4)
        + group[1] * 85u32.pow(3)
        + group[2] * 85u32.pow(2)
        + group[3] * 85
        + group[4];
    Ok([
        (value >> 24) as u8,
        (value >> 16) as u8,
        (value >> 8) as u8,
        value as u8,
    ])
}

fn decode_group(group: &[u32]) -> Result<Vec<u8>> {
    let n = group.len();
    if n == 1 {
        return Err(BotlError::CodecError("Incomplete ASCII85 group".into()));
    }
    let mut padded = [84u32; 5];
    for (i, &v) in group.iter().enumerate() {
        padded[i] = v;
    }
    let full = decode_full_group(padded)?;
    Ok(full[..n - 1].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_z() {
        assert_eq!(decode(b"z").unwrap(), &[0, 0, 0, 0]);
    }

    #[test]
    fn test_decode_man() {
        // "Man " in ASCII85
        let decoded = decode(b"9jqo^").unwrap();
        assert_eq!(decoded, b"Man ");
    }
}
