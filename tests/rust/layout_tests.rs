//! Integration tests for layout analysis.
//!
//! Tests the layout pipeline: characters -> words -> lines -> blocks.

use botl_pdf_core::geometry::BBox;
use botl_pdf_core::layout::elements::{Char, TextBlock, TextLine, Word};
use botl_pdf_core::layout::grouping::{chars_to_words, lines_to_blocks, words_to_lines};
use botl_pdf_core::layout::ordering::{sort_blocks_reading_order, sort_lines_in_block, sort_words_in_line};
use botl_pdf_core::layout::strategy::{analyze_layout, blocks_to_text, blocks_to_layout_text, LayoutParams};
use botl_pdf_core::parser::document::Document;
use botl_pdf_core::text::fonts::FontCache;
use botl_pdf_core::text::operator::interpret_content_stream;

/// Helper to get the absolute path to a fixture file.
fn fixture_path(name: &str) -> std::path::PathBuf {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.push("tests");
    path.push("fixtures");
    path.push(name);
    path
}

/// Helper to create a character at a given position.
fn make_char(text: &str, x0: f64, y0: f64, x1: f64, y1: f64) -> Char {
    Char {
        text: text.to_string(),
        bbox: BBox::new(x0, y0, x1, y1),
        font_name: "F1".to_string(),
        font_size: 12.0,
        bold: false,
        italic: false,
        color: Some((0.0, 0.0, 0.0)),
        stroking_color: None,
        rotation: 0.0,
        run_id: 0,
    }
}

/// Helper to create a character with a specific font size.
fn make_char_with_size(text: &str, x0: f64, y0: f64, x1: f64, y1: f64, font_size: f64) -> Char {
    Char {
        text: text.to_string(),
        bbox: BBox::new(x0, y0, x1, y1),
        font_name: "F1".to_string(),
        font_size,
        bold: false,
        italic: false,
        color: Some((0.0, 0.0, 0.0)),
        stroking_color: None,
        rotation: 0.0,
        run_id: 0,
    }
}

// ===========================================================================
// chars_to_words tests
// ===========================================================================

#[test]
fn test_chars_to_words_single_word() {
    // "Hi" -- two characters adjacent
    let chars = vec![
        make_char("H", 0.0, 0.0, 8.0, 12.0),
        make_char("i", 8.0, 0.0, 13.0, 12.0),
    ];
    let words = chars_to_words(&chars, 0.1);
    assert_eq!(words.len(), 1, "Adjacent chars should form one word");
    assert_eq!(words[0].text, "Hi");
}

#[test]
fn test_chars_to_words_two_words_with_gap() {
    // "He" then a space, then "wo" — gap-based break requires a space character
    let chars = vec![
        make_char("H", 0.0, 0.0, 8.0, 12.0),
        make_char("e", 8.0, 0.0, 15.0, 12.0),
        make_char(" ", 15.0, 0.0, 20.0, 12.0), // space triggers word break
        make_char("w", 30.0, 0.0, 38.0, 12.0),
        make_char("o", 38.0, 0.0, 45.0, 12.0),
    ];
    let words = chars_to_words(&chars, 0.3);
    assert_eq!(words.len(), 2, "Space should separate words");
    assert_eq!(words[0].text, "He");
    assert_eq!(words[1].text, "wo");
}

#[test]
fn test_chars_to_words_empty_input() {
    let words = chars_to_words(&[], 0.1);
    assert!(words.is_empty());
}

#[test]
fn test_chars_to_words_single_char() {
    let chars = vec![make_char("A", 0.0, 0.0, 8.0, 12.0)];
    let words = chars_to_words(&chars, 0.1);
    assert_eq!(words.len(), 1);
    assert_eq!(words[0].text, "A");
}

#[test]
fn test_chars_to_words_break_on_font_change() {
    let mut char1 = make_char("A", 0.0, 0.0, 8.0, 12.0);
    char1.font_name = "F1".to_string();
    let mut char2 = make_char("B", 8.0, 0.0, 16.0, 12.0);
    char2.font_name = "F2".to_string();

    let words = chars_to_words(&[char1, char2], 0.1);
    assert_eq!(words.len(), 2, "Different fonts should create separate words");
}

#[test]
fn test_chars_to_words_break_on_font_size_change() {
    let char1 = make_char_with_size("A", 0.0, 0.0, 8.0, 12.0, 12.0);
    let char2 = make_char_with_size("B", 8.0, 0.0, 16.0, 24.0, 20.0);

    let words = chars_to_words(&[char1, char2], 0.1);
    assert_eq!(words.len(), 2, "Large font size change should create separate words");
}

