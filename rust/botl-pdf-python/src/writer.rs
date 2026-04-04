use pyo3::exceptions::PyNotImplementedError;
use pyo3::prelude::*;

// ---------------------------------------------------------------------------
// PyWriter
// ---------------------------------------------------------------------------

/// PDF writer for creating and modifying PDF documents.
///
/// .. note:: This class is not yet implemented. All methods raise
///    ``NotImplementedError``.
#[pyclass]
pub struct PyWriter {
    _private: (),
}

#[pymethods]
impl PyWriter {
    #[new]
    fn new() -> PyResult<Self> {
        Err(PyNotImplementedError::new_err(
            "PyWriter is not yet implemented",
        ))
    }

    /// Add a blank page with the given dimensions.
    fn add_blank_page(&self, _width: f64, _height: f64) -> PyResult<()> {
        Err(PyNotImplementedError::new_err(
            "PyWriter.add_blank_page is not yet implemented",
        ))
    }

    /// Write a text string at the given position on the current page.
    fn write_text(&self, _x: f64, _y: f64, _text: &str) -> PyResult<()> {
        Err(PyNotImplementedError::new_err(
            "PyWriter.write_text is not yet implemented",
        ))
    }

    /// Save the PDF to a file at the given path.
    fn save(&self, _path: &str) -> PyResult<()> {
        Err(PyNotImplementedError::new_err(
            "PyWriter.save is not yet implemented",
        ))
    }

    /// Return the PDF content as bytes.
    fn to_bytes(&self) -> PyResult<()> {
        Err(PyNotImplementedError::new_err(
            "PyWriter.to_bytes is not yet implemented",
        ))
    }

    /// Number of pages in the writer.
    #[getter]
    fn num_pages(&self) -> PyResult<usize> {
        Err(PyNotImplementedError::new_err(
            "PyWriter.num_pages is not yet implemented",
        ))
    }

    fn __repr__(&self) -> &'static str {
        "Writer(not-implemented)"
    }
}
