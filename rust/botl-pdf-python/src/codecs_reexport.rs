use crate::errors::IntoPyResult;
use botl_pdf_core::parser::objects::PdfStream;

/// Decode a PDF stream using the filters specified in its dictionary.
///
/// This is a thin wrapper around `botl_pdf_core::codecs::decode_stream_data`
/// for use within the Python binding layer.
pub fn decode_stream(stream: &PdfStream) -> Result<Vec<u8>, pyo3::PyErr> {
    botl_pdf_core::codecs::decode_stream_data(stream).into_py()
}
