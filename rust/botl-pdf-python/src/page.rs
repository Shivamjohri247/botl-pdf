use std::sync::Arc;

use pyo3::prelude::*;

use botl_pdf_core::layout::elements::{Char, GeomLine, GeomRect};
use botl_pdf_core::layout::strategy::{
    analyze_layout, blocks_to_layout_text, blocks_to_text, LayoutParams,
};
use botl_pdf_core::parser::document::Document;
use botl_pdf_core::parser::objects::PdfObject;
use botl_pdf_core::text::cmap::CMap;
use botl_pdf_core::text::fonts::{Font, FontCache};
use botl_pdf_core::text::operator::interpret_content_stream;

use crate::elements::{PyChar, PyGeomLine, PyGeomRect};
use crate::errors::IntoPyResult;

// ---------------------------------------------------------------------------
// PageData -- extracted content cache
// ---------------------------------------------------------------------------

/// Lazily-populated extracted page content.
#[derive(Clone, Default)]
struct PageData {
    chars: Vec<Char>,
    lines: Vec<GeomLine>,
    rects: Vec<GeomRect>,
    /// Page dimensions from the MediaBox.
    width: f64,
    #[allow(dead_code)]
    height: f64,
}

impl PageData {
    fn from_interpret_result(
        result: botl_pdf_core::text::operator::InterpretResult,
        width: f64,
        height: f64,
    ) -> Self {
        // Convert LineElement -> GeomLine, RectElement -> GeomRect
        let lines = result
            .lines
            .into_iter()
            .map(|le| GeomLine {
                x0: le.x0,
                y0: le.y0,
                x1: le.x1,
                y1: le.y1,
                line_width: le.line_width,
                color: Some(le.color),
            })
            .collect();

        let rects = result
            .rects
            .into_iter()
            .map(|re| GeomRect {
                bbox: re.bbox,
                line_width: re.line_width,
                stroke_color: re.stroke_color,
                fill_color: re.fill_color,
            })
            .collect();

        Self {
            chars: result.chars,
            lines,
            rects,
            width,
            height,
        }
    }
}

// ---------------------------------------------------------------------------
// PyPage
// ---------------------------------------------------------------------------

/// A single page from a PDF document.
///
/// Provides access to page dimensions, text extraction, and element access.
#[pyclass]
pub struct PyPage {
    /// The page index (0-based).
    page_number: usize,
    /// Page label string (e.g. "iii", "A-1").
    label: String,
    /// Rotation in degrees.
    rotation: i32,
    /// Width in points.
    width: f64,
    /// Height in points.
    height: f64,
    /// Cached extracted data.
    data: Arc<parking_lot::Mutex<Option<PageData>>>,
    /// Shared parsed document for content extraction.
    doc: Arc<parking_lot::Mutex<Document>>,
    /// Document-level font cache, keyed by font ObjRef.obj_num.
    font_cache: Arc<parking_lot::Mutex<hashbrown::HashMap<u32, Font>>>,
}

impl PyPage {
    pub fn new(
        page_number: usize,
        label: String,
        rotation: i32,
        width: f64,
        height: f64,
        doc: Arc<parking_lot::Mutex<Document>>,
        font_cache: Arc<parking_lot::Mutex<hashbrown::HashMap<u32, Font>>>,
    ) -> Self {
        Self {
            page_number,
            label,
            rotation,
            width,
            height,
            data: Arc::new(parking_lot::Mutex::new(None)),
            doc,
            font_cache,
        }
    }

    /// Ensure page data has been extracted; returns a cloned copy.
    fn ensure_extracted(&self) -> PyResult<PageData> {
        {
            let guard = self.data.lock();
            if guard.is_some() {
                return Ok(guard.clone().unwrap());
            }
        }

        let extracted = {
            let mut doc = self.doc.lock();
            let page_dict = doc.get_page(self.page_number).into_py()?;
            let (width, height) = extract_mediabox(&page_dict, self.width, self.height);
            let font_cache = build_font_cache(&page_dict, &mut doc, &self.font_cache)?;
            let content_data = get_content_stream(&page_dict, &mut doc)?;
            let result = interpret_content_stream(&content_data, &font_cache, height).into_py()?;
            PageData::from_interpret_result(result, width, height)
        };

        {
            let mut guard = self.data.lock();
            *guard = Some(extracted.clone());
        }

        Ok(extracted)
    }
}

