#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A 2D bounding box. Essentially a rectangle.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bounds2D {
    /// Inclusive minimum bound.
    pub min: (i32, i32),
    /// Exclusive maximum bound.
    pub max: (i32, i32),
}

impl Bounds2D {
    /// Create a new [Bounds2D] from an inclusive min and exclusive max.
    /// If you don't know the min/max bounds, you can use `from_bounds`
    /// to create a [Bounds2D] from arbitrary coordinates.
    pub fn new(min: (i32, i32), max: (i32, i32)) -> Self {
        Self { min: min, max: max }
    }

    /// Create a new [Bounds2D] by resolving the inclusive min and exclusive max from two coordinates.
    pub fn from_bounds(a: (i32, i32), b: (i32, i32)) -> Self {
        let (ax, ay) = a;
        let (bx, by) = b;
        let min = (ax.min(bx), ay.min(by));
        let max = (ax.max(bx), ay.max(by));
        Self { min, max }
    }

    /// The size along the X axis.
    pub fn width(&self) -> u32 {
        (self.max.0 as i64 - self.min.0 as i64) as u32
    }

    /// The size along the Y axis.
    pub fn height(&self) -> u32 {
        (self.max.1 as i64 - self.min.1 as i64) as u32
    }

    /// `width` * `height`.
    pub fn area(&self) -> i64 {
        self.width() as i64 * self.height() as i64
    }

    /// The minimum bound on the X axis.
    pub fn x_min(&self) -> i32 {
        self.min.0
    }

    /// The minimum bound on the Y axis.
    pub fn y_min(&self) -> i32 {
        self.min.1
    }

    /// The maximum bound on the X axis (exclusive).
    pub fn x_max(&self) -> i32 {
        self.max.0
    }

    /// The maximum bound on the Y axis (exclusive).
    pub fn y_max(&self) -> i32 {
        self.max.1
    }

    // intersects would need to copy self and other anyway, so
    // just accept copied values rather than references.
    /// Tests for intersection with another [Bounds2D].
    pub fn intersects(self, other: Bounds2D) -> bool {
        let ((ax_min, ay_min), (ax_max, ay_max)) = (self.min, self.max);
        let ((bx_min, by_min), (bx_max, by_max)) = (other.min, other.max);
        ax_min < bx_max && bx_min < ax_max && ay_min < by_max && by_min < ay_max
    }

    /// Determine if a point is within the [Bounds2D].
    pub fn contains(self, point: (i32, i32)) -> bool {
        point.0 >= self.min.0
            && point.1 >= self.min.0
            && point.0 < self.max.0
            && point.1 < self.max.1
    }

    /// Iterate the coordinates in the [Bounds2D].
    pub fn iter(self) -> Bounds2DIter {
        Bounds2DIter {
            bounds: self,
            current: self.min,
        }
    }
}

/// Iterator for all points within a [Bounds2D].
pub struct Bounds2DIter {
    bounds: Bounds2D,
    current: (i32, i32),
}

impl Iterator for Bounds2DIter {
    type Item = (i32, i32);

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.current.1 == self.bounds.max.1 {
            return (0, Some(0));
        }
        let (x, y) = (
            self.current.0 - self.bounds.min.0,
            self.current.1 - self.bounds.min.1,
        );
        let width = self.bounds.max.0 - self.bounds.min.0;
        let height = self.bounds.max.1 - self.bounds.min.1;
        let size = (width * height) as usize;
        let index = (y * width + x) as usize;
        (size - index, Some(size - index))
    }

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.1 == self.bounds.max.1 {
            return None;
        }
        let result = self.current;
        self.current = (result.0 + 1, result.1);
        if self.current.0 == self.bounds.max.0 {
            self.current = (self.bounds.min.0, result.1 + 1);
        }
        Some(result)
    }
}
