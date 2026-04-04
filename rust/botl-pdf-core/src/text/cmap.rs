use crate::error::{BotlError, Result};
use hashbrown::HashMap;

/// A CMap (Character Map) maps character codes to Unicode code points.
/// Used for text extraction from PDF content streams.
#[derive(Debug, Clone)]
pub struct CMap {
    /// Single-character mappings: code → Unicode
    bf_char: HashMap<u32, u32>,
    /// Range mappings: start_code → (end_code, start_unicode)
    bf_range: Vec<(u32, u32, u32)>,
}

impl CMap {
    pub fn new() -> Self {
        Self {
            bf_char: HashMap::new(),
            bf_range: Vec::new(),
        }
    }

    /// Parse a ToUnicode CMap from raw bytes.
    /// Handles beginbfchar/endbfchar and beginbfrange/endbfrange sections.
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut cmap = Self::new();
        let text = std::str::from_utf8(data)
            .map_err(|_| BotlError::CMapError("CMap data is not valid UTF-8".into()))?;

        // Parse beginbfchar ... endbfchar sections
        if let Some(start) = text.find("beginbfchar") {
            let section = &text[start + "beginbfchar".len()..];
            if let Some(end) = section.find("endbfchar") {
                let section = &section[..end];
                for line in section.lines() {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    // Format: <src_hex> <dst_hex>
                    if let Some((src, dst)) = parse_bf_char_line(line) {
                        cmap.bf_char.insert(src, dst);
                    }
                }
            }
        }

        // Parse beginbfrange ... endbfrange sections
        if let Some(start) = text.find("beginbfrange") {
            let section = &text[start + "beginbfrange".len()..];
            if let Some(end) = section.find("endbfrange") {
                let section = &section[..end];
                for line in section.lines() {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    // Format: <start_hex> <end_hex> <dst_start_hex>
                    if let Some((start_code, end_code, dst_start)) = parse_bf_range_line(line) {
                        cmap.bf_range.push((start_code, end_code, dst_start));
                    }
                }
            }
        }

        Ok(cmap)
    }

    /// Map a character code to a Unicode code point.
    pub fn to_unicode(&self, code: u32) -> Option<u32> {
        // Check single-char mappings first
        if let Some(&unicode) = self.bf_char.get(&code) {
            return Some(unicode);
        }

        // Check range mappings
        for &(start, end, dst_start) in &self.bf_range {
            if code >= start && code <= end {
                return Some(dst_start + (code - start));
            }
        }

        None
    }

    /// Map a character code to a Unicode char.
    pub fn to_char(&self, code: u32) -> Option<char> {
        self.to_unicode(code).and_then(|u| char::from_u32(u))
    }

    /// Decode a sequence of bytes using this CMap.
    /// Returns the decoded Unicode string.
    pub fn decode(&self, bytes: &[u8]) -> String {
        let mut result = String::new();
        let mut i = 0;
        while i < bytes.len() {
            // Try multi-byte codes first (2 bytes), then fall back to 1 byte
            if i + 1 < bytes.len() {
                let code2 = ((bytes[i] as u32) << 8) | (bytes[i + 1] as u32);
                if let Some(ch) = self.to_char(code2) {
                    result.push(ch);
                    i += 2;
                    continue;
                }
            }
            let code1 = bytes[i] as u32;
            if let Some(ch) = self.to_char(code1) {
                result.push(ch);
            } else {
                // Fallback: use the byte as-is if it's printable ASCII
                if bytes[i].is_ascii_graphic() || bytes[i] == b' ' {
                    result.push(bytes[i] as char);
                }
            }
            i += 1;
        }
        result
    }
}

impl Default for CMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a bfchar line: `<src> <dst>`
fn parse_bf_char_line(line: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }
    let src = parse_hex_in_angle_brackets(parts[0])?;
    let dst = parse_hex_in_angle_brackets(parts[1])?;
    Some((src, dst))
}

/// Parse a bfrange line: `<start> <end> <dst_start>`
fn parse_bf_range_line(line: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }
    let start = parse_hex_in_angle_brackets(parts[0])?;
    let end = parse_hex_in_angle_brackets(parts[1])?;
    let dst = parse_hex_in_angle_brackets(parts[2])?;
    Some((start, end, dst))
}

/// Parse a hex value from angle brackets: `<0048>` → 0x48
fn parse_hex_in_angle_brackets(s: &str) -> Option<u32> {
    let s = s.trim();
    if s.starts_with('<') && s.ends_with('>') {
        let hex = &s[1..s.len() - 1];
        u32::from_str_radix(hex, 16).ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bfchar() {
        let data = br#"
/CIDInit /ProcSet findresource begin
12 dict begin
begincmap
/CIDSystemInfo
<< /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def
/CMapName /Adobe-Identity-UCS def
/CMapType 2 def
1 begincodespacerange
<00> <FF>
endcodespacerange
3 beginbfchar
<41> <0041>
<42> <0042>
<43> <0043>
endbfchar
endcmap
"#;
        let cmap = CMap::parse(data).unwrap();
        assert_eq!(cmap.to_char(0x41), Some('A'));
        assert_eq!(cmap.to_char(0x42), Some('B'));
        assert_eq!(cmap.to_char(0x43), Some('C'));
    }

    #[test]
    fn test_parse_bfrange() {
        let data = br#"
1 begincodespacerange
<00> <FF>
endcodespacerange
1 beginbfrange
<41> <5A> <0041>
endbfrange
"#;
        let cmap = CMap::parse(data).unwrap();
        assert_eq!(cmap.to_char(0x41), Some('A'));
        assert_eq!(cmap.to_char(0x5A), Some('Z'));
        assert_eq!(cmap.to_char(0x50), Some('P'));
    }

    #[test]
    fn test_decode_bytes() {
        let data = br#"
1 beginbfchar
<48> <0048>
<65> <0065>
<6C> <006C>
<6F> <006F>
endbfchar
"#;
        let cmap = CMap::parse(data).unwrap();
        assert_eq!(cmap.decode(b"Hello"), "Hello");
    }
}
