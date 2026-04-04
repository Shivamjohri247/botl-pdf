use crate::geometry::BBox;

/// A single character with its position, font, and style information.
#[derive(Debug, Clone)]
pub struct Char {
    /// The Unicode text of this character.
    pub text: String,
    /// Bounding box in page coordinates (top-left origin).
    pub bbox: BBox,
    /// Font resource name (e.g., "F1", "R7").
    pub font_name: String,
    /// Font size in points.
    pub font_size: f64,
    /// Whether the character is bold.
    pub bold: bool,
    /// Whether the character is italic.
    pub italic: bool,
    /// Fill color as (R, G, B), normalized 0.0–1.0.
    pub color: Option<(f64, f64, f64)>,
    /// Stroke color as (R, G, B), normalized 0.0–1.0.
    pub stroking_color: Option<(f64, f64, f64)>,
    /// Rotation in degrees (0, 90, 180, 270).
    pub rotation: f64,
    /// Text run identifier: characters from the same Tj/TJ call share a run_id.
    /// Used to prevent interleaving of characters from different text operations.
    pub run_id: u32,
}

impl Char {
    /// Get the center point of this character.
    pub fn center(&self) -> (f64, f64) {
        self.bbox.center()
    }

    /// Get the width of this character's bounding box.
    pub fn width(&self) -> f64 {
        self.bbox.width()
    }

    /// Get the height of this character's bounding box.
    pub fn height(&self) -> f64 {
        self.bbox.height()
    }
}

/// A word: a group of characters that form a single word.
#[derive(Debug, Clone)]
pub struct Word {
    /// The text content of this word.
    pub text: String,
    /// Bounding box enclosing all characters in this word.
    pub bbox: BBox,
    /// The characters that make up this word.
    pub chars: Vec<Char>,
    /// Text direction: "ltr", "rtl", "ttb", "btt".
    pub direction: String,
    /// Dominant font name.
    pub font_name: String,
    /// Dominant font size.
    pub font_size: f64,
}

impl Word {
    pub fn from_chars(chars: Vec<Char>) -> Option<Self> {
        if chars.is_empty() {
            return None;
        }

        let text: String = chars.iter().map(|c| c.text.as_str()).collect();
        let mut bbox = chars[0].bbox;
        for c in &chars[1..] {
            bbox = bbox.merge(&c.bbox);
        }

        // Use the font/size of the first char as dominant
        let font_name = chars[0].font_name.clone();
        let font_size = chars[0].font_size;

        Some(Word {
            text,
            bbox,
            chars,
            direction: "ltr".to_string(),
            font_name,
            font_size,
        })
    }

    pub fn width(&self) -> f64 {
        self.bbox.width()
    }

    pub fn height(&self) -> f64 {
        self.bbox.height()
    }
}

/// A line of text: words arranged horizontally.
#[derive(Debug, Clone)]
pub struct TextLine {
    /// All words in this line, in reading order.
    pub words: Vec<Word>,
    /// Bounding box enclosing all words.
    pub bbox: BBox,
    /// The combined text of all words.
    pub text: String,
}

impl TextLine {
    pub fn from_words(words: Vec<Word>) -> Option<Self> {
        if words.is_empty() {
            return None;
        }

        let text = words
            .iter()
            .map(|w| w.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        let mut bbox = words[0].bbox;
        for w in &words[1..] {
            bbox = bbox.merge(&w.bbox);
        }

        Some(TextLine { words, bbox, text })
    }
}

/// A block of text: a paragraph-like grouping of lines.
#[derive(Debug, Clone)]
pub struct TextBlock {
    /// All lines in this block, in reading order.
    pub lines: Vec<TextLine>,
    /// Bounding box enclosing all lines.
    pub bbox: BBox,
    /// The combined text of all lines.
    pub text: String,
}

impl TextBlock {
    pub fn from_lines(lines: Vec<TextLine>) -> Option<Self> {
        if lines.is_empty() {
            return None;
        }

        let text = lines
            .iter()
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let mut bbox = lines[0].bbox;
        for l in &lines[1..] {
            bbox = bbox.merge(&l.bbox);
        }

        Some(TextBlock { lines, bbox, text })
    }
}

/// A geometric line on the page (not a text line).
#[derive(Debug, Clone)]
pub struct GeomLine {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
    pub line_width: f64,
    pub color: Option<(f64, f64, f64)>,
}

/// A geometric rectangle on the page.
#[derive(Debug, Clone)]
pub struct GeomRect {
    pub bbox: BBox,
    pub line_width: f64,
    pub stroke_color: Option<(f64, f64, f64)>,
    pub fill_color: Option<(f64, f64, f64)>,
}