// ---------------------------------------------------------------------------
// Free helper functions (not tied to &self, usable inside allow_threads)
// ---------------------------------------------------------------------------

fn extract_mediabox(
    page_dict: &botl_pdf_core::parser::objects::PdfDict,
    default_w: f64,
    default_h: f64,
) -> (f64, f64) {
    if let Some(arr) = page_dict.get_array("MediaBox") {
        if arr.len() >= 4 {
            let x0 = arr[0].as_real().unwrap_or(0.0);
            let y0 = arr[1].as_real().unwrap_or(0.0);
            let x1 = arr[2].as_real().unwrap_or(612.0);
            let y1 = arr[3].as_real().unwrap_or(792.0);
            return (x1 - x0, y1 - y0);
        }
    }
    (default_w, default_h)
}

fn build_font_cache(
    page_dict: &botl_pdf_core::parser::objects::PdfDict,
    doc: &mut Document,
    shared_font_cache: &Arc<parking_lot::Mutex<hashbrown::HashMap<u32, Font>>>,
) -> PyResult<FontCache> {
    let mut cache = FontCache::new();

    let resources = match page_dict.get_dict("Resources") {
        Some(r) => r,
        None => return Ok(cache),
    };

    let font_dict = match resources.get_dict("Font") {
        Some(f) => f,
        None => return Ok(cache),
    };

    for (key, value) in font_dict.iter() {
        let font_name_str = std::str::from_utf8(key).unwrap_or("?");

        // Check if this font is already in the shared document-level cache
        if let Some(font_ref) = value.as_reference() {
            let shared = shared_font_cache.lock();
            if let Some(cached_font) = shared.get(&font_ref.obj_num) {
                cache.insert(key, cached_font.clone());
                continue;
            }
        }

        // Obtain the top-level font dictionary
        let font_dict_obj: Option<Arc<PdfObject>> = if let Some(font_ref) = value.as_reference() {
            doc.resolve(font_ref).into_py().ok()
        } else {
            Some(Arc::new(value.clone()))
        };

        let Some(font_obj) = font_dict_obj else {
            continue;
        };
        let Some(fd) = font_obj.as_dict() else {
            continue;
        };
        let Ok(mut font) = Font::from_dict(font_name_str, fd) else {
            continue;
        };

        // -- Resolve the ToUnicode CMap stream, if present -----------------
        if let Some(tu_ref) = font.to_unicode_ref {
            if let Ok(tu_obj) = doc.resolve(tu_ref) {
                if let Some(stream) = tu_obj.as_stream() {
                    let decoded = crate::codecs_reexport::decode_stream(stream);
                    if let Ok(cmap_data) = decoded {
                        if let Ok(cmap) = CMap::parse(&cmap_data) {
                            font.set_cmap(cmap);
                        }
                    }
                }
            }
        }

        // -- For Type0 fonts, resolve the descendant CIDFont if needed ----
        if font.subtype == botl_pdf_core::text::fonts::FontSubtype::Type0 {
            // If encoding is still not set, try resolving DescendantFonts
            // by reference (inline dicts are handled by from_dict already).
            if font.encoding.is_none() {
                if let Some(desc_arr) = fd.get_array("DescendantFonts") {
                    if let Some(desc_first) = desc_arr.first() {
                        if let Some(desc_ref) = desc_first.as_reference() {
                            if let Ok(desc_obj) = doc.resolve(desc_ref) {
                                if let Some(desc_dict) = desc_obj.as_dict() {
                                    font.encoding = extract_encoding_from_descendant(desc_dict);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Store in shared document-level cache if this was a reference
        if let Some(font_ref) = value.as_reference() {
            shared_font_cache.lock().insert(font_ref.obj_num, font.clone());
        }

        cache.insert(key, font);
    }

    Ok(cache)
}

/// Extract encoding from a descendant CIDFont dictionary.
fn extract_encoding_from_descendant(
    desc_dict: &botl_pdf_core::parser::objects::PdfDict,
) -> Option<String> {
    match desc_dict.get_str("Encoding") {
        Some(botl_pdf_core::parser::objects::PdfObject::Name(n)) => {
            std::str::from_utf8(n).ok().map(String::from)
        }
        Some(botl_pdf_core::parser::objects::PdfObject::Dictionary(_enc_dict)) => {
            // Encoding dict inside descendant; treat as custom.
            // A common case is "Identity-H" as a name, but some PDFs use a dict.
            None
        }
        Some(botl_pdf_core::parser::objects::PdfObject::Reference(_r)) => {
            // Would need further resolution; skip for now.
            None
        }
        _ => None,
    }
}

fn get_content_stream(
    page_dict: &botl_pdf_core::parser::objects::PdfDict,
    doc: &mut Document,
) -> PyResult<Vec<u8>> {
    let contents_obj = page_dict.get_str("Contents");

    match contents_obj {
        Some(PdfObject::Reference(r)) => {
            let obj = doc.resolve(*r).into_py()?;
            if let Some(stream) = obj.as_stream() {
                Ok(crate::codecs_reexport::decode_stream(stream)?)
            } else {
                Ok(Vec::new())
            }
        }
        Some(PdfObject::Array(arr)) => {
            let mut combined = Vec::new();
            for item in arr {
                if let Some(r) = item.as_reference() {
                    let obj = doc.resolve(r).into_py()?;
                    if let Some(stream) = obj.as_stream() {
                        let decoded = crate::codecs_reexport::decode_stream(stream)?;
                        if !combined.is_empty() && !combined.ends_with(b"\n") {
                            combined.push(b'\n');
                        }
                        combined.extend_from_slice(&decoded);
                    }
                }
            }
            Ok(combined)
        }
        _ => Ok(Vec::new()),
    }
}

fn get_layout_params(
    layout: bool,
    layout_params: Option<&Bound<'_, PyAny>>,
) -> Option<LayoutParams> {
    if !layout && layout_params.is_none() {
        return None;
    }

    let mut params = LayoutParams::default();

    if let Some(lp) = layout_params {
        if let Ok(v) = lp.getattr("word_margin") {
            if let Ok(f) = v.extract::<f64>() {
                params.word_margin = f;
            }
        }
        if let Ok(v) = lp.getattr("line_margin") {
            if let Ok(f) = v.extract::<f64>() {
                params.line_margin = f;
            }
        }
        if let Ok(v) = lp.getattr("boxes_flow") {
            if let Ok(f) = v.extract::<f64>() {
                params.boxes_flow = f;
            }
        }
    }

    Some(params)
}

// ---------------------------------------------------------------------------
// PyLayoutParams helper for Python
// ---------------------------------------------------------------------------

/// Configuration parameters controlling layout analysis behaviour.
#[pyclass]
#[derive(Clone)]
pub struct PyLayoutParams {
    /// Maximum horizontal gap between chars in the same word,
    /// as a multiple of font size.  Default: 0.1
    #[pyo3(get, set)]
    pub word_margin: f64,
    /// Maximum vertical gap between lines in the same block,
    /// as a multiple of average line height.  Default: 0.5
    #[pyo3(get, set)]
    pub line_margin: f64,
    /// Controls reading-order strictness.
    /// 0.0 = strict horizontal, 1.0 = strict vertical.  Default: 0.5
    #[pyo3(get, set)]
    pub boxes_flow: f64,
}

#[pymethods]
impl PyLayoutParams {
    #[new]
    #[pyo3(signature = (word_margin=2.0, line_margin=0.5, boxes_flow=0.5))]
    fn new(word_margin: f64, line_margin: f64, boxes_flow: f64) -> Self {
        Self {
            word_margin,
            line_margin,
            boxes_flow,
        }
    }
}

// ---------------------------------------------------------------------------
// PyPage methods
// ---------------------------------------------------------------------------

#[pymethods]
impl PyPage {
    /// Page width in points.
    #[getter]
    fn width(&self) -> f64 {
        self.width
    }

    /// Page height in points.
    #[getter]
    fn height(&self) -> f64 {
        self.height
    }

    /// Rotation in degrees (0, 90, 180, 270).
    #[getter]
    fn rotation(&self) -> i32 {
        self.rotation
    }

    /// Zero-based page index.
    #[getter]
    fn page_number(&self) -> usize {
        self.page_number
    }

    /// Page label string (e.g. ``"iii"``, ``"A-1"``).
    #[getter]
    fn label(&self) -> &str {
        &self.label
    }

    /// Extract text from this page.
    ///
    /// :param layout: If True, preserve spatial layout using spaces/newlines.
    /// :param layout_params: Optional :class:`LayoutParams` to control layout analysis.
    /// :returns: Extracted text as a string.
    #[pyo3(signature = (layout=false, layout_params=None))]
    fn extract_text(
        &self,
        py: Python<'_>,
        layout: bool,
        layout_params: Option<Bound<'_, PyAny>>,
    ) -> PyResult<String> {
        let params = get_layout_params(layout, layout_params.as_ref());

        // Clone handle for the closure; the actual heavy work happens without GIL.
        let data_handle = self.data.clone();
        let doc_handle = self.doc.clone();
        let font_cache_handle = self.font_cache.clone();
        let page_index = self.page_number;
        let default_w = self.width;
        let default_h = self.height;

        let text = py.allow_threads(|| -> PyResult<String> {
            let extracted = {
                let guard = data_handle.lock();
                if let Some(ref data) = *guard {
                    data.clone()
                } else {
                    drop(guard);
                    let mut doc = doc_handle.lock();
                    let page_dict = doc.get_page(page_index).into_py()?;
                    let (width, height) = extract_mediabox(&page_dict, default_w, default_h);
                    let font_cache = build_font_cache(&page_dict, &mut doc, &font_cache_handle)?;
                    let content_data = get_content_stream(&page_dict, &mut doc)?;
                    let result =
                        interpret_content_stream(&content_data, &font_cache, height).into_py()?;
                    let data = PageData::from_interpret_result(result, width, height);

                    {
                        let mut guard = data_handle.lock();
                        *guard = Some(data.clone());
                    }

                    data
                }
            };

            // Generate text (uses layout analysis to now taking ownership of chars)
            if let Some(lp) = params {
                let blocks = analyze_layout(extracted.chars, &lp);
                if layout {
                    Ok(blocks_to_layout_text(&blocks, extracted.width))
                } else {
                    Ok(blocks_to_text(&blocks))
                }
            } else {
                // Default mode: use layout analysis with default params for proper spacing
                let lp = LayoutParams::default();
                let blocks = analyze_layout(extracted.chars, &lp);
                Ok(blocks_to_text(&blocks))
            }
        })?;

        Ok(text)
    }

    /// All characters extracted from the page.
    #[getter]
    fn chars(&self) -> PyResult<Vec<PyChar>> {
        let data = self.ensure_extracted()?;
        Ok(data.chars.iter().map(|c| PyChar::new(c.clone())).collect())
    }

    /// Geometric lines on the page.
    #[getter]
    fn lines(&self) -> PyResult<Vec<PyGeomLine>> {
        let data = self.ensure_extracted()?;
        Ok(data
            .lines
            .iter()
            .map(|l| PyGeomLine::new(l.clone()))
            .collect())
    }

    /// Geometric rectangles on the page.
    #[getter]
    fn rects(&self) -> PyResult<Vec<PyGeomRect>> {
        let data = self.ensure_extracted()?;
        Ok(data
            .rects
            .iter()
            .map(|r| PyGeomRect::new(r.clone()))
            .collect())
    }

    fn __repr__(&self) -> String {
        format!(
            "Page({}, width={:.1}, height={:.1})",
            self.page_number, self.width, self.height
        )
    }
}
