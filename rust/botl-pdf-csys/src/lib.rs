//! Safe wrappers over codec FFI bindings.
//!
//! Provides safe Rust interfaces for PDF stream decompression codecs:
//! - JPEG (DCTDecode) via jpeg-decoder
//! - JPEG2000 (JPXDecode) via OpenJPEG (requires `openjpeg` feature)

pub mod image;
pub mod jpeg;
pub mod jpx;

pub use image::DecodedImage;
