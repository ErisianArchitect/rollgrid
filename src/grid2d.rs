use crate::fixedarray::FixedArray;
use crate::math::*;

/// A 2-Dimensional matrix of values.
pub struct Grid2D<T: Sized> {
    cells: FixedArray<T>,
    size: (u32, u32),
    offset: (i32, i32),
}

impl<T: Sized> Grid2D<T> {
    // pub fn new<F: FnMut((i32, i32)) -> T>(width: u32, height: u32, init: F) -> Self {
    //     Self {
    //         cells: FixedArray::new_2d((width as usize, height as usize), (0, 0), init)
    //     }
    // }
}
