//! Decoded image representation.

/// A decoded image from a PDF stream codec.
#[derive(Debug)]
pub struct DecodedImage {
    /// Pixel data in row-major order (RGB or grayscale).
    pub data: Vec<u8>,
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Number of color components (1=grayscale, 3=RGB, 4=CMYK).
    pub components: u8,
}

impl DecodedImage {
    /// Create a new decoded image.
    pub fn new(data: Vec<u8>, width: u32, height: u32, components: u8) -> Self {
        Self {
            data,
            width,
            height,
            components,
        }
    }

    /// Returns the expected data length based on dimensions and components.
    pub fn expected_len(&self) -> usize {
        self.width as usize * self.height as usize * self.components as usize
    }
}
