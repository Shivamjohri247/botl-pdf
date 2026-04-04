use crate::error::{BotlError, Result};
use crate::geometry::{BBox, Matrix, Point};
use crate::layout::elements::Char;
use crate::text::fonts::FontCache;
use crate::text::unicode;

/// Graphics state saved/restored by q/Q operators.
#[derive(Debug, Clone)]
pub struct GraphicsState {
    /// Current transformation matrix.
    pub ctm: Matrix,
    /// Text matrix (set by Tm).
    pub text_matrix: Matrix,
    /// Text rendering mode.
    pub rendering_mode: i32,
    /// Character spacing (Tc).
    pub char_spacing: f64,
    /// Word spacing (Tw).
    pub word_spacing: f64,
    /// Horizontal scaling (Tz), as a percentage (100 = normal).
    pub horizontal_scaling: f64,
    /// Text leading (TL).
    pub leading: f64,
    /// Text rise (Ts).
    pub text_rise: f64,
    /// Current font name (key into font cache).
    pub font_name: Option<Vec<u8>>,
    /// Current font size.
    pub font_size: f64,
    /// Fill color (RGB).
    pub fill_color: (f64, f64, f64),
    /// Stroke color (RGB).
    pub stroke_color: (f64, f64, f64),
    /// Line width.
    pub line_width: f64,
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            ctm: Matrix::IDENTITY,
            text_matrix: Matrix::IDENTITY,
            rendering_mode: 0,
            char_spacing: 0.0,
            word_spacing: 0.0,
            horizontal_scaling: 100.0,
            leading: 0.0,
            text_rise: 0.0,
            font_name: None,
            font_size: 12.0,
            fill_color: (0.0, 0.0, 0.0),
            stroke_color: (0.0, 0.0, 0.0),
            line_width: 1.0,
        }
    }
}

/// Result of interpreting a content stream.
#[derive(Debug, Clone)]
pub struct InterpretResult {
    /// Extracted characters with positions.
    pub chars: Vec<Char>,
    /// Geometric lines extracted from the content stream.
    pub lines: Vec<LineElement>,
    /// Geometric rectangles.
    pub rects: Vec<RectElement>,
}

#[derive(Debug, Clone)]
pub struct LineElement {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
    pub line_width: f64,
    pub color: (f64, f64, f64),
}

#[derive(Debug, Clone)]
pub struct RectElement {
    pub bbox: BBox,
    pub line_width: f64,
    pub stroke_color: Option<(f64, f64, f64)>,
    pub fill_color: Option<(f64, f64, f64)>,
}

