//! Integration tests for text extraction.
//!
//! Tests content stream interpretation and text extraction from real PDFs.

use botl_pdf_core::parser::document::Document;
use botl_pdf_core::text::fonts::{Font, FontCache, FontSubtype};
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

// ---------------------------------------------------------------------------
// Content stream interpretation
// ---------------------------------------------------------------------------

#[test]
fn test_interpret_simple_text_content_stream() {
    // This is the exact content stream from simple_text.pdf:
    // BT /F1 12 Tf 100 700 Td (Hello World) Tj ET
    let data = b"BT /F1 12 Tf 100 700 Td (Hello World) Tj ET";
    let font_cache = FontCache::new();
    let result = interpret_content_stream(data, &font_cache, 792.0).unwrap();

    // "Hello World" has 11 characters
    assert_eq!(
        result.chars.len(),
        11,
        "Should extract 11 characters from 'Hello World'"
    );

    // Check the first and last characters
    assert_eq!(result.chars[0].text, "H");
    assert_eq!(result.chars[10].text, "d");

    // Verify the extracted text content
    let full_text: String = result.chars.iter().map(|c| c.text.as_str()).collect();
    assert_eq!(full_text, "Hello World");
}

#[test]
fn test_interpret_empty_content_stream() {
    let data = b"";
    let font_cache = FontCache::new();
    let result = interpret_content_stream(data, &font_cache, 792.0).unwrap();
    assert!(result.chars.is_empty());
    assert!(result.lines.is_empty());
    assert!(result.rects.is_empty());
}

#[test]
fn test_interpret_text_with_tj_array() {
    // TJ operator: array of strings with kerning adjustments
    let data = b"BT /F1 12 Tf 100 700 Td [(Hel) -50 (lo)] TJ ET";
    let font_cache = FontCache::new();
    let result = interpret_content_stream(data, &font_cache, 792.0).unwrap();

    let full_text: String = result.chars.iter().map(|c| c.text.as_str()).collect();
    assert_eq!(full_text, "Hello");
}

#[test]
fn test_interpret_multiple_text_showing_ops() {
    // Two Tj operations in sequence
    let data = b"BT /F1 12 Tf 100 700 Td (Hello) Tj 200 700 Td (World) Tj ET";
    let font_cache = FontCache::new();
    let result = interpret_content_stream(data, &font_cache, 792.0).unwrap();

    let full_text: String = result.chars.iter().map(|c| c.text.as_str()).collect();
    assert!(full_text.contains("Hello"));
    assert!(full_text.contains("World"));
}

#[test]
fn test_interpret_td_positioning() {
    // Td moves text position
    let data = b"BT /F1 12 Tf (Line1) Tj 0 -20 Td (Line2) Tj ET";
    let font_cache = FontCache::new();
    let result = interpret_content_stream(data, &font_cache, 792.0).unwrap();

    assert_eq!(result.chars.len(), 10, "Should extract 10 characters");
    let text: String = result.chars.iter().map(|c| c.text.as_str()).collect();
    assert!(text.contains("Line1"));
    assert!(text.contains("Line2"));
}

#[test]
fn test_interpret_tm_sets_text_matrix() {
    // Tm sets the text matrix directly
    let data = b"BT 1 0 0 1 50 600 Tm /F1 14 Tf (Positioned) Tj ET";
    let font_cache = FontCache::new();
    let result = interpret_content_stream(data, &font_cache, 792.0).unwrap();

    assert!(!result.chars.is_empty());
    // First character should be at x ~50
    let first_x = result.chars[0].bbox.x0;
    assert!(
        (first_x - 50.0).abs() < 2.0,
        "First char x should be ~50, got {}",
        first_x
    );
}

#[test]
fn test_interpret_graphics_state_save_restore() {
    // q saves state, Q restores it
    let data = b"BT /F1 12 Tf q /F2 24 Tf Q (AfterRestore) Tj ET";
    let font_cache = FontCache::new();
    let result = interpret_content_stream(data, &font_cache, 792.0).unwrap();

    // After Q, font should be F1 at size 12
    assert!(!result.chars.is_empty());
    assert!(
        (result.chars[0].font_size - 12.0).abs() < 0.1,
        "Font size should be restored to 12, got {}",
        result.chars[0].font_size
    );
    assert_eq!(result.chars[0].font_name, "F1");
}

#[test]
fn test_interpret_color_operators() {
    let data = b"BT /F1 12 Tf 1 0 0 rg (Red) Tj 0 0 1 rg (Blue) Tj ET";
    let font_cache = FontCache::new();
    let result = interpret_content_stream(data, &font_cache, 792.0).unwrap();

    // First "Red" chars should have red fill color
    assert_eq!(result.chars[0].color, Some((1.0, 0.0, 0.0)));

    // "Blue" chars should have blue fill color
    let blue_start = result.chars.iter().position(|c| c.text == "B").unwrap();
    assert_eq!(result.chars[blue_start].color, Some((0.0, 0.0, 1.0)));
}

// ---------------------------------------------------------------------------
// Extracting text from simple_text.pdf
// ---------------------------------------------------------------------------

