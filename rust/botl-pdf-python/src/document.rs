use std::sync::Arc;

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use botl_pdf_core::parser::document::{Document, DocumentMetadata};

use crate::elements::PyTOCEntry;
use crate::errors::IntoPyResult;
use crate::page::PyPage;

// ---------------------------------------------------------------------------
// PyDocument
// ---------------------------------------------------------------------------

/// A parsed PDF document.
///
/// Created by :func:`botl_pdf.open`.
///
/// >>> import botl_pdf
/// >>> doc = botl_pdf.open("report.pdf")
/// >>> print(doc.metadata)
/// >>> for page in doc:
/// ...     print(page.extract_text())
#[pyclass]
pub struct PyDocument {
    doc: Arc<parking_lot::Mutex<Document>>,
    /// Cache of page dimensions and metadata to avoid re-parsing.
    /// Stored as (width, height, rotation, label) tuples.
    page_info_cache: Vec<(f64, f64, i32, String)>,
}

impl PyDocument {
    pub fn new(doc: Document) -> Self {
        Self {
            doc: Arc::new(parking_lot::Mutex::new(doc)),
            page_info_cache: Vec::new(),
        }
    }

    /// Ensure the page_info_cache is populated.
    fn ensure_page_cache(&mut self) -> PyResult<()> {
        if !self.page_info_cache.is_empty() {
            return Ok(());
        }

        let mut doc = self.doc.lock();
        let num_pages = doc.num_pages().into_py()?;
        let mut cache = Vec::with_capacity(num_pages);

        for i in 0..num_pages {
            let page_dict = doc.get_page(i).into_py()?;

            // MediaBox
            let (width, height) = Self::extract_page_dimensions(&page_dict);

            // Rotate
            let rotation = page_dict
                .get_integer("Rotate")
                .unwrap_or(0) as i32;

            // Label (default to 1-based index)
            let label = format!("{}", i + 1);

            cache.push((width, height, rotation, label));
        }

        self.page_info_cache = cache;
        Ok(())
    }

    fn extract_page_dimensions(
        page_dict: &botl_pdf_core::parser::objects::PdfDict,
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
        // Default US Letter
        (612.0, 792.0)
    }
}

#[pymethods]
impl PyDocument {
    /// Document metadata as a dictionary.
    ///
    /// Keys: ``title``, ``author``, ``subject``, ``keywords``,
    /// ``creator``, ``producer``, ``creation_date``, ``mod_date``,
    /// ``page_count``, ``version``.
    #[getter]
    fn metadata(&mut self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let meta: DocumentMetadata = self.doc.lock().metadata().into_py()?;

        let dict = PyDict::new(py);

        dict.set_item("title", meta.title.as_deref().unwrap_or(""))?;
        dict.set_item("author", meta.author.as_deref().unwrap_or(""))?;
        dict.set_item("subject", meta.subject.as_deref().unwrap_or(""))?;
        dict.set_item("keywords", meta.keywords.as_deref().unwrap_or(""))?;
        dict.set_item("creator", meta.creator.as_deref().unwrap_or(""))?;
        dict.set_item("producer", meta.producer.as_deref().unwrap_or(""))?;
        dict.set_item("creation_date", meta.creation_date.as_deref().unwrap_or(""))?;
        dict.set_item("mod_date", meta.mod_date.as_deref().unwrap_or(""))?;
        dict.set_item("page_count", meta.page_count)?;
        dict.set_item("version", meta.version.as_deref().unwrap_or(""))?;

        Ok(dict.into())
    }

    /// Number of pages in the document.
    #[getter]
    fn num_pages(&mut self) -> PyResult<usize> {
        Ok(self.doc.lock().num_pages().into_py()?)
    }

    /// Whether the document is encrypted.
    #[getter]
    fn is_encrypted(&self) -> bool {
        self.doc.lock().is_encrypted()
    }

    /// Table of Contents (outline / bookmarks).
    ///
    /// Returns a list of :class:`TOCEntry` objects.
    #[getter]
    fn toc(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let entries = self.extract_toc()?;
        let list = PyList::new(py, entries)?;
        Ok(list.into())
    }

    /// Get a page by index (0-based).
    fn get_page(&mut self, py: Python<'_>, index: usize) -> PyResult<Py<PyPage>> {
        self.ensure_page_cache()?;

        if index >= self.page_info_cache.len() {
            return Err(pyo3::exceptions::PyIndexError::new_err(format!(
                "Page index {} out of range (document has {} pages)",
                index,
                self.page_info_cache.len()
            )));
        }

        let (width, height, rotation, label) = self.page_info_cache[index].clone();

        let page = PyPage::new(index, label, rotation, width, height, self.doc.clone());
        Py::new(py, page)
    }

    /// Number of pages (alias for ``len(doc)``).
    fn __len__(&mut self) -> PyResult<usize> {
        self.num_pages()
    }

    /// Subscript access: ``doc[0]`` returns the first page.
    fn __getitem__(&mut self, py: Python<'_>, index: isize) -> PyResult<Py<PyPage>> {
        let num = self.num_pages()? as isize;
        let actual = if index < 0 { num + index } else { index };
        if actual < 0 || actual >= num {
            return Err(pyo3::exceptions::PyIndexError::new_err(format!(
                "Page index {} out of range",
                index
            )));
        }
        self.get_page(py, actual as usize)
    }

