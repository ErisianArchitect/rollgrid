#![allow(unused)]

use crate::{SIZE_TOO_LARGE, OFFSET_TOO_CLOSE_TO_MAX};
const VOLUME_IS_ZERO: &'static str = "Width/Height/Depth cannot be 0";

type Coord = (i32, i32, i32);

pub struct RollGrid3D<T> {
    cells: Vec<Option<T>>,
    size: (usize, usize, usize),
    wrap_offset: (usize, usize, usize),
    grid_offset: (i32, i32, i32),
}

impl<T: Default> RollGrid3D<T> {
    pub fn new_default<C: Into<Coord>>(width: usize, height: usize, depth: usize, grid_offset: C) -> Self {
        let grid_offset: Coord = grid_offset.into();
        let volume = width.checked_mul(height).expect(SIZE_TOO_LARGE).checked_mul(depth).expect(SIZE_TOO_LARGE);
        if volume == 0 {
            panic!("{VOLUME_IS_ZERO}");
        }
        Self {
            cells: (0..volume).map(|_| Some(T::default())).collect(),
            size: (width, height, depth),
            grid_offset,
            wrap_offset: (0, 0, 0)
        }
    }
}

impl<T> RollGrid3D<T> {
    pub fn new<C: Into<Coord>>(
        width: usize,
        height: usize,
        depth: usize,
        grid_offset: C
    ) -> Self {
        let grid_offset: Coord = grid_offset.into();
        let volume = width.checked_mul(height).expect(SIZE_TOO_LARGE).checked_mul(depth).expect(SIZE_TOO_LARGE);
        if volume == 0 {
            panic!("{VOLUME_IS_ZERO}");
        }
        if volume > i32::MAX as usize {
            panic!("{SIZE_TOO_LARGE}");
        }
        if grid_offset.0.checked_add(width as i32).is_none()
        || grid_offset.1.checked_add(height as i32).is_none()
        || grid_offset.2.checked_add(depth as i32).is_none() {
            panic!("{OFFSET_TOO_CLOSE_TO_MAX}");
        }
        Self {
            cells: (0..volume).map(|_| None).collect(),
            size: (width, height, depth),
            wrap_offset: (0, 0, 0),
            grid_offset
        }
    }
}

struct TempGrid3D<T> {
    pub cells: Vec<Option<T>>,
    pub size: (usize, usize, usize),
    pub offset: (i32, i32, i32),
}

impl<T> TempGrid3D<T> {
    pub fn new(size: (usize, usize, usize), offset: (i32, i32, i32)) -> Self {
        Self {
            cells: (0..size.0*size.1*size.2).map(|_| None).collect(),
            size,
            offset
        }
    }

    fn offset_index(&self, (x, y, z): (i32, i32, i32)) -> Option<usize> {
        let (mx, my, mz) = self.offset;
        let width = self.size.0 as i32;
        let height = self.size.1 as i32;
        let depth = self.size.2 as i32;
        if x < mx
        || y < my
        || z < mz
        || x >= mx + width
        || y >= my + height
        || z >= mz + depth {
            return None;
        }
        // Adjust x and y
        let nx = x - mx;
        let ny = y - my;
        let nz = z - mz;
        let plane = self.size.0 * self.size.2;
        Some(ny as usize * plane + nz as usize * self.size.0 + nx as usize)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bounds3D {
    pub min: (i32, i32, i32),
    pub max: (i32, i32, i32)
}

impl Bounds3D {
    pub fn new(min: (i32, i32, i32), max: (i32, i32, i32)) -> Self {
        Self {
            min,
            max
        }
    }

    pub fn from_bounds(a: (i32, i32, i32), b: (i32, i32, i32)) -> Self {
        let x_min = a.0.min(b.0);
        let y_min = a.1.min(b.1);
        let z_min = a.2.min(b.2);
        let x_max = a.0.max(b.0);
        let y_max = a.1.max(b.1);
        let z_max = a.2.max(b.2);
        Self {
            min: (x_min, y_min, z_min),
            max: (x_max, y_max, z_max)
        }
    }

    /// The size along the X axis.
    pub fn width(&self) -> i32 {
        self.max.0 - self.min.0
    }

    /// The size along the Y axis.
    pub fn height(&self) -> i32 {
        self.max.1 - self.min.1
    }

    /// The size along the Z axis.
    pub fn depth(&self) -> i32 {
        self.max.2 - self.min.2
    }

    pub fn volume(&self) -> i32 {
        self.width() * self.height() * self.depth()
    }

    pub fn x_min(&self) -> i32 {
        self.min.0
    }

    pub fn y_min(&self) -> i32 {
        self.min.1
    }
    
    pub fn z_min(&self) -> i32 {
        self.min.2
    }

    pub fn x_max(&self) -> i32 {
        self.max.0
    }

    pub fn y_max(&self) -> i32 {
        self.max.1
    }

    pub fn z_max(&self) -> i32 {
        self.max.2
    }

    // intersects would need to copy self and other anyway, so
    // just accept copied values rather than references.
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

    pub fn contains(self, point: (i32, i32, i32)) -> bool {
        point.0 >= self.min.0
        && point.1 >= self.min.1
        && point.2 >= self.min.2
        && point.0 < self.max.0
        && point.1 < self.max.1
        && point.2 < self.max.2
    }

    pub fn iter(self) -> Bounds3DIter {
        Bounds3DIter { bounds: self, current: self.min }
    }
}

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
            (self.current.2 - self.bounds.min.2) as usize
        );
        let width = self.bounds.width() as usize;
        let height = self.bounds.height() as usize;
        let depth = self.bounds.depth() as usize;
        let volume = self.bounds.volume() as usize;
        let index = (y * width * depth + z * width + x);
        (volume - index, Some(volume - index))
    }

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.1 == self.bounds.max.1 {
            return None;
        }
        let result = self.current;
        // inc x, then z, then y
        // self.current = (result.0 + 1, result.1, result.2);
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

#[test]
fn bounds_test() {
    let bounds = Bounds3D::new((0, 0, 0), (3, 3, 3));
    bounds.iter().for_each(|(x, y, z)| {
        println!("({x},{y},{z})");
    });
}