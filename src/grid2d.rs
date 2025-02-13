use crate::fixedarray::FixedArray;
use crate::math::*;

// TODO: There's a lot of work to be done here.
//       Ideally, `Grid2D` can be used to represent a slice of
//       a `RollGrid2D`. I'd like to be able to use this struct
//       to store a subgrid of references or mutable references.

/// A 2-Dimensional matrix of values.
pub struct Grid2D<T: Sized> {
    cells: FixedArray<T>,
    size: (u32, u32),
    offset: (i32, i32),
}

impl<T: Sized> Grid2D<T> {
    pub fn new<F: FnMut((i32, i32)) -> T>(size: (u32, u32), offset: (i32, i32), init: F) -> Self {
        Self {
            cells: FixedArray::new_2d(size, offset, init),
            size,
            offset,
        }
    }
}
