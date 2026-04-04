use crate::geometry::bbox::Point;

/// 2D affine transformation matrix in PDF format: [a, b, c, d, e, f]
///
/// Represents the transformation:
/// ```text
/// x' = a*x + c*y + e
/// y' = b*x + d*y + f
/// ```
///
/// Stored as `[a, b, c, d, e, f]` matching PDF's `Tm` and `cm` operator format.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
}

impl Matrix {
    pub const IDENTITY: Matrix = Matrix {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };

    pub fn new(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> Self {
        Self { a, b, c, d, e, f }
    }

    /// Create from a slice of 6 elements.
    pub fn from_slice(vals: &[f64]) -> Option<Self> {
        if vals.len() != 6 {
            return None;
        }
        Some(Self::new(
            vals[0], vals[1], vals[2], vals[3], vals[4], vals[5],
        ))
    }

    /// Matrix multiplication: self * other.
    pub fn multiply(&self, other: &Matrix) -> Matrix {
        Matrix {
            a: self.a * other.a + self.c * other.b,
            b: self.b * other.a + self.d * other.b,
            c: self.a * other.c + self.c * other.d,
            d: self.b * other.c + self.d * other.d,
            e: self.a * other.e + self.c * other.f + self.e,
            f: self.b * other.e + self.d * other.f + self.f,
        }
    }

    /// Transform a point through this matrix.
    pub fn transform_point(&self, p: &Point) -> Point {
        Point {
            x: self.a * p.x + self.c * p.y + self.e,
            y: self.b * p.x + self.d * p.y + self.f,
        }
    }

    /// Transform a distance vector (no translation).
    pub fn transform_vector(&self, dx: f64, dy: f64) -> (f64, f64) {
        (self.a * dx + self.c * dy, self.b * dx + self.d * dy)
    }

    /// Translation matrix.
    pub fn translate(tx: f64, ty: f64) -> Self {
        Matrix {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: tx,
            f: ty,
        }
    }

    /// Scale matrix.
    pub fn scale(sx: f64, sy: f64) -> Self {
        Matrix {
            a: sx,
            b: 0.0,
            c: 0.0,
            d: sy,
            e: 0.0,
            f: 0.0,
        }
    }

    /// Rotation matrix (radians).
    pub fn rotate(angle: f64) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Matrix {
            a: cos,
            b: sin,
            c: -sin,
            d: cos,
            e: 0.0,
            f: 0.0,
        }
    }

    /// Extract the scaling factor (determinant of the 2x2 sub-matrix).
    pub fn scaling_factor(&self) -> f64 {
        (self.a * self.d - self.b * self.c).abs().sqrt()
    }

    /// Extract the font size from a text matrix + font size combo.
    /// The effective font size is the magnitude of the vertical vector.
    pub fn effective_font_size(&self, font_size: f64) -> f64 {
        let (_, dy) = self.transform_vector(0.0, font_size);
        dy.abs()
    }
}

impl std::ops::Mul for Matrix {
    type Output = Matrix;

    fn mul(self, other: Matrix) -> Matrix {
        self.multiply(&other)
    }
}
