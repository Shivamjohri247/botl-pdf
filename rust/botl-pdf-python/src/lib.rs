use pyo3::prelude::*;

mod codecs_reexport;
mod document;
mod elements;
mod errors;
mod page;
mod writer;

use document::{PyDocument, open};
use elements::{
    PyBBox, PyChar, PyGeomLine, PyGeomRect, PyTextBlock, PyTextLine, PyTOCEntry, PyWord,
};
use page::{PyPage, PyLayoutParams};
use writer::PyWriter;

/// botl-pdf internal native module.
///
/// This module provides low-level bindings to the botl-pdf Rust core.
/// End-users should use the ``botl_pdf`` Python package instead.
#[pymodule]
#[pyo3(name = "_core")]
fn botl_pdf_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Functions
    m.add_function(wrap_pyfunction!(open, m)?)?;

    // Classes
    m.add_class::<PyDocument>()?;
    m.add_class::<PyPage>()?;
    m.add_class::<PyLayoutParams>()?;
    m.add_class::<PyBBox>()?;
    m.add_class::<PyChar>()?;
    m.add_class::<PyWord>()?;
    m.add_class::<PyTextLine>()?;
    m.add_class::<PyTextBlock>()?;
    m.add_class::<PyTOCEntry>()?;
    m.add_class::<PyGeomLine>()?;
    m.add_class::<PyGeomRect>()?;
    m.add_class::<PyWriter>()?;

    Ok(())
}
