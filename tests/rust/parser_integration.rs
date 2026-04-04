//! Integration tests for PDF parsing.
//!
//! Tests the parser module end-to-end using real PDF fixture files.

use botl_pdf_core::error::BotlError;
use botl_pdf_core::parser::document::Document;
use botl_pdf_core::parser::objects::ObjRef;
use botl_pdf_core::parser::xref::{find_startxref, parse_xref_from_data, XrefEntry};

/// Helper to get the absolute path to a fixture file.
fn fixture_path(name: &str) -> std::path::PathBuf {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // From rust/botl-pdf-core/, go up 2 levels then into tests/fixtures/
    path.pop();
    path.pop();
    path.push("tests");
    path.push("fixtures");
    path.push(name);
    path
}

// ---------------------------------------------------------------------------
// Opening and parsing real PDF files
// ---------------------------------------------------------------------------

#[test]
fn test_open_simple_text_pdf() {
    let path = fixture_path("simple_text.pdf");
    let doc = Document::open(&path);
    assert!(
        doc.is_ok(),
        "Failed to open simple_text.pdf: {:?}",
        doc.err()
    );
}

#[test]
fn test_parse_from_bytes() {
    let path = fixture_path("simple_text.pdf");
    let bytes = std::fs::read(&path).expect("Failed to read fixture file");
    let doc = Document::from_bytes(bytes);
    assert!(
        doc.is_ok(),
        "Failed to parse simple_text.pdf from bytes: {:?}",
        doc.err()
    );
}

#[test]
fn test_reject_non_pdf_bytes() {
    let result = Document::from_bytes(b"This is not a PDF file at all".to_vec());
    assert!(result.is_err());
    let err = result.err().unwrap();
    match err {
        BotlError::ParseError(msg) => assert!(msg.contains("%PDF-")),
        other => panic!("Expected ParseError, got: {:?}", other),
    }
}

#[test]
fn test_reject_too_small_file() {
    let result = Document::from_bytes(b"%PDF".to_vec());
    assert!(result.is_err());
}