/// Interpret a PDF content stream and extract positioned characters and geometry.
pub fn interpret_content_stream(
    data: &[u8],
    font_cache: &FontCache,
    page_height: f64,
) -> Result<InterpretResult> {
    let text = std::str::from_utf8(data)
        .map_err(|_| BotlError::ParseError("Content stream is not valid UTF-8".into()))?;

    let mut state = GraphicsState::default();
    let mut state_stack: Vec<GraphicsState> = Vec::new();
    let mut chars = Vec::new();
    let mut lines = Vec::new();
    let mut rects = Vec::new();

    // Track last emitted character position for synthetic space injection
    let mut last_char_end_x: Option<f64> = None;
    let mut last_char_center_y: Option<f64> = None;

    // Text run identifier: incremented for each Tj/TJ/'/" call so that
    // characters from the same text-showing operation share a run_id.
    let mut run_id: u32 = 0;

    // Current path for path construction operators
    let mut current_x = 0.0f64;
    let mut current_y = 0.0f64;

    // Tokenize and interpret operators
    let mut token_iter = ContentStreamTokenizer::new(text);

    while let Some(tokens) = token_iter.next_operator()? {
        let (operands, operator) = tokens;

        match operator.as_str() {
            // === Text state operators ===
            "Tf" => {
                // Set font: font_name font_size Tf
                if operands.len() >= 2 {
                    if let Some(name) = operands[0].as_name() {
                        state.font_name = Some(name.to_vec());
                    }
                    state.font_size = operands[1]
                        .as_real()
                        .or_else(|| operands[1].as_integer().map(|i| i as f64))
                        .unwrap_or(12.0);
                }
            }
            "Tc" => {
                if let Some(v) = operands.first().and_then(|o| o.as_real()) {
                    state.char_spacing = v;
                }
            }
            "Tw" => {
                if let Some(v) = operands.first().and_then(|o| o.as_real()) {
                    state.word_spacing = v;
                }
            }
            "Tz" => {
                if let Some(v) = operands.first().and_then(|o| o.as_real()) {
                    state.horizontal_scaling = v;
                }
            }
            "TL" => {
                if let Some(v) = operands.first().and_then(|o| o.as_real()) {
                    state.leading = v;
                }
            }
            "Tr" => {
                if let Some(v) = operands.first().and_then(|o| o.as_integer()) {
                    state.rendering_mode = v as i32;
                }
            }
            "Ts" => {
                if let Some(v) = operands.first().and_then(|o| o.as_real()) {
                    state.text_rise = v;
                }
            }

            // === Text positioning operators ===
            "Td" => {
                // Move to next line: tx ty Td
                if operands.len() >= 2 {
                    let tx = operands[0].as_real().unwrap_or(0.0);
                    let ty = operands[1].as_real().unwrap_or(0.0);
                    // Only reset tracking on significant vertical moves (line breaks)
                    if ty.abs() > state.font_size * 0.3 {
                        last_char_end_x = None;
                    }
                    let translate = Matrix::translate(tx, ty);
                    state.text_matrix = translate.multiply(&state.text_matrix);
                }
            }
            "TD" => {
                // Move to next line and set leading: tx ty TD
                if operands.len() >= 2 {
                    let tx = operands[0].as_real().unwrap_or(0.0);
                    let ty = operands[1].as_real().unwrap_or(0.0);
                    state.leading = -ty;
                    // Only reset tracking on significant vertical moves (line breaks)
                    if ty.abs() > state.font_size * 0.3 {
                        last_char_end_x = None;
                    }
                    let translate = Matrix::translate(tx, ty);
                    state.text_matrix = translate.multiply(&state.text_matrix);
                }
            }
            "Tm" => {
                // Set text matrix: a b c d e f Tm
                if operands.len() >= 6 {
                    state.text_matrix = Matrix::new(
                        operands[0].as_real().unwrap_or(1.0),
                        operands[1].as_real().unwrap_or(0.0),
                        operands[2].as_real().unwrap_or(0.0),
                        operands[3].as_real().unwrap_or(1.0),
                        operands[4].as_real().unwrap_or(0.0),
                        operands[5].as_real().unwrap_or(0.0),
                    );
                }
                // Tm completely repositions text — check if y changed significantly
                // Always reset since we don't know the previous y from this context
                last_char_end_x = None;
            }
            "T*" => {
                // Move to start of next line
                let translate = Matrix::translate(0.0, -state.leading);
                state.text_matrix = translate.multiply(&state.text_matrix);
                last_char_end_x = None;
            }

            // === Text showing operators ===
            "Tj" => {
                // Show string
                run_id += 1;
                if let Some(string_data) = operands.first().and_then(|o| o.as_string()) {
                    maybe_inject_space(
                        &state,
                        font_cache,
                        page_height,
                        &mut chars,
                        &mut last_char_end_x,
                        &mut last_char_center_y,
                    );
                    show_string(
                        string_data,
                        &mut state,
                        font_cache,
                        page_height,
                        &mut chars,
                        &mut last_char_end_x,
                        &mut last_char_center_y,
                        run_id,
                    );
                }
            }
            "'" => {
                // Move to next line and show string
                run_id += 1;
                let translate = Matrix::translate(0.0, -state.leading);
                state.text_matrix = translate.multiply(&state.text_matrix);
                last_char_end_x = None; // New line
                if let Some(string_data) = operands.first().and_then(|o| o.as_string()) {
                    maybe_inject_space(
                        &state,
                        font_cache,
                        page_height,
                        &mut chars,
                        &mut last_char_end_x,
                        &mut last_char_center_y,
                    );
                    show_string(
                        string_data,
                        &mut state,
                        font_cache,
                        page_height,
                        &mut chars,
                        &mut last_char_end_x,
                        &mut last_char_center_y,
                        run_id,
                    );
                }
            }
            "\"" => {
                // Set spacing, move to next line, show string: aw ac string
                run_id += 1;
                if operands.len() >= 3 {
                    state.word_spacing = operands[0].as_real().unwrap_or(0.0);
                    state.char_spacing = operands[1].as_real().unwrap_or(0.0);
                    let translate = Matrix::translate(0.0, -state.leading);
                    state.text_matrix = translate.multiply(&state.text_matrix);
                    last_char_end_x = None; // New line
                    if let Some(string_data) = operands[2].as_string() {
                        maybe_inject_space(
                            &state,
                            font_cache,
                            page_height,
                            &mut chars,
                            &mut last_char_end_x,
                            &mut last_char_center_y,
                        );
                        show_string(
                            string_data,
                            &mut state,
                            font_cache,
                            page_height,
                            &mut chars,
                            &mut last_char_end_x,
                            &mut last_char_center_y,
                            run_id,
                        );
                    }
                }
            }
            "TJ" => {
                // Show string array (with kerning adjustments)
                run_id += 1;
                if let Some(arr) = operands.first().and_then(|o| o.as_array()) {
                    maybe_inject_space(
                        &state,
                        font_cache,
                        page_height,
                        &mut chars,
                        &mut last_char_end_x,
                        &mut last_char_center_y,
                    );
                    for item in arr {
                        if let Some(string_data) = item.as_string() {
                            show_string(
                                string_data,
                                &mut state,
                                font_cache,
                                page_height,
                                &mut chars,
                                &mut last_char_end_x,
                                &mut last_char_center_y,
                                run_id,
                            );
                        } else if let Some(kern) = item
                            .as_real()
                            .or_else(|| item.as_integer().map(|i| i as f64))
                        {
                            // Kern value: adjust text position by -kern/1000 * font_size
                            if state.font_size > 0.0 {
                                let adjustment = -kern / 1000.0 * state.font_size;
                                let scaling = state.horizontal_scaling / 100.0;
                                state.text_matrix.e += adjustment * scaling;
                            }
                        }
                    }
                }
            }

            // === Graphics state operators ===
            "q" => {
                state_stack.push(state.clone());
            }
            "Q" => {
                if let Some(saved) = state_stack.pop() {
                    state = saved;
                }
            }
            "cm" => {
                // Concatenate matrix
                if operands.len() >= 6 {
                    let m = Matrix::new(
                        operands[0].as_real().unwrap_or(1.0),
                        operands[1].as_real().unwrap_or(0.0),
                        operands[2].as_real().unwrap_or(0.0),
                        operands[3].as_real().unwrap_or(1.0),
                        operands[4].as_real().unwrap_or(0.0),
                        operands[5].as_real().unwrap_or(0.0),
                    );
                    state.ctm = state.ctm.multiply(&m);
                }
            }

            // === Color operators ===
            "rg" => {
                // Fill color (RGB)
                if operands.len() >= 3 {
                    state.fill_color = (
                        operands[0].as_real().unwrap_or(0.0),
                        operands[1].as_real().unwrap_or(0.0),
                        operands[2].as_real().unwrap_or(0.0),
                    );
                }
            }
            "RG" => {
                // Stroke color (RGB)
                if operands.len() >= 3 {
                    state.stroke_color = (
                        operands[0].as_real().unwrap_or(0.0),
                        operands[1].as_real().unwrap_or(0.0),
                        operands[2].as_real().unwrap_or(0.0),
                    );
                }
            }
            "g" => {
                // Fill color (Gray)
                if let Some(v) = operands.first().and_then(|o| o.as_real()) {
                    state.fill_color = (v, v, v);
                }
            }
            "G" => {
                // Stroke color (Gray)
                if let Some(v) = operands.first().and_then(|o| o.as_real()) {
                    state.stroke_color = (v, v, v);
                }
            }

            // === Line width ===
            "w" => {
                if let Some(v) = operands.first().and_then(|o| o.as_real()) {
                    state.line_width = v;
                }
            }

            // === Path construction ===
            "m" => {
                // moveto
                if operands.len() >= 2 {
                    current_x = operands[0].as_real().unwrap_or(0.0);
                    current_y = operands[1].as_real().unwrap_or(0.0);
                }
            }
            "l" => {
                // lineto
                if operands.len() >= 2 {
                    let x1 = operands[0].as_real().unwrap_or(0.0);
                    let y1 = operands[1].as_real().unwrap_or(0.0);
                    // Transform through CTM
                    let p0 = state.ctm.transform_point(&Point::new(current_x, current_y));
                    let p1 = state.ctm.transform_point(&Point::new(x1, y1));
                    // Flip Y for top-left origin
                    lines.push(LineElement {
                        x0: p0.x,
                        y0: page_height - p0.y,
                        x1: p1.x,
                        y1: page_height - p1.y,
                        line_width: state.line_width,
                        color: state.stroke_color,
                    });
                    current_x = x1;
                    current_y = y1;
                }
            }
            "re" => {
                // rectangle: x y width height
                if operands.len() >= 4 {
                    let x = operands[0].as_real().unwrap_or(0.0);
                    let y = operands[1].as_real().unwrap_or(0.0);
                    let w = operands[2].as_real().unwrap_or(0.0);
                    let h = operands[3].as_real().unwrap_or(0.0);
                    let p = state.ctm.transform_point(&Point::new(x, y));
                    let pw = state.ctm.transform_point(&Point::new(x + w, y));
                    let ph = state.ctm.transform_point(&Point::new(x, y + h));
                    // Simplified: assume no rotation in CTM for rect
                    let x0 = p.x.min(pw.x).min(ph.x);
                    let y0 = p.y.min(pw.y).min(ph.y);
                    let x1 = p.x.max(pw.x).max(ph.x);
                    let y1 = p.y.max(pw.y).max(ph.y);
                    rects.push(RectElement {
                        bbox: BBox::new(x0, page_height - y1, x1, page_height - y0),
                        line_width: state.line_width,
                        stroke_color: Some(state.stroke_color),
                        fill_color: Some(state.fill_color),
                    });
                }
            }

            // Ignore operators we don't need for text extraction
            _ => {}
        }
    }

    Ok(InterpretResult {
        chars,
        lines,
        rects,
    })
}