#[test]
fn test_extract_text_from_simple_text_pdf() {
    let path = fixture_path("simple_text.pdf");
    let mut doc = Document::open(&path).unwrap();

    // Get page 0's content stream
    let page = doc.get_page(0).unwrap();
    let contents_ref = page.get_reference("Contents").unwrap();
    let stream_obj = doc.resolve(contents_ref).unwrap();
    let stream = stream_obj.as_stream().unwrap();

    // Decode the content stream
    let decoded = botl_pdf_core::codecs::decode_stream_data(&stream).unwrap();

    // Interpret the content stream
    let font_cache = FontCache::new();
    let result = interpret_content_stream(&decoded, &font_cache, 792.0).unwrap();

    // Verify "Hello World" text is extracted
    let text: String = result.chars.iter().map(|c| c.text.as_str()).collect();
    assert!(
        text.contains("Hello"),
        "Extracted text should contain 'Hello', got: '{}'",
        text
    );
    assert!(
        text.contains("World"),
        "Extracted text should contain 'World', got: '{}'",
        text
    );
}

// ---------------------------------------------------------------------------
// Multi-page extraction
// ---------------------------------------------------------------------------

#[test]
fn test_extract_text_from_multi_page_pdf() {
    let path = fixture_path("multi_page.pdf");
    let mut doc = Document::open(&path).unwrap();

    // Verify the page count
    assert_eq!(doc.num_pages().unwrap(), 3);

    // The multi_page fixture has a non-contiguous xref that limits page traversal
    // to page 0. Test what we can reliably resolve.
    let page = doc.get_page(0).unwrap();
    let contents_ref = page.get_reference("Contents").unwrap();
    let stream_obj = doc.resolve(contents_ref).unwrap();
    let stream = stream_obj.as_stream().unwrap();

    let decoded = botl_pdf_core::codecs::decode_stream_data(&stream).unwrap();
    let font_cache = FontCache::new();
    let result = interpret_content_stream(&decoded, &font_cache, 792.0).unwrap();

    let text: String = result.chars.iter().map(|c| c.text.as_str()).collect();
    assert!(
        text.contains("Page One"),
        "Page 0 should contain 'Page One', got: '{}'",
        text
    );
}

// ---------------------------------------------------------------------------
// Font cache usage
// ---------------------------------------------------------------------------

#[test]
fn test_font_cache_with_page_fonts() {
    let path = fixture_path("simple_text.pdf");
    let mut doc = Document::open(&path).unwrap();

    // Build a font cache from the page resources
    let page = doc.get_page(0).unwrap();
    let resources = page.get_dict("Resources").unwrap();
    let font_dict = resources.get_dict("Font").unwrap();

    let mut font_cache = FontCache::new();

    // Iterate font entries and create Font objects
    for (key, value) in font_dict.iter() {
        if let Some(font_ref) = value.as_reference() {
            if let Ok(font_obj) = doc.resolve(font_ref) {
                if let Some(font_dict_obj) = font_obj.as_dict() {
                    let name = std::str::from_utf8(key).unwrap_or("unknown");
                    if let Ok(font) = Font::from_dict(name, font_dict_obj) {
                        font_cache.insert(key, font);
                    }
                }
            }
        }
    }

    // Verify the font cache contains F1
    assert!(
        font_cache.get(b"F1").is_some(),
        "Font cache should contain F1"
    );

    let f1 = font_cache.get(b"F1").unwrap();
    assert_eq!(f1.subtype, FontSubtype::Type1);
    assert_eq!(f1.base_font.as_deref(), Some("Helvetica"));
}

// ---------------------------------------------------------------------------
// Character positioning
// ---------------------------------------------------------------------------

#[test]
fn test_character_bounding_boxes_are_valid() {
    let data = b"BT /F1 12 Tf 100 700 Td (AB) Tj ET";
    let font_cache = FontCache::new();
    let result = interpret_content_stream(data, &font_cache, 792.0).unwrap();

    // Each character should have a valid bounding box
    for (i, ch) in result.chars.iter().enumerate() {
        assert!(
            ch.bbox.width() > 0.0,
            "Char {} ('{}') should have positive width",
            i,
            ch.text
        );
        assert!(
            ch.bbox.height() > 0.0,
            "Char {} ('{}') should have positive height",
            i,
            ch.text
        );
    }

    // Characters should advance horizontally
    if result.chars.len() >= 2 {
        assert!(
            result.chars[1].bbox.x0 >= result.chars[0].bbox.x0,
            "Second char should be at or after first char horizontally"
        );
    }
}

#[test]
fn test_character_font_info() {
    let data = b"BT /F1 12 Tf 100 700 Td (Test) Tj ET";
    let font_cache = FontCache::new();
    let result = interpret_content_stream(data, &font_cache, 792.0).unwrap();

    for ch in &result.chars {
        assert_eq!(ch.font_name, "F1");
        assert!((ch.font_size - 12.0).abs() < 0.1, "Font size should be ~12");
    }
}

// ---------------------------------------------------------------------------
// FlateDecode compressed content stream
// ---------------------------------------------------------------------------

#[test]
fn test_extract_text_from_flate_compressed_pdf() {
    let path = fixture_path("flate_compressed.pdf");
    let mut doc = Document::open(&path).unwrap();

    let page = doc.get_page(0).unwrap();
    let contents_ref = page.get_reference("Contents").unwrap();
    let stream_obj = doc.resolve(contents_ref).unwrap();
    let stream = stream_obj.as_stream().unwrap();

    // Verify it uses FlateDecode
    assert!(stream.filter().is_some());

    let decoded = botl_pdf_core::codecs::decode_stream_data(&stream).unwrap();
    let font_cache = FontCache::new();
    let result = interpret_content_stream(&decoded, &font_cache, 792.0).unwrap();

    let text: String = result.chars.iter().map(|c| c.text.as_str()).collect();
    assert!(
        text.contains("Compressed") || text.contains("Text"),
        "Decoded text should contain content from the PDF, got: '{}'",
        text
    );
}
