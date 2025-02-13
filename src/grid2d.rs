use crate::bounds2d::*;
use crate::error_messages::*;
use crate::fixedarray::FixedArray;
use crate::math::*;

/// A 2-Dimensional matrix.
pub struct Grid2D<T> {
    cells: FixedArray<T>,
    size: (u32, u32),
    offset: (i32, i32),
}

impl<T> Grid2D<T> {
    /// Create a new [Grid2D] using a function to initialize cells.
    ///
    /// The init function should take as input the coordinate that is
    /// being initialized, and should return the desired value for the
    /// cell.
    pub fn new<F: FnMut((i32, i32)) -> T>(size: (u32, u32), offset: (i32, i32), init: F) -> Self {
        Self {
            cells: FixedArray::new_2d(size, offset, init),
            size,
            offset,
        }
    }

    /// The grid has an offset, so this function will find the index of the cell
    /// at the world coordinate `(x, y)`.
    pub fn offset_index(&self, (x, y): (i32, i32)) -> Option<usize> {
        let (x, y) = (x as i64, y as i64);
        let (off_x, off_y) = self.offset.convert::<(i64, i64)>();
        let width = self.size.0 as i64;
        let height = self.size.1 as i64;
        if x < off_x || y < off_y || x >= off_x + width || y >= off_y + height {
            return None;
        }
        let adj_x = x - off_x;
        let adj_y = y - off_y;
        Some((adj_y as usize * width as usize) + adj_x as usize)
    }

    /// Get the offset relative to the grid's offset.
    pub fn relative_offset(&self, coord: (i32, i32)) -> (i64, i64) {
        let (x, y) = coord.convert::<(i64, i64)>();
        let (ox, oy) = self.offset.convert::<(i64, i64)>();
        (x - ox, y - oy)
    }

    /// Replace item at `coord` using `replace` function that takes as
    /// input the old value and returns the new value. This will swap the
    /// value in-place.
    ///
    /// # Panics
    /// - When out of bounds, this method will panic.
    pub fn replace_with<F: FnOnce(T) -> T>(&mut self, coord: (i32, i32), replace: F) {
        let index = OUT_OF_BOUNDS.expect(self.offset_index(coord));
        self.cells.replace_with(index, replace);
    }

    /// Replace item at `coord` using [std::mem::replace] and then returns
    /// the old value.
    ///
    /// # Panics
    /// - When out of bounds, this method will panic.
    pub fn replace(&mut self, coord: (i32, i32), value: T) -> T {
        let index = OUT_OF_BOUNDS.expect(self.offset_index(coord));
        self.cells.replace(index, value)
    }

    /// Reads the value from the cell without moving it. This leaves the memory in the cell unchanged.
    #[must_use]
    pub unsafe fn read(&self, coord: (i32, i32)) -> T {
        let index = OUT_OF_BOUNDS.expect(self.offset_index(coord));
        self.cells.read(index)
    }

    /// Overwrites a cell at the given coordinate with the given value without reading or dropping the old value.
    ///
    /// This is safe, but it could leak allocations or resources, so care should be taken not to overwrite an object that should be dropped.
    ///
    /// Semantically, `value` is moved into the cell at the given coordinate.
    ///
    /// This is appropriate for initializing uninitialized cells, or overwriting memory that has previously been [read] from.
    pub unsafe fn write(&mut self, coord: (i32, i32), value: T) {
        let index = OUT_OF_BOUNDS.expect(self.offset_index(coord));
        self.cells.write(index, value);
    }

    /// Get a reference to the cell's value if it exists and the coord is in bounds, otherwise return `None`.
    pub fn get(&self, coord: (i32, i32)) -> Option<&T> {
        let index = self.offset_index(coord)?;
        Some(&self.cells[index])
    }

    /// Get a mutable reference to the cell's value if the coord is in bounds, otherwise return `None`.
    pub fn get_mut(&mut self, coord: (i32, i32)) -> Option<&mut T> {
        let index = self.offset_index(coord)?;
        Some(&mut self.cells[index])
    }

    /// Set the cell's value, returning the old value in the process.
    pub fn set(&mut self, coord: (i32, i32), value: T) -> Option<T> {
        let index = self.offset_index(coord)?;
        let dest = &mut self.cells[index];
        Some(std::mem::replace(dest, value))
    }

    /// Get the dimensions of the grid.
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    /// The size along the X axis.
    pub fn width(&self) -> u32 {
        self.size.0
    }