    fn __repr__(&self) -> PyResult<String> {
        let n = self.doc.lock().num_pages().into_py()?;
        Ok(format!("<Document pages={}>", n))
    }
}

// ---------------------------------------------------------------------------
// TOC extraction (outline / bookmarks)
// ---------------------------------------------------------------------------

impl PyDocument {
    fn extract_toc(&self) -> PyResult<Vec<PyTOCEntry>> {
        let mut doc = self.doc.lock();
        let catalog = doc.catalog().into_py()?;

        // Get the Outlines entry from the catalog
        let outlines_ref = match catalog.get_reference("Outlines") {
            Some(r) => r,
            None => return Ok(Vec::new()),
        };

        let outlines_obj = doc.resolve(outlines_ref).into_py()?;
        let outlines_dict = outlines_obj
            .as_dict()
            .ok_or_else(|| {
                crate::errors::to_pyerr(botl_pdf_core::error::BotlError::ParseError(
                    "Outlines is not a dict".into(),
                ))
            })?;

        // Get the First entry (first top-level outline item)
        let first_ref = match outlines_dict.get_reference("First") {
            Some(r) => r,
            None => return Ok(Vec::new()),
        };

        let mut entries = Vec::new();
        drop(doc); // release lock before recursive walk_outline which takes its own
        Self::walk_outline(&self.doc, first_ref, 0, &mut entries)?;

        Ok(entries)
    }

    fn walk_outline(
        doc: &parking_lot::Mutex<Document>,
        item_ref: botl_pdf_core::parser::objects::ObjRef,
        level: u32,
        entries: &mut Vec<PyTOCEntry>,
    ) -> PyResult<()> {
        let mut doc_guard = doc.lock();
        let item_obj = doc_guard.resolve(item_ref).into_py()?;
        let item_dict = item_obj
            .as_dict()
            .ok_or_else(|| {
                crate::errors::to_pyerr(botl_pdf_core::error::BotlError::ParseError(
                    "Outline item is not a dict".into(),
                ))
            })?;

        // Title
        let title = item_dict
            .get_string("Title")
            .unwrap_or("")
            .to_string();

        // Try to resolve destination to a page number
        let page_number = Self::resolve_outline_dest(&mut doc_guard, &item_dict);

        entries.push(PyTOCEntry::new(title, level, page_number));

        // Collect child/sibling refs before releasing the lock
        let child_ref = item_dict.get_reference("First");
        let next_ref = item_dict.get_reference("Next");
        drop(doc_guard);

        // Walk children (First)
        if let Some(child_ref) = child_ref {
            Self::walk_outline(doc, child_ref, level + 1, entries)?;
        }

        // Walk siblings (Next)
        if let Some(next_ref) = next_ref {
            Self::walk_outline(doc, next_ref, level, entries)?;
        }

        Ok(())
    }

    /// Resolve an outline item's destination to a 0-based page index.
    ///
    /// PDF outline items can specify destinations in two ways:
    /// 1. Via `/Dest` -- directly points to a destination array or name
    /// 2. Via `/A/D` -- an action dictionary with a `/D` destination entry
    ///
    /// A destination array has the form `[page_ref /XYZ left top zoom]`
    /// where `page_ref` is an indirect reference to a page dictionary.
    fn resolve_outline_dest(
        doc: &mut Document,
        item_dict: &botl_pdf_core::parser::objects::PdfDict,
    ) -> Option<usize> {
        // Try /Dest first
        if let Some(dest) = item_dict.get_str("Dest") {
            if let Some(page_idx) = doc.resolve_destination_page(dest) {
                return Some(page_idx);
            }
        }

        // Try /A (action) with /D (destination)
        if let Some(action_dict) = item_dict.get_dict("A") {
            if let Some(dest) = action_dict.get_str("D") {
                if let Some(page_idx) = doc.resolve_destination_page(dest) {
                    return Some(page_idx);
                }
            }
        }

        None
    }
}

// ---------------------------------------------------------------------------
// Python module-level open function
// ---------------------------------------------------------------------------

/// Open a PDF document.
///
/// :param path_or_bytes: A file path (string/PathLike) or raw PDF bytes.
/// :param password: Optional password for encrypted documents.
/// :param lazy: If True (default), defer content stream decoding until needed.
/// :returns: A :class:`Document` instance.
#[pyfunction]
#[pyo3(signature = (path_or_bytes, password=None, lazy=true))]
pub fn open(
    py: Python<'_>,
    path_or_bytes: &Bound<'_, PyAny>,
    password: Option<&str>,
    lazy: bool,
) -> PyResult<Py<PyDocument>> {
    // Password support is not yet implemented in the core, but we accept
    // the parameter for API compatibility.
    let _ = password;

    let doc = if path_or_bytes.is_instance_of::<pyo3::types::PyBytes>() {
        let bytes: &[u8] = path_or_bytes.downcast::<pyo3::types::PyBytes>()?.as_bytes();
        let data = bytes.to_vec();
        py.allow_threads(|| Document::from_bytes(data).into_py())?
    } else {
        // Try as a string path
        let path_str: String = path_or_bytes.extract()?;
        let path = std::path::Path::new(&path_str);
        py.allow_threads(|| Document::open(path).into_py())?
    };

    let _ = lazy; // lazy is the default behaviour since we cache pages on demand

    let pydoc = PyDocument::new(doc);
    Py::new(py, pydoc)
}