/// Inject a synthetic space character if there's a gap between the last emitted
/// character and the current text position. DISABLED — word spacing is handled
/// entirely by the layout analysis layer (chars_to_words) which has better spatial
/// heuristics for detecting word boundaries.
fn maybe_inject_space(
    _state: &GraphicsState,
    _font_cache: &FontCache,
    _page_height: f64,
    _chars: &mut Vec<Char>,
    _last_char_end_x: &mut Option<f64>,
    _last_char_center_y: &mut Option<f64>,
) {
    // Space injection between Tj/TJ calls is intentionally disabled.
    // The layout analysis layer (chars_to_words in grouping.rs) handles word
    // boundaries using spatial proximity of character bounding boxes, which is
    // more accurate than checking gaps at the text-showing operator level.
}

/// Show a text string, producing positioned Char elements.
#[allow(clippy::too_many_arguments)]
fn show_string(
    data: &[u8],
    state: &mut GraphicsState,
    font_cache: &FontCache,
    page_height: f64,
    chars: &mut Vec<Char>,
    last_char_end_x: &mut Option<f64>,
    last_char_center_y: &mut Option<f64>,
    run_id: u32,
) {
    let font = state
        .font_name
        .as_ref()
        .and_then(|name| font_cache.get(name));

    let mut i = 0;
    while i < data.len() {
        // Determine character code size based on font type.
        // For Type0 (composite) fonts the codes are typically 2 bytes
        // (Identity-H / CID mapping); for simple fonts they are 1 byte.
        let (code, advance) = if let Some(f) = font {
            match f.subtype {
                crate::text::fonts::FontSubtype::Type0 => {
                    // Composite fonts: try 2-byte first
                    if i + 1 < data.len() {
                        let code2 = ((data[i] as u32) << 8) | (data[i + 1] as u32);
                        (code2, 2)
                    } else {
                        (data[i] as u32, 1)
                    }
                }
                _ => (data[i] as u32, 1),
            }
        } else {
            (data[i] as u32, 1)
        };

        // Resolve the Unicode character using the priority chain:
        //   1. Font's ToUnicode CMap  (most accurate)
        //   2. Font's named encoding  (WinAnsiEncoding, MacRomanEncoding, etc.)
        //   3. Raw byte fallback       (direct codepoint or replacement char)
        let text_char = if let Some(f) = font {
            // 1. Try the resolved ToUnicode CMap
            if let Some(cmap) = f.cmap() {
                if let Some(ch) = cmap.to_char(code) {
                    ch
                } else if advance == 1 {
                    // CMap didn't have the code; try encoding for single-byte
                    resolve_via_encoding(code as u8, f.encoding.as_deref())
                } else {
                    raw_fallback(code)
                }
            } else if advance == 1 {
                // 2. No CMap but we have a single-byte code; try encoding
                resolve_via_encoding(code as u8, f.encoding.as_deref())
            } else {
                // 3. No CMap, multi-byte code; raw fallback
                raw_fallback(code)
            }
        } else {
            // No font at all
            raw_fallback(code)
        };

        // Calculate position from text matrix and CTM
        let text_pos = Point::new(state.text_matrix.e, state.text_matrix.f);
        let page_pos = state.ctm.transform_point(&text_pos);

        // Calculate glyph width for advancing text position
        let raw_width = font.map(|f| f.get_raw_width(code)).unwrap_or(600.0);
        let glyph_advance = (raw_width / 1000.0) * state.font_size;
        let scaling = state.horizontal_scaling / 100.0;

        // Transform width and height through text matrix for bbox
        let (dx, dy) = state
            .text_matrix
            .transform_vector(glyph_advance, state.font_size);
        let abs_dx = dx.abs();
        let abs_dy = dy.abs();

        // Build character bounding box in page coordinates (top-left origin)
        let page_x = page_pos.x;
        let page_y = page_pos.y;
        let bbox = BBox::new(
            page_x,
            page_height - page_y - abs_dy,
            page_x + abs_dx,
            page_height - page_y,
        );

        chars.push(Char {
            text: text_char.to_string(),
            bbox,
            font_name: state
                .font_name
                .as_ref()
                .and_then(|n| std::str::from_utf8(n).ok())
                .unwrap_or("unknown")
                .to_string(),
            font_size: state.font_size,
            bold: false,   // TODO: detect from font flags
            italic: false, // TODO: detect from font flags
            color: Some(state.fill_color),
            stroking_color: Some(state.stroke_color),
            rotation: 0.0, // TODO: calculate from text matrix
            run_id,
        });

        // Track last character position for space injection
        *last_char_end_x = Some(bbox.x1);
        *last_char_center_y = Some(bbox.center().1);

        // Advance text position
        let total_advance = glyph_advance * scaling + state.char_spacing;
        state.text_matrix.e += total_advance;

        // Word spacing for space character (only for single-byte codes)
        if advance == 1 && code == 0x20 {
            state.text_matrix.e += state.word_spacing;
        }

        i += advance;
    }
}

