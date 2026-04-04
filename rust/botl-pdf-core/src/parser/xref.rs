use crate::error::{BotlError, Result};
use crate::parser::objects::{ObjRef, PdfDict};

/// An entry in the cross-reference table.
#[derive(Debug, Clone, Copy)]
pub enum XrefEntry {
    /// In-use entry: points to byte offset in the file.
    InUse { offset: u64, gen_num: u16 },
    /// Free entry: the object number of the next free object.
    Free { next_free: u32, gen_num: u16 },
    /// Compressed entry: stored in an object stream.
    Compressed {
        /// Object number of the containing object stream.
        obj_stream_num: u32,
        /// Index within the object stream.
        index: u32,
    },
}

/// The cross-reference table mapping object numbers to their locations.
#[derive(Debug, Clone)]
pub struct XrefTable {
    /// Map from object number to its xref entry.
    entries: hashbrown::HashMap<u32, XrefEntry>,
    /// The trailer dictionary.
    pub trailer: PdfDict,
}

impl XrefTable {
    pub fn new(trailer: PdfDict) -> Self {
        Self {
            entries: hashbrown::HashMap::new(),
            trailer,
        }
    }

    pub fn insert(&mut self, obj_num: u32, entry: XrefEntry) {
        self.entries.insert(obj_num, entry);
    }

    pub fn get(&self, obj_num: u32) -> Option<&XrefEntry> {
        self.entries.get(&obj_num)
    }