#[test]
fn test_chars_to_words_break_on_vertical_displacement() {
    let chars = vec![
        make_char("A", 0.0, 0.0, 8.0, 12.0),
        make_char("B", 8.0, 30.0, 16.0, 42.0), // far below
    ];
    let words = chars_to_words(&chars, 0.1);
    assert_eq!(words.len(), 2, "Vertical displacement should create separate words");
}

#[test]
fn test_chars_to_words_space_character_breaks() {
    let chars = vec![
        make_char("A", 0.0, 0.0, 8.0, 12.0),
        make_char(" ", 8.0, 0.0, 14.0, 12.0),
        make_char("B", 14.0, 0.0, 22.0, 12.0),
    ];
    let words = chars_to_words(&chars, 0.1);
    // Space causes a word break when the NEXT char after space is processed.
    // The space itself is included in the preceding word: "A " and "B".
    assert_eq!(words.len(), 2, "Space should separate words");
    assert!(words[0].text.starts_with("A"), "First word should start with A");
    assert_eq!(words[1].text, "B");
}

#[test]
fn test_chars_to_words_word_bboxes_enclose_chars() {
    let chars = vec![
        make_char("H", 10.0, 5.0, 18.0, 17.0),
        make_char("i", 18.0, 5.0, 23.0, 17.0),
    ];
    let words = chars_to_words(&chars, 0.1);
    assert_eq!(words.len(), 1);

    let word = &words[0];
    assert!((word.bbox.x0 - 10.0).abs() < f64::EPSILON);
    assert!((word.bbox.y0 - 5.0).abs() < f64::EPSILON);
    assert!((word.bbox.x1 - 23.0).abs() < f64::EPSILON);
    assert!((word.bbox.y1 - 17.0).abs() < f64::EPSILON);
}

// ===========================================================================
// words_to_lines tests
// ===========================================================================

#[test]
fn test_words_to_lines_single_line() {
    let chars1 = vec![make_char("H", 0.0, 0.0, 8.0, 12.0)];
    let chars2 = vec![make_char("i", 10.0, 0.0, 18.0, 12.0)];
    let w1 = Word::from_chars(chars1).unwrap();
    let w2 = Word::from_chars(chars2).unwrap();

    let lines = words_to_lines(&[w1, w2]);
    assert_eq!(lines.len(), 1, "Words at same y should form one line");
    assert_eq!(lines[0].text, "H i");
}

#[test]
fn test_words_to_lines_two_lines() {
    let chars1 = vec![make_char("a", 0.0, 0.0, 10.0, 12.0)];
    let chars2 = vec![make_char("b", 0.0, 20.0, 10.0, 32.0)];
    let w1 = Word::from_chars(chars1).unwrap();
    let w2 = Word::from_chars(chars2).unwrap();

    let lines = words_to_lines(&[w1, w2]);
    assert_eq!(lines.len(), 2, "Words at different y positions should form different lines");
}

#[test]
fn test_words_to_lines_empty() {
    let lines = words_to_lines(&[]);
    assert!(lines.is_empty());
}

#[test]
fn test_words_to_lines_preserves_input_order() {
    // words_to_lines preserves input order (sorting is done upstream at char level)
    let chars1 = vec![make_char("B", 20.0, 0.0, 30.0, 12.0)];
    let chars2 = vec![make_char("A", 0.0, 0.0, 10.0, 12.0)];
    let w1 = Word::from_chars(chars1).unwrap();
    let w2 = Word::from_chars(chars2).unwrap();

    let lines = words_to_lines(&[w1, w2]);
    assert_eq!(lines.len(), 1);
    // Input order is preserved: "B A"
    assert_eq!(lines[0].text, "B A");
}

#[test]
fn test_words_to_lines_line_bbox_encloses_words() {
    let chars1 = vec![make_char("H", 10.0, 5.0, 18.0, 17.0)];
    let chars2 = vec![make_char("i", 25.0, 5.0, 30.0, 17.0)];
    let w1 = Word::from_chars(chars1).unwrap();
    let w2 = Word::from_chars(chars2).unwrap();

    let lines = words_to_lines(&[w1, w2]);
    assert_eq!(lines.len(), 1);

    let line = &lines[0];
    assert!((line.bbox.x0 - 10.0).abs() < f64::EPSILON);
    assert!((line.bbox.y0 - 5.0).abs() < f64::EPSILON);
    assert!((line.bbox.x1 - 30.0).abs() < f64::EPSILON);
    assert!((line.bbox.y1 - 17.0).abs() < f64::EPSILON);
}

// ===========================================================================
// lines_to_blocks tests
// ===========================================================================

