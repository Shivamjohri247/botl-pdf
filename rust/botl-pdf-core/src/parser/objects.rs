use crate::error::{BotlError, Result};
use crate::parser::lexer;
use hashbrown::HashMap;

/// A PDF object reference: `7 0 R`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjRef {
    pub obj_num: u32,
    pub gen_num: u16,
}

impl ObjRef {
    pub fn new(obj_num: u32, gen_num: u16) -> Self {
        Self { obj_num, gen_num }
    }
}

/// PDF object types.
#[derive(Debug, Clone, PartialEq)]
pub enum PdfObject {
    Null,
    Boolean(bool),
    Integer(i64),
    Real(f64),
    String(Vec<u8>),
    Name(Vec<u8>),
    Array(Vec<PdfObject>),
    Dictionary(PdfDict),
    Stream(PdfStream),
    Reference(ObjRef),
}

impl PdfObject {
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            PdfObject::Integer(i) => Some(*i),
            PdfObject::Real(f) => Some(*f as i64),
            _ => None,
        }
    }

    pub fn as_real(&self) -> Option<f64> {
        match self {
            PdfObject::Real(f) => Some(*f),
            PdfObject::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn as_name(&self) -> Option<&[u8]> {
        match self {
            PdfObject::Name(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_name_str(&self) -> Option<&str> {
        self.as_name().and_then(|n| std::str::from_utf8(n).ok())
    }

    pub fn as_string(&self) -> Option<&[u8]> {
        match self {
            PdfObject::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_string_utf8(&self) -> Option<&str> {
        self.as_string().and_then(|s| std::str::from_utf8(s).ok())
    }

    pub fn as_array(&self) -> Option<&[PdfObject]> {
        match self {
            PdfObject::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_dict(&self) -> Option<&PdfDict> {
        match self {
            PdfObject::Dictionary(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_reference(&self) -> Option<ObjRef> {
        match self {
            PdfObject::Reference(r) => Some(*r),
            _ => None,
        }
    }

    pub fn as_stream(&self) -> Option<&PdfStream> {
        match self {
            PdfObject::Stream(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            PdfObject::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, PdfObject::Null)
    }

    pub fn into_dict(self) -> Option<PdfDict> {
        match self {
            PdfObject::Dictionary(d) => Some(d),
            _ => None,
        }
    }

    pub fn into_stream(self) -> Option<PdfStream> {
        match self {
            PdfObject::Stream(s) => Some(s),
            _ => None,
        }
    }
}

/// A PDF dictionary.
#[derive(Debug, Clone, PartialEq)]
pub struct PdfDict {
    entries: Vec<(Vec<u8>, PdfObject)>,
    index: HashMap<Vec<u8>, usize>,
}

impl PdfDict {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            index: HashMap::new(),
        }
    }

    pub fn from_entries(entries: Vec<(Vec<u8>, PdfObject)>) -> Self {
        let index: HashMap<Vec<u8>, usize> = entries
            .iter()
            .enumerate()
            .map(|(i, (k, _))| (k.clone(), i))
            .collect();
        Self { entries, index }
    }

    pub fn insert(&mut self, key: Vec<u8>, value: PdfObject) {
        if let Some(&pos) = self.index.get(&key) {
            self.entries[pos].1 = value;
        } else {
            let pos = self.entries.len();
            self.entries.push((key.clone(), value));
            self.index.insert(key, pos);
        }
    }

    pub fn get(&self, key: &[u8]) -> Option<&PdfObject> {
        self.index.get(key).map(|&i| &self.entries[i].1)
    }

    pub fn get_str(&self, key: &str) -> Option<&PdfObject> {
        self.get(key.as_bytes())
    }

    pub fn get_name(&self, key: &str) -> Option<&str> {
        self.get_str(key).and_then(|o| o.as_name_str())
    }

    pub fn get_integer(&self, key: &str) -> Option<i64> {
        self.get_str(key).and_then(|o| o.as_integer())
    }

    pub fn get_real(&self, key: &str) -> Option<f64> {
        self.get_str(key).and_then(|o| o.as_real())
    }

    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.get_str(key).and_then(|o| o.as_string_utf8())
    }

    pub fn get_dict(&self, key: &str) -> Option<&PdfDict> {
        self.get_str(key).and_then(|o| o.as_dict())
    }

    pub fn get_array(&self, key: &str) -> Option<&[PdfObject]> {
        self.get_str(key).and_then(|o| o.as_array())
    }

    pub fn get_reference(&self, key: &str) -> Option<ObjRef> {
        self.get_str(key).and_then(|o| o.as_reference())
    }

    pub fn contains_key_str(&self, key: &str) -> bool {
        self.index.contains_key(key.as_bytes())
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&[u8], &PdfObject)> {
        self.entries.iter().map(|(k, v)| (k.as_slice(), v))
    }
}

impl Default for PdfDict {
    fn default() -> Self {
        Self::new()
    }
}

/// A PDF stream: dictionary + raw byte data.
#[derive(Debug, Clone, PartialEq)]
pub struct PdfStream {
    pub dict: PdfDict,
    pub data: Vec<u8>,
}

impl PdfStream {
    pub fn new(dict: PdfDict, data: Vec<u8>) -> Self {
        Self { dict, data }
    }

    pub fn filter(&self) -> Option<&PdfObject> {
        self.dict.get_str("Filter")
    }

    pub fn decode_parms(&self) -> Option<&PdfObject> {
        self.dict.get_str("DecodeParms")
    }

    pub fn length(&self) -> Option<i64> {
        self.dict.get_integer("Length")
    }
}

/// An indirect object: `7 0 obj ... endobj`.
#[derive(Debug, Clone)]
pub struct IndirectObject {
    pub obj_num: u32,
    pub gen_num: u16,
    pub object: PdfObject,
}

/// Recursive descent parser for PDF objects.
pub struct ObjectParser<'a> {
    input: &'a [u8],
    pub pos: usize,
}

impl<'a> ObjectParser<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Self { input, pos: 0 }
    }

    pub fn remaining(&self) -> &'a [u8] {
        &self.input[self.pos..]
    }

    /// Parse a single PDF object.
    pub fn parse_object(&mut self) -> Result<PdfObject> {
        let remaining = &self.input[self.pos..];
        let (new_remaining, token) = lexer::next_token(remaining)
            .map_err(|e| BotlError::ParseError(format!("Lexer: {:?}", e)))?;
        self.pos = self.input.len() - new_remaining.len();

        match token {
            lexer::Token::Integer(i) => {
                // Check for indirect reference: `7 0 R`
                let saved_pos = self.pos;
                if let Ok(obj) = self.try_parse_reference(i) {
                    return Ok(obj);
                }
                self.pos = saved_pos;
                Ok(PdfObject::Integer(i))
            }
            lexer::Token::Real(f) => Ok(PdfObject::Real(f)),
            lexer::Token::String(s) => Ok(PdfObject::String(s)),
            lexer::Token::HexString(s) => Ok(PdfObject::String(s)),
            lexer::Token::Name(n) => Ok(PdfObject::Name(n)),
            lexer::Token::Boolean(b) => Ok(PdfObject::Boolean(b)),
            lexer::Token::Null => Ok(PdfObject::Null),
            lexer::Token::ArrayStart => {
                let mut items = Vec::new();
                loop {
                    let rem = &self.input[self.pos..];
                    let (r, tok) = lexer::next_token(rem)
                        .map_err(|e| BotlError::ParseError(format!("In array: {:?}", e)))?;
                    if matches!(tok, lexer::Token::ArrayEnd) {
                        self.pos = self.input.len() - r.len();
                        return Ok(PdfObject::Array(items));
                    }
                    // Don't consume; let parse_object handle it from current pos
                    items.push(self.parse_object()?);
                }
            }
            lexer::Token::DictStart => {
                let dict = self.parse_dict_contents()?;
                // Check for stream keyword
                let remaining = &self.input[self.pos..];
                let skipped = lexer::skip_ws(remaining);
                if skipped.starts_with(b"stream")
                    && (skipped.len() == 6 || skipped[6] == b'\r' || skipped[6] == b'\n')
                {
                    let stream = self.parse_stream_data(dict, remaining, skipped)?;
                    return Ok(PdfObject::Stream(stream));
                }
                Ok(PdfObject::Dictionary(dict))
            }
            _ => Err(BotlError::ParseError(format!(
                "Unexpected token: {:?}",
                token
            ))),
        }
    }

    fn try_parse_reference(&mut self, obj_num: i64) -> Result<PdfObject> {
        let remaining = &self.input[self.pos..];
        let (r1, tok1) = lexer::next_token(remaining)
            .map_err(|e| BotlError::ParseError(format!("Lexer: {:?}", e)))?;
        let gen_num = match tok1 {
            lexer::Token::Integer(g) => g,
            _ => return Err(BotlError::ParseError("Not a reference".into())),
        };

        let remaining2 = &self.input[self.input.len() - r1.len()..];
        let (r2, tok2) = lexer::next_token(remaining2)
            .map_err(|e| BotlError::ParseError(format!("Lexer: {:?}", e)))?;
        match tok2 {
            lexer::Token::Name(ref n) if n == b"R" => {
                self.pos = self.input.len() - r2.len();
                Ok(PdfObject::Reference(ObjRef::new(obj_num as u32, gen_num as u16)))
            }
            _ => Err(BotlError::ParseError("Not a reference".into())),
        }
    }

    fn parse_dict_contents(&mut self) -> Result<PdfDict> {
        let mut entries = Vec::new();
        loop {
            let remaining = &self.input[self.pos..];
            let (r, token) = lexer::next_token(remaining)
                .map_err(|e| BotlError::ParseError(format!("In dict: {:?}", e)))?;

            if matches!(token, lexer::Token::DictEnd) {
                self.pos = self.input.len() - r.len();
                return Ok(PdfDict::from_entries(entries));
            }

            let key = match token {
                lexer::Token::Name(n) => n,
                _ => return Err(BotlError::ParseError("Dict key must be a name".into())),
            };
            self.pos = self.input.len() - r.len();
            let value = self.parse_object()?;
            entries.push((key, value));
        }
    }

    fn parse_stream_data(
        &mut self,
        dict: PdfDict,
        remaining: &[u8],
        skipped: &[u8],
    ) -> Result<PdfStream> {
        let stream_kw_end = remaining.len() - skipped.len() + 6;
        let data_start = if stream_kw_end < remaining.len() && remaining[stream_kw_end] == b'\r' {
            stream_kw_end + if stream_kw_end + 1 < remaining.len() && remaining[stream_kw_end + 1] == b'\n' { 2 } else { 1 }
        } else if stream_kw_end < remaining.len() && remaining[stream_kw_end] == b'\n' {
            stream_kw_end + 1
        } else {
            stream_kw_end
        };

        let declared_length = dict.get_integer("Length").unwrap_or(0) as usize;
        let abs_data_start = self.pos + data_start;

        let (data_end, abs_end) = if declared_length > 0 && abs_data_start + declared_length <= self.input.len() {
            let candidate_end = abs_data_start + declared_length;
            let after = lexer::skip_ws(&self.input[candidate_end..]);
            if after.starts_with(b"endstream") {
                (candidate_end, self.input.len() - after.len() + 9)
            } else {
                self.find_endstream(abs_data_start)?
            }
        } else {
            self.find_endstream(abs_data_start)?
        };

        let data = self.input[abs_data_start..data_end].to_vec();
        self.pos = abs_end;
        Ok(PdfStream::new(dict, data))
    }

    fn find_endstream(&self, start: usize) -> Result<(usize, usize)> {
        let haystack = &self.input[start..];
        for i in 0..haystack.len().saturating_sub(9) {
            if &haystack[i..i + 9] == b"endstream" {
                return Ok((start + i, start + i + 9));
            }
        }
        Err(BotlError::ParseError("Could not find endstream".into()))
    }

    /// Parse an indirect object: `7 0 obj ... endobj`.
    pub fn parse_indirect_object(&mut self) -> Result<IndirectObject> {
        let obj_num = match self.parse_object()? {
            PdfObject::Integer(i) => i as u32,
            _ => return Err(BotlError::ParseError("Expected object number".into())),
        };
        let gen_num = match self.parse_object()? {
            PdfObject::Integer(i) => i as u16,
            _ => return Err(BotlError::ParseError("Expected generation number".into())),
        };

        // Expect "obj" keyword
        let remaining = &self.input[self.pos..];
        let (r, token) = lexer::next_token(remaining)
            .map_err(|e| BotlError::ParseError(format!("Lexer: {:?}", e)))?;
        match &token {
            lexer::Token::Name(n) if n == b"obj" => {}
            _ => return Err(BotlError::ParseError("Expected 'obj' keyword".into())),
        }
        self.pos = self.input.len() - r.len();

        let object = self.parse_object()?;

        // Consume "endobj" if present
        let remaining = &self.input[self.pos..];
        let skipped = lexer::skip_ws(remaining);
        if skipped.starts_with(b"endobj") {
            self.pos = self.input.len() - skipped.len() + 6;
        }

        Ok(IndirectObject {
            obj_num,
            gen_num,
            object,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dict() {
        let mut dict = PdfDict::new();
        dict.insert(b"Type".to_vec(), PdfObject::Name(b"Catalog".to_vec()));
        dict.insert(b"Pages".to_vec(), PdfObject::Reference(ObjRef::new(2, 0)));

        assert_eq!(dict.get_name("Type"), Some("Catalog"));
        assert_eq!(dict.get_reference("Pages"), Some(ObjRef::new(2, 0)));
    }

    #[test]
    fn test_parse_simple_dict() {
        let input = b"<< /Type /Catalog /Pages 2 0 R >>";
        let mut parser = ObjectParser::new(input);
        let obj = parser.parse_object().unwrap();
        let dict = obj.as_dict().unwrap();
        assert_eq!(dict.get_name("Type"), Some("Catalog"));
        assert_eq!(dict.get_reference("Pages"), Some(ObjRef::new(2, 0)));
    }

    #[test]
    fn test_parse_array() {
        let input = b"[1 2 (hello) /Name]";
        let mut parser = ObjectParser::new(input);
        let obj = parser.parse_object().unwrap();
        let arr = obj.as_array().unwrap();
        assert_eq!(arr.len(), 4);
        assert_eq!(arr[0].as_integer(), Some(1));
        assert_eq!(arr[2].as_string_utf8(), Some("hello"));
        assert_eq!(arr[3].as_name_str(), Some("Name"));
    }

    #[test]
    fn test_parse_reference() {
        let input = b"7 0 R";
        let mut parser = ObjectParser::new(input);
        let obj = parser.parse_object().unwrap();
        assert_eq!(obj.as_reference(), Some(ObjRef::new(7, 0)));
    }

    #[test]
    fn test_parse_indirect_object() {
        let input = b"1 0 obj << /Type /Catalog >> endobj";
        let mut parser = ObjectParser::new(input);
        let indirect = parser.parse_indirect_object().unwrap();
        assert_eq!(indirect.obj_num, 1);
        assert_eq!(indirect.gen_num, 0);
        assert_eq!(indirect.object.as_dict().unwrap().get_name("Type"), Some("Catalog"));
    }

    #[test]
    fn test_parse_nested_dict_with_resources() {
        // Reproduces real PDF structure from calibre-generated PDFs (NO spaces between >>)
        let input = b"<</Type/Page/Contents 25 1 R/MediaBox[ 0 0 612 792]/Parent 18 0 R/Resources<</ProcSet[/PDF/Text/ImageB/ImageC/ImageI]/XObject<</XOb13 13 1 R>>>>>>";
        let mut parser = ObjectParser::new(input);
        let obj = parser.parse_object().unwrap();
        let dict = obj.as_dict().unwrap();
        assert_eq!(dict.get_name("Type"), Some("Page"));
        let resources = dict.get_dict("Resources").unwrap();
        assert!(resources.get_array("ProcSet").is_some());
        let xobject = resources.get_dict("XObject").unwrap();
        assert_eq!(xobject.get_reference("XOb13"), Some(ObjRef::new(13, 1)));
    }

    #[test]
    fn test_parse_deeply_nested_dicts() {
        // Triple-nested dict with consecutive >> >> >>
        let input = b"<< /A << /B << /C 42 >> >> >>";
        let mut parser = ObjectParser::new(input);
        let obj = parser.parse_object().unwrap();
        let dict = obj.as_dict().unwrap();
        let a = dict.get_dict("A").unwrap();
        let b = a.get_dict("B").unwrap();
        assert_eq!(b.get_integer("C"), Some(42));
    }
}

    #[test]
    fn test_debug_nested_dicts() {
        // Start simpler and build up
        // Level 1: just << /C 42 >>
        let input = b"<< /C 42 >>";
        let mut parser = ObjectParser::new(input);
        let obj = parser.parse_object().unwrap();
        assert!(obj.as_dict().is_some(), "Level 1 failed");
        
        // Level 2: << /B << /C 42 >> >>
        let input = b"<< /B << /C 42 >> >>";
        let mut parser = ObjectParser::new(input);
        let obj = parser.parse_object().unwrap();
        assert!(obj.as_dict().is_some(), "Level 2 with spaces failed");
        
        // Level 2 no spaces: << /B<</C 42>>>>>
        let input = b"<< /B<</C 42>>>>";
        let mut parser = ObjectParser::new(input);
        let obj = parser.parse_object().unwrap();
        assert!(obj.as_dict().is_some(), "Level 2 no spaces failed");
        
        // Level 3: << /A << /B << /C 42 >> >> >>
        let input = b"<< /A << /B << /C 42 >> >> >>";
        let mut parser = ObjectParser::new(input);
        let obj = parser.parse_object().unwrap();
        assert!(obj.as_dict().is_some(), "Level 3 with spaces failed");
        
        // Level 3 no spaces: << /A<< /B<< /C 42 >>>>>>>
        let input = b"<< /A<< /B<< /C 42 >>>>>>";
        let mut parser = ObjectParser::new(input);
        let obj = parser.parse_object();
        match &obj {
            Ok(_) => {},
            Err(e) => panic!("Level 3 no spaces failed: {:?}", e),
        }
    }

    #[test]
    fn test_level2_no_spaces_minimal() {
        let input = b"<< /B<</C 42>>>>";
        eprintln!("Input: {:?}", String::from_utf8_lossy(input));
        eprintln!("Input len: {}", input.len());
        let mut parser = ObjectParser::new(input);
        let obj = parser.parse_object();
        match &obj {
            Ok(_) => {},
            Err(e) => {
                eprintln!("FAILED at pos {}:", parser.pos);
                eprintln!("Remaining: {:?}", String::from_utf8_lossy(&input[parser.pos.min(input.len())..]));
                panic!("Failed: {:?}", e);
            }
        }
    }
