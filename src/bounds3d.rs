#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A 3D bounding box.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bounds3D {
    /// Inclusive minimum bound.
    pub min: (i32, i32, i32),
    /// Exclusive maximum bound.
    pub max: (i32, i32, i32),
}

impl Bounds3D {
    /// Create a new [Bounds3D] with the specified minimum and maximum bounds.
    pub fn new(min: (i32, i32, i32), max: (i32, i32, i32)) -> Self {
        Self { min, max }
    }

    /// Create a new [Bounds3D] from two unordered points.
    pub fn from_bounds(a: (i32, i32, i32), b: (i32, i32, i32)) -> Self {
        let x_min = a.0.min(b.0);
        let y_min = a.1.min(b.1);
        let z_min = a.2.min(b.2);
        let x_max = a.0.max(b.0);
        let y_max = a.1.max(b.1);
        let z_max = a.2.max(b.2);
        Self {
            min: (x_min, y_min, z_min),
            max: (x_max, y_max, z_max),
        }
    }

    /// The size along the X axis.
    pub fn width(&self) -> u32 {
        (self.max.0 as i64 - self.min.0 as i64) as u32
    }

    /// The size along the Y axis.
    pub fn height(&self) -> u32 {
        (self.max.1 as i64 - self.min.1 as i64) as u32
    }

    /// The size along the Z axis.
    pub fn depth(&self) -> u32 {
        (self.max.2 as i64 - self.min.2 as i64) as u32
    }

    /// The volume is `width * height * depth`.
    pub fn volume(&self) -> i128 {
        self.width() as i128 * self.height() as i128 * self.depth() as i128
    }

    /// The minumum bound along the `X` axis.
    pub fn x_min(&self) -> i32 {
        self.min.0
    }

    /// The minimum bound along the `Y` axis.
    pub fn y_min(&self) -> i32 {
        self.min.1
    }

    /// The minimum bound along the `Z` axis.
    pub fn z_min(&self) -> i32 {
        self.min.2
    }

    /// The maximum bound along the `X` axis (exclusive).
    pub fn x_max(&self) -> i32 {
        self.max.0
    }

    /// The maxmimum bound along the `Y` axis (exclusive).
    pub fn y_max(&self) -> i32 {
        self.max.1
    }

    /// The maximum bound along the `Z` axis (exclusive).
    pub fn z_max(&self) -> i32 {
        self.max.2
    }

    // intersects would need to copy self and other anyway, so
    // just accept copied values rather than references.
    /// Tests for intersection with another [Bounds3D].
    pub fn intersects(self, other: Bounds3D) -> bool {
        let (ax_min, ay_min, az_min) = self.min;
        let (ax_max, ay_max, az_max) = self.max;
        let (bx_min, by_min, bz_min) = other.min;
        let (bx_max, by_max, bz_max) = other.max;
        ax_min < bx_max
            && bx_min < ax_max
            && ay_min < by_max
            && by_min < ay_max
            && az_min < bz_max
            && bz_min < az_max
    }

    /// Determine if a point is within the [Bounds3D].
    pub fn contains(self, point: (i32, i32, i32)) -> bool {
        point.0 >= self.min.0
            && point.1 >= self.min.1
            && point.2 >= self.min.2
            && point.0 < self.max.0
            && point.1 < self.max.1
            && point.2 < self.max.2
    }

    /// Iterate over the points in the [Bounds3D].
    pub fn iter(self) -> Bounds3DIter {
        Bounds3DIter {
            bounds: self,
            current: self.min,
        }
    }
}

/// Iterator for all points within a [Bounds3D].
pub struct Bounds3DIter {
    bounds: Bounds3D,
    current: (i32, i32, i32),
}

impl Iterator for Bounds3DIter {
    type Item = (i32, i32, i32);

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.current.2 == self.bounds.max.2 {
            return (0, Some(0));
        }
        let (x, y, z) = (
            (self.current.0 - self.bounds.min.0) as usize,
            (self.current.1 - self.bounds.min.1) as usize,
            (self.current.2 - self.bounds.min.2) as usize,
        );
        let width = self.bounds.width() as usize;
        let depth = self.bounds.depth() as usize;
        let volume = self.bounds.volume() as usize;
        let index = y * width * depth + z * width + x;
        (volume - index, Some(volume - index))
    }

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.1 == self.bounds.max.1 {
            return None;
        }
        let result = self.current;
        // inc x, then z, then y
        self.current = if result.0 + 1 == self.bounds.max.0 {
            if result.2 + 1 == self.bounds.max.2 {
                (self.bounds.min.0, result.1 + 1, self.bounds.min.2)
            } else {
                (self.bounds.min.0, result.1, result.2 + 1)
            }
        } else {
            (result.0 + 1, result.1, result.2)
        };
        Some(result)
    }
}