#[test]
fn test_lines_to_blocks_close_lines_same_block() {
    let chars1 = vec![make_char("a", 0.0, 0.0, 10.0, 12.0)];
    let chars2 = vec![make_char("b", 0.0, 16.0, 10.0, 28.0)];
    let w1 = Word::from_chars(chars1).unwrap();
    let w2 = Word::from_chars(chars2).unwrap();
    let l1 = TextLine::from_words(vec![w1]).unwrap();
    let l2 = TextLine::from_words(vec![w2]).unwrap();

    // gap = 4, threshold = 0.5 * 12 = 6, 4 < 6 so same block
    let blocks = lines_to_blocks(&[l1, l2], 0.5);
    assert_eq!(blocks.len(), 1, "Close lines should be in the same block");
    assert!(blocks[0].text.contains("a"));
    assert!(blocks[0].text.contains("b"));
}

#[test]
fn test_lines_to_blocks_far_lines_different_blocks() {
    let chars1 = vec![make_char("a", 0.0, 0.0, 10.0, 12.0)];
    let chars2 = vec![make_char("b", 0.0, 50.0, 10.0, 62.0)];
    let w1 = Word::from_chars(chars1).unwrap();
    let w2 = Word::from_chars(chars2).unwrap();
    let l1 = TextLine::from_words(vec![w1]).unwrap();
    let l2 = TextLine::from_words(vec![w2]).unwrap();

    // gap = 38, threshold = 0.5 * 12 = 6, 38 > 6 so different blocks
    let blocks = lines_to_blocks(&[l1, l2], 0.5);
    assert_eq!(blocks.len(), 2, "Far-apart lines should be in different blocks");
}

#[test]
fn test_lines_to_blocks_empty() {
    let blocks = lines_to_blocks(&[], 0.5);
    assert!(blocks.is_empty());
}

// ===========================================================================
// Full layout pipeline (analyze_layout)
// ===========================================================================

#[test]
fn test_analyze_layout_basic() {
    let chars = vec![
        make_char("H", 0.0, 0.0, 8.0, 12.0),
        make_char("i", 8.0, 0.0, 13.0, 12.0),
    ];
    let params = LayoutParams::default();
    let blocks = analyze_layout(&chars, &params);

    assert!(!blocks.is_empty(), "Should produce at least one block");
    assert!(blocks[0].text.contains("Hi"), "Block text should contain 'Hi'");
}

#[test]
fn test_analyze_layout_empty() {
    let params = LayoutParams::default();
    let blocks = analyze_layout(&[], &params);
    assert!(blocks.is_empty());
}

#[test]
fn test_analyze_layout_two_words_one_line() {
    let chars = vec![
        make_char("H", 0.0, 0.0, 8.0, 12.0),
        make_char("e", 8.0, 0.0, 15.0, 12.0),
        make_char("l", 15.0, 0.0, 20.0, 12.0),
        make_char("l", 20.0, 0.0, 25.0, 12.0),
        make_char("o", 25.0, 0.0, 32.0, 12.0),
        // Gap to create a new word
        make_char("W", 45.0, 0.0, 53.0, 12.0),
        make_char("o", 53.0, 0.0, 60.0, 12.0),
        make_char("r", 60.0, 0.0, 65.0, 12.0),
        make_char("l", 65.0, 0.0, 70.0, 12.0),
        make_char("d", 70.0, 0.0, 77.0, 12.0),
    ];
    let params = LayoutParams::default();
    let blocks = analyze_layout(&chars, &params);

    assert!(!blocks.is_empty());
    let text = &blocks[0].text;
    assert!(text.contains("Hello"), "Should contain 'Hello'");
    assert!(text.contains("World"), "Should contain 'World'");
}

#[test]
fn test_analyze_layout_two_lines() {
    let chars = vec![
        make_char("A", 0.0, 0.0, 8.0, 12.0),
        make_char("B", 0.0, 20.0, 8.0, 32.0),
    ];
    let params = LayoutParams::default();
    let blocks = analyze_layout(&chars, &params);

    // These are on different lines; depending on line_margin, may be one or two blocks
    assert!(!blocks.is_empty());
    let all_text: String = blocks.iter().map(|b| b.text.as_str()).collect();
    assert!(all_text.contains("A"));
    assert!(all_text.contains("B"));
}

// ===========================================================================
// blocks_to_text and blocks_to_layout_text
// ===========================================================================

#[test]
fn test_blocks_to_text() {
    let chars = vec![
        make_char("a", 0.0, 0.0, 8.0, 12.0),
        make_char("b", 20.0, 0.0, 28.0, 12.0),
    ];
    let params = LayoutParams::default();
    let blocks = analyze_layout(&chars, &params);
    let text = blocks_to_text(&blocks);
    assert!(!text.is_empty());
}

