use std::fmt;

/// Axis-aligned bounding box with top-left origin (matching PDF spec's coordinate
/// system after our default y-flip).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BBox {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

impl BBox {
    #[inline]
    pub fn new(x0: f64, y0: f64, x1: f64, y1: f64) -> Self {
        debug_assert!(x0 <= x1, "x0 must be <= x1");
        debug_assert!(y0 <= y1, "y0 must be <= y1");
        Self { x0, y0, x1, y1 }
    }

    #[inline]
    pub fn width(&self) -> f64 {
        self.x1 - self.x0
    }

    #[inline]
    pub fn height(&self) -> f64 {
        self.y1 - self.y0
    }

    #[inline]
    pub fn center(&self) -> (f64, f64) {
        ((self.x0 + self.x1) / 2.0, (self.y0 + self.y1) / 2.0)
    }

    #[inline]
    pub fn area(&self) -> f64 {
        self.width() * self.height()
    }

    /// Returns true if this bbox fully contains `other`.
    pub fn contains(&self, other: &BBox) -> bool {
        self.x0 <= other.x0
            && self.y0 <= other.y0
            && self.x1 >= other.x1
            && self.y1 >= other.y1
    }

    /// Returns true if this bbox overlaps with `other`.
    /// If `threshold` > 0.0, requires at least that fraction of the smaller bbox's
    /// area to be overlapped.
    pub fn overlaps(&self, other: &BBox, threshold: f64) -> bool {
        let ix0 = self.x0.max(other.x0);
        let iy0 = self.y0.max(other.y0);
        let ix1 = self.x1.min(other.x1);
        let iy1 = self.y1.min(other.y1);

        if ix0 >= ix1 || iy0 >= iy1 {
            return false;
        }

        if threshold > 0.0 {
            let intersect_area = (ix1 - ix0) * (iy1 - iy0);
            let smaller_area = self.area().min(other.area());
            intersect_area / smaller_area >= threshold
        } else {
            true
        }
    }

    /// Returns the minimal bbox enclosing both self and other.
    pub fn merge(&self, other: &BBox) -> BBox {
        BBox {
            x0: self.x0.min(other.x0),
            y0: self.y0.min(other.y0),
            x1: self.x1.max(other.x1),
            y1: self.y1.max(other.y1),
        }
    }

    /// Returns the intersection of self and other, or None if they don't overlap.
    pub fn intersect(&self, other: &BBox) -> Option<BBox> {
        let x0 = self.x0.max(other.x0);
        let y0 = self.y0.max(other.y0);
        let x1 = self.x1.min(other.x1);
        let y1 = self.y1.min(other.y1);

        if x0 < x1 && y0 < y1 {
            Some(BBox { x0, y0, x1, y1 })
        } else {
            None
        }
    }
}

impl fmt::Display for BBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BBox({:.1}, {:.1}, {:.1}, {:.1})",
            self.x0, self.y0, self.x1, self.y1
        )
    }
}

/// A 2D point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}