    pub fn contains(&self, obj_num: u32) -> bool {
        self.entries.contains_key(&obj_num)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Get the root catalog reference from the trailer.
    pub fn root(&self) -> Option<ObjRef> {
        self.trailer.get_reference("Root")
    }

    /// Get the info dictionary reference from the trailer.
    pub fn info(&self) -> Option<ObjRef> {
        self.trailer.get_reference("Info")
    }

    /// Get the encrypt dictionary reference from the trailer.
    pub fn encrypt(&self) -> Option<ObjRef> {
        self.trailer.get_reference("Encrypt")
    }

    /// Merge another xref table into this one (for incremental updates).
    /// Entries from `other` override entries in `self`.
    pub fn merge(&mut self, other: &XrefTable) {
        for (&obj_num, entry) in &other.entries {
            self.entries.insert(obj_num, *entry);
        }
        // Merge trailer keys (other wins)
        for (key, value) in other.trailer.iter() {
            self.trailer.insert(key.to_vec(), value.clone());
        }
    }
}

/// Parse a traditional xref table from the given bytes.
///
/// Format:
/// ```text
/// xref
/// 0 6
/// 0000000000 65535 f
/// 0000000015 00000 n
/// ...
/// ```
pub fn parse_xref_table(data: &[u8]) -> Result<XrefTable> {
    let mut pos = 0;

    // Skip "xref" keyword
    let keyword = skip_ws_and_read_word(data, &mut pos)?;
    if keyword != b"xref" {
        return Err(BotlError::XrefError("Expected 'xref' keyword".into()));
    }

    let mut entries = hashbrown::HashMap::new();
    let mut trailer = PdfDict::new();

    loop {
        let remaining = &data[pos..];
        let skipped = crate::parser::lexer::skip_ws(remaining);
        pos = data.len() - skipped.len();

        // Check for trailer keyword
        let remaining = &data[pos..];
        if remaining.starts_with(b"trailer") {
            pos += 7; // skip "trailer"
                      // Parse trailer dictionary
            let mut obj_parser = crate::parser::objects::ObjectParser::new(&data[pos..]);
            let trailer_obj = obj_parser.parse_object()?;
            trailer = trailer_obj
                .into_dict()
                .ok_or_else(|| BotlError::XrefError("Expected dictionary for trailer".into()))?;
            pos += obj_parser.pos;
            break;
        }

        // Parse subsection: start_obj count
        let start_obj = read_integer_at(data, &mut pos)?;
        let count = read_integer_at(data, &mut pos)?;

        // Parse `count` entries, each exactly 20 bytes: "oooooooooo ggggg n \r\n" or "oooooooooo ggggg n \n"
        for i in 0..count {
            let obj_num = start_obj + i;

            // Skip whitespace
            while pos < data.len() && matches!(data[pos], b' ' | b'\r' | b'\n' | b'\t') {
                pos += 1;
            }

            if pos + 20 > data.len() {
                return Err(BotlError::XrefError("Truncated xref entry".into()));
            }

            let entry_line = &data[pos..pos + 20];
            pos += 20;

            // Parse: offset(10) space gen(5) space type(1) space/eol(2)
            let offset_str = std::str::from_utf8(&entry_line[0..10])
                .map_err(|_| BotlError::XrefError("Invalid xref offset".into()))?;
            let gen_str = std::str::from_utf8(&entry_line[11..16])
                .map_err(|_| BotlError::XrefError("Invalid xref generation".into()))?;
            let entry_type = entry_line[17];

            let offset: u64 = offset_str
                .trim()
                .parse()
                .map_err(|_| BotlError::XrefError("Invalid offset number".into()))?;
            let gen_num: u16 = gen_str
                .trim()
                .parse()
                .map_err(|_| BotlError::XrefError("Invalid generation number".into()))?;

            let entry = match entry_type {
                b'n' => XrefEntry::InUse { offset, gen_num },
                b'f' => XrefEntry::Free {
                    next_free: offset as u32,
                    gen_num,
                },
                _ => {
                    return Err(BotlError::XrefError(format!(
                        "Invalid xref entry type: '{}'",
                        entry_type as char
                    )))
                }
            };

            entries.insert(obj_num as u32, entry);
        }
    }

    Ok(XrefTable { entries, trailer })
}

/// Find the startxref value by scanning backwards from the end of the file.
/// Returns the byte offset of the xref section.
pub fn find_startxref(data: &[u8]) -> Result<u64> {
    // Search backwards for "startxref"
    let search = b"startxref";
    let mut pos = data.len();
    while pos >= search.len() {
        pos -= 1;
        if &data[pos.saturating_sub(search.len() - 1)..=pos] == search {
            // Read the number after startxref
            let num_start = pos + 1;
            let remaining = skip_ws_bytes(&data[num_start..]);
            let num_end = remaining
                .iter()
                .position(|&b| !b.is_ascii_digit())
                .unwrap_or(remaining.len());
            let num_str = std::str::from_utf8(&remaining[..num_end])
                .map_err(|_| BotlError::XrefError("Invalid startxref value".into()))?;
            return num_str
                .trim()
                .parse()
                .map_err(|_| BotlError::XrefError("Invalid startxref number".into()));
        }
    }
    Err(BotlError::XrefError("Could not find startxref".into()))
}

/// Find the xref position and parse the cross-reference table.
pub fn parse_xref_from_data(data: &[u8]) -> Result<XrefTable> {
    let startxref = find_startxref(data)?;

    let xref_start = startxref as usize;
    if xref_start >= data.len() {
        return Err(BotlError::XrefError(
            "startxref points past end of file".into(),
        ));
    }

    // Check if it's a traditional xref table or an xref stream
    let remaining = crate::parser::lexer::skip_ws(&data[xref_start..]);
    if remaining.starts_with(b"xref") {
        parse_xref_table(&data[xref_start..])
    } else {
        // Must be an xref stream (PDF 1.5+)
        parse_xref_stream(data, xref_start)
    }
}

/// Parse an xref stream (PDF 1.5+ cross-reference stream).
fn parse_xref_stream(data: &[u8], offset: usize) -> Result<XrefTable> {
    let mut parser = crate::parser::objects::ObjectParser::new(&data[offset..]);
    let indirect = parser.parse_indirect_object()?;
    let stream = indirect
        .object
        .into_stream()
        .ok_or_else(|| BotlError::XrefError("Expected stream object for xref".into()))?;

    let size = stream
        .dict
        .get_integer("Size")
        .ok_or_else(|| BotlError::XrefError("Xref stream missing Size".into()))?
        as u32;

    // Get W array (field widths)
    let w_array = stream
        .dict
        .get_array("W")
        .ok_or_else(|| BotlError::XrefError("Xref stream missing W array".into()))?;
    let w: Vec<usize> = w_array
        .iter()
        .map(|o| o.as_integer().unwrap_or(0) as usize)
        .collect();
    if w.len() != 3 {
        return Err(BotlError::XrefError("W array must have 3 elements".into()));
    }

    let field_len: usize = w[0] + w[1] + w[2];
    if field_len == 0 {
        return Err(BotlError::XrefError("W array fields sum to zero".into()));
    }

    // Decode the stream data
    let decoded = crate::codecs::decode_stream_data(&stream)?;

    // Get optional Index array (default: [0, Size])
    let index_entries = if let Some(index_arr) = stream.dict.get_array("Index") {
        let nums: Vec<u32> = index_arr
            .iter()
            .map(|o| o.as_integer().unwrap_or(0) as u32)
            .collect();
        // Pairs of (start, count)
        let mut entries = Vec::new();
        let mut i = 0;
        while i + 1 < nums.len() {
            entries.push((nums[i], nums[i + 1]));
            i += 2;
        }
        entries
    } else {
        vec![(0, size)]
    };

    let mut entries = hashbrown::HashMap::new();
    let mut byte_pos = 0;

    for (start, count) in index_entries {
        for i in 0..count {
            if byte_pos + field_len > decoded.len() {
                break;
            }

            let obj_num = start + i;

            // Read field 1: type (default 1 if w[0] == 0)
            let field_type = if w[0] > 0 {
                read_big_endian(&decoded[byte_pos..byte_pos + w[0]]) as u8
            } else {
                1
            };
            byte_pos += w[0];

            // Read field 2
            let field2 = read_big_endian(&decoded[byte_pos..byte_pos + w[1]]);
            byte_pos += w[1];

            // Read field 3
            let field3 = read_big_endian(&decoded[byte_pos..byte_pos + w[2]]);
            byte_pos += w[2];

            let entry = match field_type {
                0 => XrefEntry::Free {
                    next_free: field2 as u32,
                    gen_num: field3 as u16,
                },
                1 => XrefEntry::InUse {
                    offset: field2,
                    gen_num: field3 as u16,
                },
                2 => XrefEntry::Compressed {
                    obj_stream_num: field2 as u32,
                    index: field3 as u32,
                },
                _ => continue,
            };

            entries.insert(obj_num, entry);
        }
    }

    // Extract trailer fields from stream dict
    let mut trailer = PdfDict::new();
    if let Some(root) = stream.dict.get_str("Root") {
        trailer.insert(b"Root".to_vec(), root.clone());
    }
    if let Some(size_obj) = stream.dict.get_str("Size") {
        trailer.insert(b"Size".to_vec(), size_obj.clone());
    }
    if let Some(prev) = stream.dict.get_str("Prev") {
        trailer.insert(b"Prev".to_vec(), prev.clone());
    }

    Ok(XrefTable { entries, trailer })
}

fn read_big_endian(data: &[u8]) -> u64 {
    let mut val = 0u64;
    for &b in data {
        val = (val << 8) | (b as u64);
    }
    val
}

fn skip_ws_and_read_word<'a>(data: &'a [u8], pos: &mut usize) -> Result<&'a [u8]> {
    while *pos < data.len() && matches!(data[*pos], b' ' | b'\r' | b'\n' | b'\t') {
        *pos += 1;
    }
    let start = *pos;
    while *pos < data.len() && data[*pos].is_ascii_alphabetic() {
        *pos += 1;
    }
    if *pos == start {
        return Err(BotlError::ParseError("Expected keyword".into()));
    }
    Ok(&data[start..*pos])
}

