use crate::error::Result;
use crate::geometry::Matrix;
use crate::parser::objects::{ObjRef, PdfDict, PdfObject};
use crate::text::cmap::CMap;
use hashbrown::HashMap;

/// Represents a PDF font with its metrics and encoding.
#[derive(Debug, Clone)]
pub struct Font {
    /// Unique name within the page resource dictionary.
    pub name: String,
    /// Base font name (e.g., "Helvetica", "Times-Roman").
    pub base_font: Option<String>,
    /// Font subtype: Type0, Type1, TrueType, CIDFontType0, CIDFontType2.
    pub subtype: FontSubtype,
    /// Glyph widths indexed by character code.
    widths: HashMap<u32, f64>,
    /// Default width for glyphs not in the widths table.
    default_width: f64,
    /// Font size last set by Tf operator.
    pub size: f64,
    /// Font bounding box [x_min, y_min, x_max, y_max] in glyph units.
    pub bbox: Option<[f64; 4]>,
    /// Encoding name (e.g., "WinAnsiEncoding", "MacRomanEncoding").
    pub encoding: Option<String>,
    /// ToUnicode CMap name.
    pub to_unicode_ref: Option<ObjRef>,
    /// Resolved ToUnicode CMap for character mapping.
    cmap: Option<CMap>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontSubtype {
    Type1,
    Type1C, // Type1C (CFF)
    TrueType,
    Type0,
    CIDFontType0,
    CIDFontType2,
    Bitmap,
    Unknown,
}

impl Font {
    /// Create a new font with the given resource name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            base_font: None,
            subtype: FontSubtype::Unknown,
            widths: HashMap::new(),
            default_width: 1000.0, // Standard default for Type1/TT fonts
            size: 12.0,
            bbox: None,
            encoding: None,
            to_unicode_ref: None,
            cmap: None,
        }
    }

    /// Parse a font from its PDF dictionary.
    pub fn from_dict(name: &str, dict: &PdfDict) -> Result<Self> {
        let mut font = Self::new(name);

        font.base_font = dict.get_name("BaseFont").map(String::from);
        font.subtype = match dict.get_name("Subtype").unwrap_or("") {
            "Type1" => FontSubtype::Type1,
            "Type1C" => FontSubtype::Type1C,
            "TrueType" => FontSubtype::TrueType,
            "Type0" => FontSubtype::Type0,
            "CIDFontType0" => FontSubtype::CIDFontType0,
            "CIDFontType2" => FontSubtype::CIDFontType2,
            "MMType1" => FontSubtype::Type1,
            _ => FontSubtype::Unknown,
        };

        // For Type0 (composite) fonts, look into DescendantFonts for encoding
        if font.subtype == FontSubtype::Type0 {
            // ToUnicode stays at the top-level Type0 font dict
            font.to_unicode_ref = dict.get_reference("ToUnicode");

            // Descend into the descendant CIDFont dict for Encoding
            if let Some(desc_arr) = dict.get_array("DescendantFonts") {
                if let Some(desc_obj) = desc_arr.first() {
                    if let Some(desc_dict) = desc_obj.as_dict() {
                        // Encoding in the descendant CIDFont dict
                        font.encoding = Self::extract_encoding_name(desc_dict);

                        // Parse widths from the descendant CIDFont dict
                        font.parse_cid_widths_from_dict(desc_dict);
                    } else if let Some(desc_ref) = desc_obj.as_reference() {
                        // If it's a reference, we can't resolve it here (no doc access).
                        // Store the reference; the caller (page.rs build_font_cache)
                        // can resolve it later. For now, fall back to top-level W/DW.
                        _ = desc_ref;
                        font.parse_widths(dict)?;
                    }
                }
            }
        } else {
            // Simple fonts: Encoding and ToUnicode are directly on the font dict
            font.encoding = Self::extract_encoding_name(dict);
            font.to_unicode_ref = dict.get_reference("ToUnicode");

            // Parse FontDescriptor for metrics
            if let Some(_fd_ref) = dict.get_reference("FontDescriptor") {
                // We'd need to resolve the reference via the document.
                // For now, store the reference. The document layer handles resolution.
            }

            // Parse widths based on font type
            font.parse_widths(dict)?;
        }

        Ok(font)
    }

    /// Extract the encoding name from a dictionary, handling both Name and
    /// Dictionary forms of the /Encoding entry.
    fn extract_encoding_name(dict: &PdfDict) -> Option<String> {
        match dict.get_str("Encoding") {
            Some(PdfObject::Name(n)) => std::str::from_utf8(n).ok().map(String::from),
            Some(PdfObject::Reference(_)) => {
                // Encoding is an indirect reference; we cannot resolve it here.
                // The caller should resolve and set the encoding name later.
                None
            }
            Some(PdfObject::Dictionary(enc_dict)) => {
                // Encoding dict: look for /BaseEncoding name.
                // If present, use that. Otherwise /Type may indicate the kind.
                enc_dict
                    .get_name("BaseEncoding")
                    .map(|base| base.to_string())
            }
            Some(PdfObject::String(s)) => std::str::from_utf8(s).ok().map(String::from),
            _ => None,
        }
    }

    fn parse_widths(&mut self, dict: &PdfDict) -> Result<()> {
        match self.subtype {
            FontSubtype::Type1 | FontSubtype::Type1C | FontSubtype::TrueType => {
                // Simple fonts: FirstChar, LastChar, Widths array
                let first_char = dict.get_integer("FirstChar").unwrap_or(0) as u32;
                if let Some(widths_arr) = dict.get_array("Widths") {
                    for (i, w) in widths_arr.iter().enumerate() {
                        let char_code = first_char + i as u32;
                        if let Some(width) = w.as_real() {
                            self.widths.insert(char_code, width);
                        } else if let Some(width) = w.as_integer() {
                            self.widths.insert(char_code, width as f64);
                        }
                    }
                }
            }
            FontSubtype::Type0 => {
                // Composite fonts: W and DW entries in descendant CIDFont
                self.default_width = dict.get_real("DW").unwrap_or(1000.0);
                if let Some(w_array) = dict.get_array("W") {
                    self.parse_cid_widths(w_array);
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Parse CID widths from a descendant CIDFont dictionary.
    fn parse_cid_widths_from_dict(&mut self, dict: &PdfDict) {
        self.default_width = dict.get_real("DW").unwrap_or(1000.0);
        if let Some(w_array) = dict.get_array("W") {
            self.parse_cid_widths(w_array);
        }
    }

    /// Parse CID font width entries from W array.
    /// Format: c [w1 w2 ...] or c_first c_last w
    fn parse_cid_widths(&mut self, w_array: &[PdfObject]) {
        let mut i = 0;
        while i < w_array.len() {
            if let Some(c) = w_array[i].as_integer() {
                let c = c as u32;
                if i + 1 < w_array.len() {
                    if let Some(arr) = w_array[i + 1].as_array() {
                        // c [w1 w2 ...]: consecutive widths starting at c
                        for (j, w) in arr.iter().enumerate() {
                            let width = w
                                .as_real()
                                .or_else(|| w.as_integer().map(|v| v as f64))
                                .unwrap_or(self.default_width);
                            self.widths.insert(c + j as u32, width);
                        }
                        i += 2;
                    } else if let Some(c_last) = w_array[i + 1].as_integer() {
                        // c_first c_last w: same width for range
                        if i + 2 < w_array.len() {
                            let width_obj = &w_array[i + 2];
                            let w = width_obj
                                .as_real()
                                .or_else(|| width_obj.as_integer().map(|v| v as f64))
                                .unwrap_or(self.default_width);
                            for cid in c..=(c_last as u32) {
                                self.widths.insert(cid, w);
                            }
                            i += 3;
                        } else {
                            i += 2;
                        }
                    } else {
                        i += 1;
                    }
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
    }

    /// Get the width of a glyph in text space units.
    /// Returns the width scaled by the current font size and CTM.
    pub fn get_glyph_width(&self, char_code: u32, font_size: f64, ctm: &Matrix) -> f64 {
        let raw_width = self
            .widths
            .get(&char_code)
            .copied()
            .unwrap_or(self.default_width);
        // Width is in glyph units (typically 1/1000 of text space)
        let text_width = (raw_width / 1000.0) * font_size;
        // Apply horizontal scaling from CTM
        let (dx, _) = ctm.transform_vector(text_width, 0.0);
        dx.abs()
    }

    /// Get the raw width in glyph space units (unscaled).
    pub fn get_raw_width(&self, char_code: u32) -> f64 {
        self.widths
            .get(&char_code)
            .copied()
            .unwrap_or(self.default_width)
    }

    pub fn set_size(&mut self, size: f64) {
        self.size = size;
    }

    /// Set the resolved ToUnicode CMap for this font.
    pub fn set_cmap(&mut self, cmap: CMap) {
        self.cmap = Some(cmap);
    }

    /// Get a reference to the resolved ToUnicode CMap, if any.
    pub fn cmap(&self) -> Option<&CMap> {
        self.cmap.as_ref()
    }
}

/// Cache of fonts used on a page, keyed by resource name.
#[derive(Debug, Clone)]
pub struct FontCache {
    fonts: HashMap<Vec<u8>, Font>,
}

impl FontCache {
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
        }
    }

    /// Insert a font into the cache.
    pub fn insert(&mut self, name: &[u8], font: Font) {
        self.fonts.insert(name.to_vec(), font);
    }

    /// Get a font by its resource name.
    pub fn get(&self, name: &[u8]) -> Option<&Font> {
        self.fonts.get(name)
    }

    /// Get a mutable reference to a font by resource name.
    pub fn get_mut(&mut self, name: &[u8]) -> Option<&mut Font> {
        self.fonts.get_mut(name)
    }
}

impl Default for FontCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_creation() {
        let font = Font::new("F1");
        assert_eq!(font.name, "F1");
        assert_eq!(font.size, 12.0);
    }

    #[test]
    fn test_simple_font_widths() {
        let mut dict = PdfDict::new();
        dict.insert(b"Subtype".to_vec(), PdfObject::Name(b"Type1".to_vec()));
        dict.insert(b"FirstChar".to_vec(), PdfObject::Integer(65)); // 'A'
        dict.insert(
            b"Widths".to_vec(),
            PdfObject::Array(vec![
                PdfObject::Integer(600), // A
                PdfObject::Integer(700), // B
                PdfObject::Integer(500), // C
            ]),
        );

        let font = Font::from_dict("F1", &dict).unwrap();
        assert_eq!(font.get_raw_width(65), 600.0); // A
        assert_eq!(font.get_raw_width(66), 700.0); // B
        assert_eq!(font.get_raw_width(67), 500.0); // C
        assert_eq!(font.get_raw_width(68), 1000.0); // D (default)
    }

    #[test]
    fn test_glyph_width_scaled() {
        let mut font = Font::new("F1");
        font.widths.insert(65, 600.0);
        let ctm = Matrix::IDENTITY;
        let width = font.get_glyph_width(65, 12.0, &ctm);
        assert!((width - 7.2).abs() < 0.001); // 600/1000 * 12 = 7.2
    }

    #[test]
    fn test_type0_font_encoding_from_descendant() {
        // Type0 font with DescendantFonts containing CIDFont with Encoding
        let mut descendant = PdfDict::new();
        descendant.insert(
            b"Subtype".to_vec(),
            PdfObject::Name(b"CIDFontType2".to_vec()),
        );
        descendant.insert(
            b"Encoding".to_vec(),
            PdfObject::Name(b"Identity-H".to_vec()),
        );

        let mut dict = PdfDict::new();
        dict.insert(b"Subtype".to_vec(), PdfObject::Name(b"Type0".to_vec()));
        dict.insert(b"BaseFont".to_vec(), PdfObject::Name(b"Helvetica".to_vec()));
        dict.insert(
            b"DescendantFonts".to_vec(),
            PdfObject::Array(vec![PdfObject::Dictionary(descendant)]),
        );

        let font = Font::from_dict("F1", &dict).unwrap();
        assert_eq!(font.subtype, FontSubtype::Type0);
        assert_eq!(font.encoding.as_deref(), Some("Identity-H"));
    }

    #[test]
    fn test_encoding_as_dict() {
        // Font where Encoding is a dict with BaseEncoding
        let mut enc_dict = PdfDict::new();
        enc_dict.insert(
            b"BaseEncoding".to_vec(),
            PdfObject::Name(b"WinAnsiEncoding".to_vec()),
        );

        let mut dict = PdfDict::new();
        dict.insert(b"Subtype".to_vec(), PdfObject::Name(b"Type1".to_vec()));
        dict.insert(b"Encoding".to_vec(), PdfObject::Dictionary(enc_dict));

        let font = Font::from_dict("F1", &dict).unwrap();
        assert_eq!(font.encoding.as_deref(), Some("WinAnsiEncoding"));
    }

    #[test]
    fn test_set_and_get_cmap() {
        let mut font = Font::new("F1");
        assert!(font.cmap().is_none());

        let cmap = CMap::parse(b"1 beginbfchar\n<41> <0041>\nendbfchar\n").unwrap();
        font.set_cmap(cmap);
        assert!(font.cmap().is_some());
        assert_eq!(font.cmap().unwrap().to_char(0x41), Some('A'));
    }
}
