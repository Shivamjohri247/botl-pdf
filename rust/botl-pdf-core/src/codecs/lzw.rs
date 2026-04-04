use crate::error::{BotlError, Result};
use crate::parser::objects::PdfDict;

/// Decode LZW encoded data.
pub fn decode(data: &[u8], params: Option<&PdfDict>) -> Result<Vec<u8>> {
    let early_change = params
        .and_then(|p| p.get_integer("EarlyChange"))
        .unwrap_or(1) as i32;

    let mut decoder = LzwDecoder::new(early_change);
    decoder.decode(data)
}

struct LzwDecoder {
    early_change: i32,
}

impl LzwDecoder {
    fn new(early_change: i32) -> Self {
        Self { early_change }
    }

    fn decode(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::with_capacity(data.len() * 2);

        let clear_code: u16 = 256;
        let eod_code: u16 = 257;
        let mut code_size: u32 = 9;
        let mut next_code: u16 = 258;

        // Initialize table with single-byte entries
        let mut table: Vec<Option<Vec<u8>>> = (0..258).map(|i| Some(vec![i as u8])).collect();
        table.resize(4096, None);

        let mut bit_buffer: u64 = 0;
        let mut bits_in_buffer: u32 = 0;
        let mut byte_pos: usize = 0;
        let mut prev_code: Option<u16> = None;

        loop {
            let code = self.read_bits(data, &mut byte_pos, &mut bit_buffer, &mut bits_in_buffer, code_size)?;

            if code == eod_code {
                break;
            }

            if code == clear_code {
                code_size = 9;
                next_code = 258;
                table = (0..258).map(|i| Some(vec![i as u8])).collect();
                table.resize(4096, None);
                prev_code = None;
                continue;
            }

            let entry = if (code as usize) < next_code as usize {
                table[code as usize].clone()
            } else if let Some(prev) = prev_code {
                if let Some(ref prev_entry) = table[prev as usize] {
                    let mut new_entry = prev_entry.clone();
                    if let Some(&first) = prev_entry.first() {
                        new_entry.push(first);
                    }
                    Some(new_entry)
                } else {
                    None
                }
            } else {
                return Err(BotlError::CodecError("Invalid LZW code".into()));
            };

            let entry = entry.ok_or_else(|| BotlError::CodecError("LZW code not in table".into()))?;
            output.extend_from_slice(&entry);

            if let Some(prev) = prev_code {
                if next_code < 4096 {
                    if let Some(ref prev_entry) = table[prev as usize] {
                        let mut new_entry = prev_entry.clone();
                        if let Some(&first) = entry.first() {
                            new_entry.push(first);
                        }
                        table[next_code as usize] = Some(new_entry);
                        next_code += 1;

                        let threshold = (1i32 << code_size) + self.early_change;
                        if next_code as i32 >= threshold && code_size < 12 {
                            code_size += 1;
                        }
                    }
                }
            }

            prev_code = Some(code);
        }

        Ok(output)
    }

    fn read_bits(
        &self,
        data: &[u8],
        byte_pos: &mut usize,
        bit_buffer: &mut u64,
        bits_in_buffer: &mut u32,
        code_size: u32,
    ) -> Result<u16> {
        while *bits_in_buffer < code_size {
            if *byte_pos >= data.len() {
                return Ok(257); // EOD
            }
            *bit_buffer |= (data[*byte_pos] as u64) << *bits_in_buffer;
            *bits_in_buffer += 8;
            *byte_pos += 1;
        }

        let mask = (1u64 << code_size) - 1;
        let code = (*bit_buffer & mask) as u16;
        *bit_buffer >>= code_size;
        *bits_in_buffer -= code_size;
        Ok(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lzw_clear_code() {
        let data = vec![0x80, 0x00]; // Clear code at 9 bits
        let result = decode(&data, None);
        assert!(result.is_ok());
    }
}