    /// The size along the Y axis.
    pub fn height(&self) -> u32 {
        self.size.1
    }

    /// Get the offset of the grid.
    pub fn offset(&self) -> (i32, i32) {
        self.offset
    }

    /// Get the minimum bound on the `X` axis.
    pub fn x_min(&self) -> i32 {
        self.offset.0
    }

    /// Get the maximum bound on the `X` axis.
    pub fn x_max(&self) -> i32 {
        add_u32_to_i32(self.offset.0, self.size.0)
    }

    /// Get the minimum bound on the `Y` axis.
    pub fn y_min(&self) -> i32 {
        self.offset.1
    }

    /// Get the maximum bound on the `Y` axis.
    pub fn y_max(&self) -> i32 {
        add_u32_to_i32(self.offset.1, self.size.1)
    }

    /// Get the bounds of the grid.
    pub fn bounds(&self) -> Bounds2D {
        Bounds2D {
            min: self.offset,
            max: (self.x_max(), self.y_max()),
        }
    }

    /// This is equivalent to the area (width * height).
    pub fn len(&self) -> usize {
        self.size.0 as usize * self.size.1 as usize
    }

    /// Get an iterator over the cells in the grid.
    pub fn iter(&self) -> Grid2DIterator<T> {
        Grid2DIterator {
            bounds_iter: self.bounds().iter(),
            grid: self,
        }
    }

    /// Get a mutable iterator over the cells in the grid.
    pub fn iter_mut(&mut self) -> Grid2DMutIterator<T> {
        Grid2DMutIterator {
            bounds_iter: self.bounds().iter(),
            grid: self,
        }
    }
}

impl<T: Copy> Grid2D<T> {
    /// Get a copy of the grid value.
    pub fn get_copy(&self, coord: (i32, i32)) -> Option<T> {
        let index = self.offset_index(coord)?;
        Some(self.cells[index])
    }
}

impl<T: Clone> Grid2D<T> {
    /// Get a clone of the grid value.
    pub fn get_clone(&self, coord: (i32, i32)) -> Option<T> {
        let index = self.offset_index(coord)?;
        Some(self.cells[index].clone())
    }
}

impl<T: Clone> Clone for Grid2D<T> {
    fn clone(&self) -> Self {
        Self {
            cells: self.cells.clone(),
            size: self.size,
            offset: self.offset,
        }
    }
}

impl<T> std::ops::Index<(i32, i32)> for Grid2D<T> {
    type Output = T;
    fn index(&self, index: (i32, i32)) -> &Self::Output {
        let index = OUT_OF_BOUNDS.expect(self.offset_index(index));
        &self.cells[index]
    }
}

impl<T> std::ops::IndexMut<(i32, i32)> for Grid2D<T> {
    fn index_mut(&mut self, index: (i32, i32)) -> &mut Self::Output {
        let index = OUT_OF_BOUNDS.expect(self.offset_index(index));
        &mut self.cells[index]
    }
}

impl<T> AsRef<Grid2D<T>> for Grid2D<T> {
    fn as_ref(&self) -> &Grid2D<T> {
        self
    }
}

impl<T> AsMut<Grid2D<T>> for Grid2D<T> {
    fn as_mut(&mut self) -> &mut Grid2D<T> {
        self
    }
}

unsafe impl<T: Send> Send for Grid2D<T> {}
unsafe impl<T: Sync> Sync for Grid2D<T> {}

pub struct Grid2DIterator<'a, T> {
    grid: &'a Grid2D<T>,
    bounds_iter: Bounds2DIter,
}

impl<'a, T> Iterator for Grid2DIterator<'a, T> {
    type Item = ((i32, i32), &'a T);

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.bounds_iter.size_hint()
    }

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.bounds_iter.next()?;
        let index = self.grid.offset_index(next)?;
        Some((next, &self.grid.cells[index]))
    }
}

pub struct Grid2DMutIterator<'a, T> {
    grid: &'a mut Grid2D<T>,
    bounds_iter: Bounds2DIter,
}

impl<'a, T> Iterator for Grid2DMutIterator<'a, T> {
    type Item = ((i32, i32), &'a mut T);

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.bounds_iter.size_hint()
    }

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.bounds_iter.next()?;
        let index = self.grid.offset_index(next)?;
        unsafe {
            let cells_ptr = self.grid.cells.as_mut_ptr();
            let cell_ptr = cells_ptr.add(index);
            Some((next, cell_ptr.as_mut().unwrap()))
        }
    }
}