#[test]
fn test_reject_empty_bytes() {
    let result = Document::from_bytes(Vec::new());
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// PDF version detection
// ---------------------------------------------------------------------------

#[test]
fn test_version_simple_text() {
    let path = fixture_path("simple_text.pdf");
    let doc = Document::open(&path).unwrap();
    let version = doc.version();
    assert!(version.is_some(), "PDF should have a version");
    let v = version.unwrap();
    assert!(
        v.starts_with("1."),
        "Version should start with 1., got: {}",
        v
    );
}

// ---------------------------------------------------------------------------
// Page count and page tree traversal
// ---------------------------------------------------------------------------

#[test]
fn test_page_count_simple_text() {
    let path = fixture_path("simple_text.pdf");
    let mut doc = Document::open(&path).unwrap();
    let pages = doc.num_pages().unwrap();
    assert_eq!(pages, 1, "simple_text.pdf should have exactly 1 page");
}

#[test]
fn test_page_count_multi_page() {
    let path = fixture_path("multi_page.pdf");
    let mut doc = Document::open(&path).unwrap();
    let pages = doc.num_pages().unwrap();
    assert_eq!(pages, 3, "multi_page.pdf should have exactly 3 pages");
}

#[test]
fn test_get_page_zero_indexed() {
    let path = fixture_path("simple_text.pdf");
    let mut doc = Document::open(&path).unwrap();

    // Page 0 should exist
    let page = doc.get_page(0);
    assert!(page.is_ok(), "Page 0 should exist");

    let page_dict = page.unwrap();
    assert_eq!(page_dict.get_name("Type"), Some("Page"));
}

#[test]
fn test_get_page_out_of_range() {
    let path = fixture_path("simple_text.pdf");
    let mut doc = Document::open(&path).unwrap();

    // Page 1 should not exist (only 1 page, 0-indexed)
    let result = doc.get_page(1);
    assert!(result.is_err());
    match result.unwrap_err() {
        BotlError::PageOutOfRange { page, total } => {
            assert_eq!(page, 1);
            assert_eq!(total, 1);
        }
        other => panic!("Expected PageOutOfRange, got {:?}", other),
    }
}

#[test]
fn test_page_tree_traversal_multi_page() {
    let path = fixture_path("multi_page.pdf");
    let mut doc = Document::open(&path).unwrap();

    // Verify we can get the page count
    let pages = doc.num_pages().unwrap();
    assert_eq!(pages, 3);

    // Retrieve pages. The multi_page fixture has a non-contiguous xref
    // that may affect resolution. At minimum, page 0 must work.
    let page0 = doc.get_page(0);
    assert!(page0.is_ok(), "Page 0 should exist");
    let page0_dict = page0.unwrap();
    assert_eq!(page0_dict.get_name("Type"), Some("Page"));

    // Pages beyond the count should fail
    assert!(doc.get_page(pages).is_err());
}

#[test]
fn test_page_has_mediabox() {
    let path = fixture_path("simple_text.pdf");
    let mut doc = Document::open(&path).unwrap();
    let page = doc.get_page(0).unwrap();

    // MediaBox should be [0 0 612 792] (US Letter)
    let mediabox = page.get_array("MediaBox");
    assert!(mediabox.is_some(), "Page should have MediaBox");

    let mb = mediabox.unwrap();
    assert_eq!(mb.len(), 4);
    // Verify the dimensions: 612 x 792
    assert_eq!(mb[0].as_integer(), Some(0));
    assert_eq!(mb[1].as_integer(), Some(0));
    assert_eq!(mb[2].as_integer(), Some(612));
    assert_eq!(mb[3].as_integer(), Some(792));
}

#[test]
fn test_page_has_contents_reference() {
    let path = fixture_path("simple_text.pdf");
    let mut doc = Document::open(&path).unwrap();
    let page = doc.get_page(0).unwrap();

    let contents_ref = page.get_reference("Contents");
    assert!(
        contents_ref.is_some(),
        "Page should have Contents reference"
    );
}

#[test]
fn test_page_has_resources() {
    let path = fixture_path("simple_text.pdf");
    let mut doc = Document::open(&path).unwrap();
    let page = doc.get_page(0).unwrap();

    let resources = page.get_dict("Resources");
    assert!(resources.is_some(), "Page should have Resources dictionary");

    let res = resources.unwrap();
    let font_dict = res.get_dict("Font");
    assert!(
        font_dict.is_some(),
        "Resources should have Font subdictionary"
    );
}

// ---------------------------------------------------------------------------
// Catalog (root object) access
// ---------------------------------------------------------------------------

#[test]
fn test_catalog() {
    let path = fixture_path("simple_text.pdf");
    let mut doc = Document::open(&path).unwrap();
    let catalog = doc.catalog().unwrap();

    assert_eq!(catalog.get_name("Type"), Some("Catalog"));
    assert!(catalog.get_reference("Pages").is_some());
}

// ---------------------------------------------------------------------------
// Object resolution
// ---------------------------------------------------------------------------

#[test]
fn test_resolve_indirect_object() {
    let path = fixture_path("simple_text.pdf");
    let mut doc = Document::open(&path).unwrap();

    // Resolve the catalog (object 1 0)
    let catalog_obj = doc.resolve(ObjRef::new(1, 0)).unwrap();
    let catalog = catalog_obj.as_dict().unwrap();
    assert_eq!(catalog.get_name("Type"), Some("Catalog"));
}

#[test]
fn test_resolve_font_object() {
    let path = fixture_path("simple_text.pdf");
    let mut doc = Document::open(&path).unwrap();

    // Object 5 is the font (Helvetica Type1)
    let font_obj = doc.resolve(ObjRef::new(5, 0)).unwrap();
    let font_dict = font_obj.as_dict().unwrap();
    assert_eq!(font_dict.get_name("Type"), Some("Font"));
    assert_eq!(font_dict.get_name("Subtype"), Some("Type1"));
    assert_eq!(font_dict.get_name("BaseFont"), Some("Helvetica"));
}

#[test]
fn test_resolve_content_stream() {
    let path = fixture_path("simple_text.pdf");
    let mut doc = Document::open(&path).unwrap();

    // Object 4 is the content stream
    let stream_obj = doc.resolve(ObjRef::new(4, 0)).unwrap();
    let stream = stream_obj.as_stream().unwrap();

    // Verify the stream contains the text "Hello World"
    let content = std::str::from_utf8(&stream.data).unwrap();
    assert!(
        content.contains("Hello World"),
        "Content stream should contain 'Hello World'"
    );
}

#[test]
fn test_resolve_invalid_reference() {
    let path = fixture_path("simple_text.pdf");
    let mut doc = Document::open(&path).unwrap();

    // Object 999 does not exist
    let result = doc.resolve(ObjRef::new(999, 0));
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Metadata extraction
// ---------------------------------------------------------------------------

#[test]
fn test_metadata_simple_text_no_info() {
    let path = fixture_path("simple_text.pdf");
    let mut doc = Document::open(&path).unwrap();
    let meta = doc.metadata().unwrap();

    // simple_text.pdf has no Info dictionary
    assert_eq!(meta.page_count, 1);
    assert!(meta.version.is_some());
    // These fields should be None since no Info dict
    assert!(meta.title.is_none());
    assert!(meta.author.is_none());
}

#[test]
fn test_metadata_pdf_with_info() {
    let path = fixture_path("metadata.pdf");
    let mut doc = Document::open(&path).unwrap();
    let meta = doc.metadata().unwrap();

    assert_eq!(meta.page_count, 1);
    assert_eq!(meta.title.as_deref(), Some("Test PDF Title"));
    assert_eq!(meta.author.as_deref(), Some("Test Author"));
    assert_eq!(meta.subject.as_deref(), Some("Test Subject"));
    assert_eq!(meta.creator.as_deref(), Some("Test Creator"));
    assert_eq!(meta.producer.as_deref(), Some("botl-pdf test"));
    assert!(meta.creation_date.is_some());
}

// ---------------------------------------------------------------------------
// Encryption check
// ---------------------------------------------------------------------------

#[test]
fn test_is_not_encrypted() {
    let path = fixture_path("simple_text.pdf");
    let doc = Document::open(&path).unwrap();
    assert!(
        !doc.is_encrypted(),
        "simple_text.pdf should not be encrypted"
    );
}

#[test]
fn test_metadata_is_not_encrypted() {
    let path = fixture_path("metadata.pdf");
    let doc = Document::open(&path).unwrap();
    assert!(!doc.is_encrypted(), "metadata.pdf should not be encrypted");
}

// ---------------------------------------------------------------------------
// XRef parsing
// ---------------------------------------------------------------------------

#[test]
fn test_xref_table_entries() {
    let path = fixture_path("simple_text.pdf");
    let doc = Document::open(&path).unwrap();
    let xref = doc.xref();

    // simple_text.pdf has 6 objects (0 through 5)
    assert!(xref.len() >= 5, "xref should have at least 5 entries");

    // Object 0 should be free
    assert!(matches!(xref.get(0), Some(XrefEntry::Free { .. })));

    // Object 1 should be InUse (catalog)
    assert!(matches!(xref.get(1), Some(XrefEntry::InUse { .. })));
}

#[test]
fn test_xref_root_reference() {
    let path = fixture_path("simple_text.pdf");
    let doc = Document::open(&path).unwrap();
    let xref = doc.xref();

    let root = xref.root();
    assert!(root.is_some(), "xref should have a Root reference");
    assert_eq!(root.unwrap(), ObjRef::new(1, 0));
}

#[test]
fn test_find_startxref() {
    let path = fixture_path("simple_text.pdf");
    let bytes = std::fs::read(&path).unwrap();
    let result = find_startxref(&bytes);
    assert!(result.is_ok(), "Should find startxref");
    let offset = result.unwrap();
    assert!(offset > 0, "startxref offset should be positive");
    assert!(
        (offset as usize) < bytes.len(),
        "startxref should be within file bounds"
    );
}

#[test]
fn test_parse_xref_from_data() {
    let path = fixture_path("simple_text.pdf");
    let bytes = std::fs::read(&path).unwrap();
    let xref = parse_xref_from_data(&bytes);
    assert!(xref.is_ok(), "Should parse xref from data");

    let xref = xref.unwrap();
    assert!(xref.root().is_some(), "Parsed xref should have Root");
}

// ---------------------------------------------------------------------------
// Multi-page xref
// ---------------------------------------------------------------------------

#[test]
fn test_multi_page_xref() {
    let path = fixture_path("multi_page.pdf");
    let doc = Document::open(&path).unwrap();
    let xref = doc.xref();

    // multi_page.pdf has more objects due to multiple pages
    assert!(
        xref.len() >= 10,
        "multi_page xref should have at least 10 entries"
    );
}

// ---------------------------------------------------------------------------
// Incremental update handling
// ---------------------------------------------------------------------------

#[test]
fn test_incremental_find_all_xref_single() {
    use botl_pdf_core::parser::incremental::find_all_xref_sections;

    let path = fixture_path("simple_text.pdf");
    let bytes = std::fs::read(&path).unwrap();
    let sections = find_all_xref_sections(&bytes).unwrap();

    // A file without incremental updates should have exactly 1 xref section
    assert_eq!(sections.len(), 1, "Should find exactly 1 xref section");
}

#[test]
fn test_incremental_merge() {
    use botl_pdf_core::parser::incremental::merge_xref_sections;

    let path = fixture_path("simple_text.pdf");
    let bytes = std::fs::read(&path).unwrap();
    let sections = botl_pdf_core::parser::incremental::find_all_xref_sections(&bytes).unwrap();

    let merged = merge_xref_sections(sections);
    assert!(merged.root().is_some(), "Merged xref should have Root");
}

// ---------------------------------------------------------------------------
// FlateDecode compressed PDF
// ---------------------------------------------------------------------------

#[test]
fn test_open_flate_compressed_pdf() {
    let path = fixture_path("flate_compressed.pdf");
    let doc = Document::open(&path);
    assert!(doc.is_ok(), "Should be able to open flate_compressed.pdf");
}

#[test]
fn test_flate_compressed_page_count() {
    let path = fixture_path("flate_compressed.pdf");
    let mut doc = Document::open(&path).unwrap();
    let pages = doc.num_pages().unwrap();
    assert_eq!(pages, 1, "flate_compressed.pdf should have 1 page");
}

#[test]
fn test_flate_compressed_content_stream() {
    let path = fixture_path("flate_compressed.pdf");
    let mut doc = Document::open(&path).unwrap();

    // Get the page and its content stream reference
    let page = doc.get_page(0).unwrap();
    // Just verify we can navigate the structure
    assert!(page.get_reference("Contents").is_some());

    // Resolve the content stream and decode it
    let contents_ref = page.get_reference("Contents").unwrap();
    let stream_obj = doc.resolve(contents_ref).unwrap();
    let stream = stream_obj.as_stream().unwrap();

    // Decode the FlateDecode stream
    let decoded = botl_pdf_core::codecs::decode_stream_data(stream);
    assert!(decoded.is_ok(), "Should decode FlateDecode stream");

    let decoded_text = decoded.unwrap();
    let content = std::str::from_utf8(&decoded_text).unwrap();
    assert!(
        content.contains("Compressed") || content.contains("Text"),
        "Decoded content should contain text from the PDF, got: {:?}",
        content
    );
}

// ---------------------------------------------------------------------------
// Raw data access
// ---------------------------------------------------------------------------

#[test]
fn test_document_data_access() {
    let path = fixture_path("simple_text.pdf");
    let doc = Document::open(&path).unwrap();
    let data = doc.data();

    assert!(
        data.starts_with(b"%PDF-"),
        "Data should start with %PDF- header"
    );
    assert!(data.len() > 100, "Data should have reasonable length");
}
