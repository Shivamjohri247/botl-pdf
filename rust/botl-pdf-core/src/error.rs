/// Core error type for all botl-pdf operations.
#[derive(Debug, thiserror::Error)]
pub enum BotlError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("PDF is encrypted. Provide password via botl_pdf.open(path, password=...)")]
    PasswordRequired,

    #[error("Page {page} out of range (document has {total} pages)")]
    PageOutOfRange {
        page: usize,
        total: usize,
    },

    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    #[error("Codec error: {0}")]
    CodecError(String),

    #[error("Font error: {0}")]
    FontError(String),

    #[error("CMap error: {0}")]
    CMapError(String),

    #[error("Layout error: {0}")]
    LayoutError(String),

    #[error("Invalid object reference: {0} {1} R")]
    InvalidReference(u32, u16),

    #[error("Cross-reference error: {0}")]
    XrefError(String),

    #[error("Truncated file")]
    TruncatedFile,
}

pub type Result<T> = std::result::Result<T, BotlError>;