#[test]
fn test_blocks_to_layout_text() {
    let chars = vec![
        make_char("X", 0.0, 0.0, 8.0, 12.0),
    ];
    let params = LayoutParams::default();
    let blocks = analyze_layout(&chars, &params);
    let text = blocks_to_layout_text(&blocks, 612.0);
    assert!(text.contains("X"));
}

// ===========================================================================
// Reading order
// ===========================================================================

fn make_block(x0: f64, y0: f64, x1: f64, y1: f64, text: &str) -> TextBlock {
    let ch = Char {
        text: text.to_string(),
        bbox: BBox::new(x0, y0, x1, y1),
        font_name: "F1".to_string(),
        font_size: 12.0,
        bold: false,
        italic: false,
        color: None,
        stroking_color: None,
        rotation: 0.0,
        run_id: 0,
    };
    let word = Word::from_chars(vec![ch]).unwrap();
    let line = TextLine::from_words(vec![word]).unwrap();
    TextBlock::from_lines(vec![line]).unwrap()
}

#[test]
fn test_reading_order_top_to_bottom() {
    let mut blocks = vec![
        make_block(0.0, 100.0, 200.0, 112.0, "second"),
        make_block(0.0, 0.0, 200.0, 12.0, "first"),
    ];
    sort_blocks_reading_order(&mut blocks, 0.5);
    assert_eq!(blocks[0].text, "first");
    assert_eq!(blocks[1].text, "second");
}

#[test]
fn test_reading_order_left_to_right() {
    let mut blocks = vec![
        make_block(200.0, 0.0, 400.0, 12.0, "right"),
        make_block(0.0, 0.0, 180.0, 12.0, "left"),
    ];
    sort_blocks_reading_order(&mut blocks, 0.5);
    // Both at the same y; the one with smaller x should come first
    assert_eq!(blocks[0].text, "left");
}

// ===========================================================================
// Sort words / lines helpers
// ===========================================================================

#[test]
fn test_sort_words_in_line() {
    let chars = vec![
        make_char("B", 20.0, 0.0, 30.0, 12.0),
        make_char("A", 0.0, 0.0, 10.0, 12.0),
        make_char("C", 40.0, 0.0, 50.0, 12.0),
    ];
    let mut words: Vec<Word> = chars
        .into_iter()
        .map(|c| Word::from_chars(vec![c]).unwrap())
        .collect();

    sort_words_in_line(&mut words);
    assert_eq!(words[0].text, "A");
    assert_eq!(words[1].text, "B");
    assert_eq!(words[2].text, "C");
}

#[test]
fn test_sort_lines_in_block() {
    let chars1 = vec![make_char("B", 0.0, 20.0, 10.0, 32.0)];
    let chars2 = vec![make_char("A", 0.0, 0.0, 10.0, 12.0)];
    let w1 = Word::from_chars(chars1).unwrap();
    let w2 = Word::from_chars(chars2).unwrap();
    let l1 = TextLine::from_words(vec![w1]).unwrap();
    let l2 = TextLine::from_words(vec![w2]).unwrap();

    let mut lines = vec![l1, l2];
    sort_lines_in_block(&mut lines);
    assert_eq!(lines[0].text, "A");
    assert_eq!(lines[1].text, "B");
}

// ===========================================================================
// Full pipeline on simple_text.pdf
// ===========================================================================

#[test]
fn test_layout_pipeline_on_simple_text_pdf() {
    let path = fixture_path("simple_text.pdf");
    let mut doc = Document::open(&path).unwrap();

    // Get the content stream
    let page = doc.get_page(0).unwrap();
    let contents_ref = page.get_reference("Contents").unwrap();
    let stream_obj = doc.resolve(contents_ref).unwrap();
    let stream = stream_obj.as_stream().unwrap();
    let decoded = botl_pdf_core::codecs::decode_stream_data(&stream).unwrap();

    // Interpret the content stream to extract characters
    let font_cache = FontCache::new();
    let result = interpret_content_stream(&decoded, &font_cache, 792.0).unwrap();

    assert!(!result.chars.is_empty(), "Should extract characters");

    // Run layout analysis
    let params = LayoutParams::default();
    let blocks = analyze_layout(&result.chars, &params);

    assert!(!blocks.is_empty(), "Should produce at least one block");

    let text = blocks_to_text(&blocks);
    assert!(
        text.contains("Hello") || text.contains("World"),
        "Layout text should contain 'Hello' or 'World', got: '{}'",
        text
    );
}