/// Try to resolve a single-byte character code through the named encoding.
fn resolve_via_encoding(byte: u8, encoding: Option<&str>) -> char {
    match encoding {
        Some(enc) => unicode::decode_char(byte, enc),
        None => {
            // No encoding specified; use WinAnsiEncoding as the default
            // for Latin-based fonts, or raw ASCII for low codes.
            if byte.is_ascii() {
                byte as char
            } else {
                unicode::decode_char(byte, "WinAnsiEncoding")
            }
        }
    }
}

/// Raw fallback when no CMap or encoding is available.
fn raw_fallback(code: u32) -> char {
    if code <= 0x7F {
        code as u8 as char
    } else {
        std::char::from_u32(code).unwrap_or('\u{FFFD}')
    }
}

/// Simple tokenizer for PDF content streams.
/// Splits input into operands + operator pairs.
struct ContentStreamTokenizer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> ContentStreamTokenizer<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    /// Parse the next operator and its operands.
    /// Returns None at end of input.
    fn next_operator(
        &mut self,
    ) -> Result<Option<(Vec<crate::parser::objects::PdfObject>, String)>> {
        use crate::parser::objects::PdfObject;

        let mut operands = Vec::new();

        loop {
            // Skip whitespace
            while self.pos < self.input.len() {
                let b = self.input.as_bytes()[self.pos];
                if !matches!(b, b' ' | b'\t' | b'\r' | b'\n' | b'\x0c') {
                    break;
                }
                self.pos += 1;
            }

            if self.pos >= self.input.len() {
                return Ok(None);
            }

            let remaining = &self.input[self.pos..];
            let bytes = remaining.as_bytes();

            // Comment
            if bytes[0] == b'%' {
                if let Some(newline_pos) = remaining.find(['\r', '\n']) {
                    self.pos += newline_pos + 1;
                } else {
                    self.pos = self.input.len();
                }
                continue;
            }

            // String literal
            if bytes[0] == b'(' {
                if let Some(end) = find_matching_paren(remaining) {
                    let s = &remaining[1..end];
                    self.pos += end + 1;
                    operands.push(PdfObject::String(s.as_bytes().to_vec()));
                    continue;
                }
            }

            // Hex string
            if bytes[0] == b'<' && bytes.len() > 1 && bytes[1] != b'<' {
                if let Some(end) = remaining.find('>') {
                    let hex_str = &remaining[1..end];
                    let decoded = decode_hex_string(hex_str);
                    self.pos += end + 1;
                    operands.push(PdfObject::String(decoded));
                    continue;
                }
            }

            // Array
            if bytes[0] == b'[' {
                // Find matching ]
                let mut depth = 1;
                let mut j = 1;
                while j < bytes.len() && depth > 0 {
                    if bytes[j] == b'[' {
                        depth += 1;
                    } else if bytes[j] == b']' {
                        depth -= 1;
                    } else if bytes[j] == b'(' {
                        // Skip string contents
                        j += 1;
                        while j < bytes.len() && bytes[j] != b')' {
                            if bytes[j] == b'\\' {
                                j += 1;
                            }
                            j += 1;
                        }
                    }
                    j += 1;
                }
                let array_str = &remaining[..j];
                self.pos += j;
                // Parse array contents recursively using ObjectParser
                let mut obj_parser =
                    crate::parser::objects::ObjectParser::new(array_str.as_bytes());
                if let Ok(obj) = obj_parser.parse_object() {
                    operands.push(obj);
                }
                continue;
            }

            // Name
            if bytes[0] == b'/' {
                let end = remaining[1..]
                    .find(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '.')
                    .map(|p| p + 1)
                    .unwrap_or(remaining.len());
                let name = &remaining[1..end];
                self.pos += end;
                operands.push(PdfObject::Name(name.as_bytes().to_vec()));
                continue;
            }

            // Number (integer or real)
            if bytes[0].is_ascii_digit() || bytes[0] == b'-' || bytes[0] == b'+' || bytes[0] == b'.'
            {
                let end = remaining
                    .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-' && c != '+')
                    .unwrap_or(remaining.len());
                let num_str = &remaining[..end];
                self.pos += end;
                if num_str.contains('.') {
                    if let Ok(f) = num_str.parse::<f64>() {
                        operands.push(PdfObject::Real(f));
                    }
                } else if let Ok(i) = num_str.parse::<i64>() {
                    operands.push(PdfObject::Integer(i));
                }
                continue;
            }

            // Operator keyword (alphabetic characters)
            if bytes[0].is_ascii_alphabetic()
                || bytes[0] == b'*'
                || bytes[0] == b'\''
                || bytes[0] == b'"'
            {
                let end = remaining
                    .find(|c: char| !c.is_ascii_alphabetic() && c != '*' && c != '\'' && c != '"')
                    .unwrap_or(remaining.len());
                if end == 0 {
                    self.pos += 1;
                    continue;
                }
                let op = &remaining[..end];
                self.pos += end;
                return Ok(Some((operands, op.to_string())));
            }

            // Unknown character, skip
            self.pos += 1;
        }
    }
}

