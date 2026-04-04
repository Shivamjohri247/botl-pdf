use botl_pdf_core::error::BotlError;
use pyo3::exceptions::{
    PyFileNotFoundError, PyRuntimeError, PyValueError,
};
use pyo3::prelude::*;

/// Convert a BotlError into the most appropriate Python exception.
pub fn to_pyerr(err: BotlError) -> PyErr {
    match &err {
        BotlError::IoError(io_err) => match io_err.kind() {
            std::io::ErrorKind::NotFound => {
                PyFileNotFoundError::new_err(err.to_string())
            }
            std::io::ErrorKind::PermissionDenied => {
                PyFileNotFoundError::new_err(err.to_string())
            }
            _ => PyRuntimeError::new_err(err.to_string()),
        },

        BotlError::ParseError(_) => PyValueError::new_err(err.to_string()),

        BotlError::EncryptionError(_) => PyRuntimeError::new_err(err.to_string()),

        BotlError::PasswordRequired => PyRuntimeError::new_err(err.to_string()),

        BotlError::PageOutOfRange { .. } => PyValueError::new_err(err.to_string()),

        BotlError::UnsupportedFeature(_) => PyRuntimeError::new_err(err.to_string()),

        BotlError::CodecError(_) => PyRuntimeError::new_err(err.to_string()),

        BotlError::FontError(_) => PyRuntimeError::new_err(err.to_string()),

        BotlError::CMapError(_) => PyRuntimeError::new_err(err.to_string()),

        BotlError::LayoutError(_) => PyRuntimeError::new_err(err.to_string()),

        BotlError::InvalidReference(_, _) => PyValueError::new_err(err.to_string()),

        BotlError::XrefError(_) => PyValueError::new_err(err.to_string()),

        BotlError::TruncatedFile => PyValueError::new_err(err.to_string()),
    }
}

/// A trait to ergonomically convert `Result<T, BotlError>` into `PyResult<T>`.
pub trait IntoPyResult<T> {
    fn into_py(self) -> PyResult<T>;
}

impl<T> IntoPyResult<T> for Result<T, BotlError> {
    fn into_py(self) -> PyResult<T> {
        self.map_err(to_pyerr)
    }
}
