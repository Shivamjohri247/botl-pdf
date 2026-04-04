use pyo3::prelude::*;

use botl_pdf_core::geometry::BBox;
use botl_pdf_core::layout::elements::{Char, GeomLine, GeomRect, TextBlock, TextLine, Word};

// ---------------------------------------------------------------------------
// PyBBox
// ---------------------------------------------------------------------------

/// An axis-aligned bounding box with coordinates ``(x0, y0, x1, y1)``.
///
/// The coordinate system uses top-left origin (matching the library's
/// default y-flip of the PDF coordinate space).
#[pyclass(frozen)]
#[derive(Clone)]
pub struct PyBBox {
    inner: BBox,
}

impl PyBBox {
    pub fn new(inner: BBox) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyBBox {
    /// Left edge x-coordinate.
    #[getter]
    fn x0(&self) -> f64 {
        self.inner.x0
    }

    /// Top edge y-coordinate.
    #[getter]
    fn y0(&self) -> f64 {
        self.inner.y0
    }

    /// Right edge x-coordinate.
    #[getter]
    fn x1(&self) -> f64 {
        self.inner.x1
    }

    /// Bottom edge y-coordinate.
    #[getter]
    fn y1(&self) -> f64 {
        self.inner.y1
    }

    /// Width of the bounding box (``x1 - x0``).
    #[getter]
    fn width(&self) -> f64 {
        self.inner.width()
    }

    /// Height of the bounding box (``y1 - y0``).
    #[getter]
    fn height(&self) -> f64 {
        self.inner.height()
    }

    /// Return the ``(x, y)`` center point of the bounding box.
    fn center(&self) -> (f64, f64) {
        self.inner.center()
    }

    /// Area of the bounding box.
    fn area(&self) -> f64 {
        self.inner.area()
    }

    fn __repr__(&self) -> String {
        format!(
            "BBox({:.1}, {:.1}, {:.1}, {:.1})",
            self.inner.x0, self.inner.y0, self.inner.x1, self.inner.y1
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
}

// ---------------------------------------------------------------------------
// PyChar
// ---------------------------------------------------------------------------

/// A single character extracted from a PDF page, with its position and style.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct PyChar {
    inner: Char,
}

impl PyChar {
    pub fn new(inner: Char) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyChar {
    /// The Unicode text of this character.
    #[getter]
    fn text(&self) -> &str {
        &self.inner.text
    }

    /// Bounding box of this character.
    #[getter]
    fn bbox(&self) -> PyBBox {
        PyBBox::new(self.inner.bbox)
    }

    /// Font resource name (e.g. ``"F1"``).
    #[getter]
    fn font_name(&self) -> &str {
        &self.inner.font_name
    }

    /// Font size in points.
    #[getter]
    fn font_size(&self) -> f64 {
        self.inner.font_size
    }

    /// Whether the character is bold.
    #[getter]
    fn bold(&self) -> bool {
        self.inner.bold
    }

    /// Whether the character is italic.
    #[getter]
    fn italic(&self) -> bool {
        self.inner.italic
    }

    /// Fill colour as ``(R, G, B)`` normalised to 0.0-1.0, or ``None``.
    #[getter]
    fn color(&self) -> Option<(f64, f64, f64)> {
        self.inner.color
    }

    /// Stroke colour as ``(R, G, B)`` normalised to 0.0-1.0, or ``None``.
    #[getter]
    fn stroking_color(&self) -> Option<(f64, f64, f64)> {
        self.inner.stroking_color
    }

    /// Text run identifier: characters from the same Tj/TJ call share a run_id.
    #[getter]
    fn run_id(&self) -> u32 {
        self.inner.run_id
    }

    /// Rotation in degrees.
    #[getter]
    fn rotation(&self) -> f64 {
        self.inner.rotation
    }

    fn __repr__(&self) -> String {
        format!("Char({:?})", self.inner.text)
    }
}

// ---------------------------------------------------------------------------
// PyWord
// ---------------------------------------------------------------------------

/// A group of characters that form a single word.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct PyWord {
    inner: Word,
}

impl PyWord {
    pub fn new(inner: Word) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyWord {
    /// The text content of this word.
    #[getter]
    fn text(&self) -> &str {
        &self.inner.text
    }

    /// Bounding box enclosing all characters in this word.
    #[getter]
    fn bbox(&self) -> PyBBox {
        PyBBox::new(self.inner.bbox)
    }

    /// The characters that make up this word.
    #[getter]
    fn chars(&self) -> Vec<PyChar> {
        self.inner
            .chars
            .iter()
            .map(|c| PyChar::new(c.clone()))
            .collect()
    }

    /// Text direction (``"ltr"``, ``"rtl"``, ``"ttb"``, ``"btt"``).
    #[getter]
    fn direction(&self) -> &str {
        &self.inner.direction
    }

    /// Dominant font name.
    #[getter]
    fn font_name(&self) -> &str {
        &self.inner.font_name
    }

    /// Dominant font size in points.
    #[getter]
    fn font_size(&self) -> f64 {
        self.inner.font_size
    }

    /// Width of the word bounding box.
    #[getter]
    fn width(&self) -> f64 {
        self.inner.width()
    }

    /// Height of the word bounding box.
    #[getter]
    fn height(&self) -> f64 {
        self.inner.height()
    }

    fn __repr__(&self) -> String {
        format!("Word({:?})", self.inner.text)
    }
}

// ---------------------------------------------------------------------------
// PyTextLine
// ---------------------------------------------------------------------------

/// A line of text: words arranged horizontally.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct PyTextLine {
    inner: TextLine,
}

impl PyTextLine {
    pub fn new(inner: TextLine) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyTextLine {
    /// All words in this line, in reading order.
    #[getter]
    fn words(&self) -> Vec<PyWord> {
        self.inner
            .words
            .iter()
            .map(|w| PyWord::new(w.clone()))
            .collect()
    }

    /// Bounding box enclosing all words.
    #[getter]
    fn bbox(&self) -> PyBBox {
        PyBBox::new(self.inner.bbox)
    }

    /// The combined text of all words.
    #[getter]
    fn text(&self) -> &str {
        &self.inner.text
    }

    fn __repr__(&self) -> String {
        format!("TextLine({:?})", self.inner.text)
    }
}

// ---------------------------------------------------------------------------
// PyTextBlock
// ---------------------------------------------------------------------------

/// A paragraph-like grouping of text lines.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct PyTextBlock {
    inner: TextBlock,
}

impl PyTextBlock {
    pub fn new(inner: TextBlock) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyTextBlock {
    /// All lines in this block, in reading order.
    #[getter]
    fn lines(&self) -> Vec<PyTextLine> {
        self.inner
            .lines
            .iter()
            .map(|l| PyTextLine::new(l.clone()))
            .collect()
    }

    /// Bounding box enclosing all lines.
    #[getter]
    fn bbox(&self) -> PyBBox {
        PyBBox::new(self.inner.bbox)
    }

    /// The combined text of all lines (separated by newlines).
    #[getter]
    fn text(&self) -> &str {
        &self.inner.text
    }

    fn __repr__(&self) -> String {
        format!("TextBlock({:?})", self.inner.text)
    }
}

// ---------------------------------------------------------------------------
// PyTOCEntry
// ---------------------------------------------------------------------------

/// An entry in the document's Table of Contents (outline / bookmarks).
#[pyclass]
#[derive(Clone)]
pub struct PyTOCEntry {
    inner: TOCEntryData,
}

/// Internal data for a TOC entry.
#[derive(Clone)]
struct TOCEntryData {
    title: String,
    level: u32,
    page_number: Option<usize>,
    dest: Option<String>,
}

impl PyTOCEntry {
    pub fn new(title: String, level: u32, page_number: Option<usize>) -> Self {
        Self {
            inner: TOCEntryData {
                title,
                level,
                page_number,
                dest: None,
            },
        }
    }

    pub fn with_dest(mut self, dest: String) -> Self {
        self.inner.dest = Some(dest);
        self
    }
}

#[pymethods]
impl PyTOCEntry {
    /// Title of the outline entry.
    #[getter]
    fn title(&self) -> &str {
        &self.inner.title
    }

    /// Nesting level (0 = top-level).
    #[getter]
    fn level(&self) -> u32 {
        self.inner.level
    }

    /// Destination page number (0-indexed), if resolvable.
    #[getter]
    fn page_number(&self) -> Option<usize> {
        self.inner.page_number
    }

    /// Raw destination string, if available.
    #[getter]
    fn dest(&self) -> Option<&str> {
        self.inner.dest.as_deref()
    }

    fn __repr__(&self) -> String {
        format!(
            "TOCEntry(level={}, title={:?})",
            self.inner.level, self.inner.title
        )
    }
}

// ---------------------------------------------------------------------------
// PyGeomLine
// ---------------------------------------------------------------------------

/// A geometric line on the page (not a text line).
#[pyclass(frozen)]
#[derive(Clone)]
pub struct PyGeomLine {
    inner: GeomLine,
}

impl PyGeomLine {
    pub fn new(inner: GeomLine) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyGeomLine {
    /// Start x-coordinate.
    #[getter]
    fn x0(&self) -> f64 {
        self.inner.x0
    }

    /// Start y-coordinate.
    #[getter]
    fn y0(&self) -> f64 {
        self.inner.y0
    }

    /// End x-coordinate.
    #[getter]
    fn x1(&self) -> f64 {
        self.inner.x1
    }

    /// End y-coordinate.
    #[getter]
    fn y1(&self) -> f64 {
        self.inner.y1
    }

    /// Line width.
    #[getter]
    fn line_width(&self) -> f64 {
        self.inner.line_width
    }

    /// Stroke colour as ``(R, G, B)``, or ``None``.
    #[getter]
    fn color(&self) -> Option<(f64, f64, f64)> {
        self.inner.color
    }

    fn __repr__(&self) -> String {
        format!(
            "GeomLine({:.1},{:.1} -> {:.1},{:.1})",
            self.inner.x0, self.inner.y0, self.inner.x1, self.inner.y1
        )
    }
}

// ---------------------------------------------------------------------------
// PyGeomRect
// ---------------------------------------------------------------------------

/// A geometric rectangle on the page.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct PyGeomRect {
    inner: GeomRect,
}

impl PyGeomRect {
    pub fn new(inner: GeomRect) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyGeomRect {
    /// Bounding box of the rectangle.
    #[getter]
    fn bbox(&self) -> PyBBox {
        PyBBox::new(self.inner.bbox)
    }

    /// Line width.
    #[getter]
    fn line_width(&self) -> f64 {
        self.inner.line_width
    }

    /// Stroke colour as ``(R, G, B)``, or ``None``.
    #[getter]
    fn stroke_color(&self) -> Option<(f64, f64, f64)> {
        self.inner.stroke_color
    }

    /// Fill colour as ``(R, G, B)``, or ``None``.
    #[getter]
    fn fill_color(&self) -> Option<(f64, f64, f64)> {
        self.inner.fill_color
    }

    fn __repr__(&self) -> String {
        format!("GeomRect({})", self.inner.bbox)
    }
}