fn find_matching_paren(s: &str) -> Option<usize> {
    let mut depth = 0;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            b'\\' => i += 1, // skip escaped char
            _ => {}
        }
        i += 1;
    }
    None
}

fn decode_hex_string(s: &str) -> Vec<u8> {
    let s: String = s.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    let mut result = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i + 1 < chars.len() {
        let byte = u8::from_str_radix(&format!("{}{}", chars[i], chars[i + 1]), 16).unwrap_or(0);
        result.push(byte);
        i += 2;
    }
    if i < chars.len() {
        result.push(u8::from_str_radix(&format!("{}0", chars[i]), 16).unwrap_or(0));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_text_extraction() {
        // BT /F1 12 Tf 100 700 Td (Hello) Tj ET
        let data = b"BT /F1 12 Tf 100 700 Td (Hello) Tj ET";
        let font_cache = FontCache::new();
        let result = interpret_content_stream(data, &font_cache, 792.0).unwrap();
        assert_eq!(result.chars.len(), 5);
        assert_eq!(result.chars[0].text, "H");
        assert_eq!(result.chars[4].text, "o");
    }

    #[test]
    fn test_text_matrix() {
        // BT 1 0 0 1 100 700 Tm /F1 12 Tf (A) Tj ET
        let data = b"BT 1 0 0 1 100 700 Tm /F1 12 Tf (A) Tj ET";
        let font_cache = FontCache::new();
        let result = interpret_content_stream(data, &font_cache, 792.0).unwrap();
        assert_eq!(result.chars.len(), 1);
        // The char should be positioned at x=100, y=700 in PDF coords
        // In top-left origin: y = 792 - 700 = 92
        assert!((result.chars[0].bbox.x0 - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_graphics_state_save_restore() {
        // BT /F1 12 Tf q /F2 14 Tf Q (A) Tj ET
        // After Q, font should be F1 again
        let data = b"BT /F1 12 Tf q /F2 14 Tf Q (A) Tj ET";
        let font_cache = FontCache::new();
        let result = interpret_content_stream(data, &font_cache, 792.0).unwrap();
        assert_eq!(result.chars.len(), 1);
        // Font should be F1 (size 12) after Q restores state
        assert!((result.chars[0].font_size - 12.0).abs() < 0.1);
    }
}