fn read_integer_at(data: &[u8], pos: &mut usize) -> Result<u32> {
    while *pos < data.len() && matches!(data[*pos], b' ' | b'\r' | b'\n' | b'\t') {
        *pos += 1;
    }
    let start = *pos;
    while *pos < data.len() && (data[*pos].is_ascii_digit() || data[*pos] == b'-') {
        *pos += 1;
    }
    let s = std::str::from_utf8(&data[start..*pos])
        .map_err(|_| BotlError::ParseError("Invalid integer".into()))?;
    s.trim()
        .parse()
        .map_err(|_| BotlError::ParseError("Invalid integer value".into()))
}

fn skip_ws_bytes(data: &[u8]) -> &[u8] {
    let len = data
        .iter()
        .position(|&b| !matches!(b, b' ' | b'\t' | b'\r' | b'\n' | b'\x0c'))
        .unwrap_or(data.len());
    &data[len..]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_startxref() {
        let data = b"blah blah\nstartxref\n1234\n%%EOF\n";
        assert_eq!(find_startxref(data).unwrap(), 1234);
    }

    #[test]
    fn test_parse_xref_table() {
        let xref_data = b"xref\n0 3\n\
            0000000000 65535 f \n\
            0000000009 00000 n \n\
            0000000058 00000 n \n\
            trailer\n\
            << /Size 3 /Root 1 0 R >>\n";
        let xref = parse_xref_table(xref_data).unwrap();
        assert_eq!(xref.len(), 3);
        assert!(matches!(xref.get(0), Some(XrefEntry::Free { .. })));
        assert!(matches!(
            xref.get(1),
            Some(XrefEntry::InUse { offset: 9, .. })
        ));
        assert!(matches!(
            xref.get(2),
            Some(XrefEntry::InUse { offset: 58, .. })
        ));
        assert_eq!(xref.root(), Some(ObjRef::new(1, 0)));
    }
}
