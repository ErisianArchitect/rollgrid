use crate::grid3d::*;
use crate::{
    bounds3d::*,
    error_messages::*,
    fixedarray::FixedArray,
    math::{add_u32_to_i32, checked_add_u32_to_i32, Convert},
    *,
};

/// A 3D implementation of a rolling grid. It's a data structure similar
/// to a circular buffer in the sense that cells can wrap around.
/// It uses the modulus operator combined with an internal wrap offset to
/// create the illusion that cells are being moved while the cells remain
/// in the same position in the underlying array.
pub struct RollGrid3D<T> {
    cells: FixedArray<T>,
    size: (u32, u32, u32),
    wrap_offset: (u32, u32, u32),
    grid_offset: (i32, i32, i32),
}

unsafe impl<T: Send> Send for RollGrid3D<T> {}
unsafe impl<T: Sync> Sync for RollGrid3D<T> {}

impl<T: Default> RollGrid3D<T> {
    /// Create a new [RollGrid3D] with all the cells set to the default for `T`.
    pub fn new_default(size: (u32, u32, u32), grid_offset: (i32, i32, i32)) -> Self {
        Self {
            cells: FixedArray::new_3d(size, grid_offset, |_| T::default()),
            size,
            grid_offset,
            wrap_offset: (0, 0, 0),
        }
    }
}

impl RollGrid3D<()> {
    /// Creates a new grid of unit types.
    pub fn new_zst(size: (u32, u32, u32), grid_offset: (i32, i32, i32)) -> Self {
        RollGrid3D {
            cells: FixedArray::new_3d(size, grid_offset, |_| ()),
            size,
            grid_offset,
            wrap_offset: (0, 0, 0),
        }
    }
}

impl<T> RollGrid3D<T> {
    /// Create a new [RollGrid3D] using a function to initialize cells.
    ///
    /// The init function should take as input the coordinate that is
    /// being initialized, and should return the desired value for the
    /// cell.
    pub fn new<F: FnMut((i32, i32, i32)) -> T>(
        size: (u32, u32, u32),
        grid_offset: (i32, i32, i32),
        init: F,
    ) -> Self {
        Self {
            cells: FixedArray::new_3d(size, grid_offset, init),
            size,
            wrap_offset: (0, 0, 0),
            grid_offset,
        }
    }

    /// Try to create a new [RollGrid3D] with a fallible function to initialize cells.
    ///
    /// The init function should take as input the coordinate that is being
    /// initialized, and should return the desired value for the cell.
    pub fn try_new<E, F: FnMut((i32, i32, i32)) -> Result<T, E>>(
        size: (u32, u32, u32),
        grid_offset: (i32, i32, i32),
        init: F,
    ) -> Result<Self, E> {
        Ok(Self {
            cells: FixedArray::try_new_3d(size, grid_offset, init)?,
            size,
            wrap_offset: (0, 0, 0),
            grid_offset,
        })
    }

    /// Inflate the size by `inflate`, keeping the bounds centered.
    ///
    /// If the size is `(2, 2, 2)` with an offset of `(1, 1, 1)`, and you want to inflate by `(1, 1, 1)`.
    /// The result of that operation would have a size of `(4, 4, 4)` and an offset of `(0, 0, 0)`.
    ///
    /// # Example
    /// ```rust, no_run
    /// grid.inflate_size((1, 1, 1), cell_manager(
    ///     // Load
    ///     |pos| {
    ///         println!("Load: {:?}", pos);
    ///         // return the loaded value
    ///         // Typically you wouldn't return the position,
    ///         // you would want to load a new cell here.
    ///         pos
    ///     },
    ///     // Unload
    ///     |pos, old_value| {
    ///         println!("Unload: {:?}", pos);
    ///     },
    ///     // Reload
    ///     |old_pos, new_pos, cell| {
    ///         println!("Reload({:?}, {:?})")
    ///     }
    /// ))
    /// ```
    /// See [CellManage].
    pub fn inflate_size<M>(&mut self, inflate: (u32, u32, u32), manage: M)
    where
        M: CellManage<(i32, i32, i32), T>,
    {
        // let inf = inflate as i32;
        let width = self
            .size
            .0
            .checked_add(inflate.0.checked_mul(2).expect(INFLATE_OVERFLOW.msg()))
            .expect(INFLATE_OVERFLOW.msg());
        let height = self
            .size
            .1
            .checked_add(inflate.1.checked_mul(2).expect(INFLATE_OVERFLOW.msg()))
            .expect(INFLATE_OVERFLOW.msg());
        let depth = self
            .size
            .2
            .checked_add(inflate.2.checked_mul(2).expect(INFLATE_OVERFLOW.msg()))
            .expect(INFLATE_OVERFLOW.msg());
        let off_x = self.grid_offset.0 as i64;
        let off_y = self.grid_offset.1 as i64;
        let off_z = self.grid_offset.2 as i64;
        let pos_x = off_x - inflate.0 as i64;
        INFLATE_OVERFLOW.panic_if(pos_x < i32::MIN as i64);
        let pos_y = off_y - inflate.1 as i64;
        INFLATE_OVERFLOW.panic_if(pos_y < i32::MIN as i64);
        let pos_z = off_z - inflate.2 as i64;
        INFLATE_OVERFLOW.panic_if(pos_z < i32::MIN as i64);
        let right = pos_x + width as i64;
        INFLATE_OVERFLOW.panic_if(right > i32::MAX as i64);
        let bottom = pos_y + height as i64;
        INFLATE_OVERFLOW.panic_if(bottom > i32::MAX as i64);
        let back = pos_z + depth as i64;
        INFLATE_OVERFLOW.panic_if(back > i32::MAX as i64);
        let position = (pos_x as i32, pos_y as i32, pos_z as i32);
        self.resize_and_reposition((width, height, depth), position, manage);
    }

    /// Try to inflate the size by `inflate` using a fallible function, keeping the bounds centered.
    ///
    /// If the size is `(2, 2, 2)` with an offset of `(1, 1, 1)`, and you want to inflate by `(1, 1, 1)`.
    /// The result of that operation would have a size of `(4, 4, 4)` and an offset of `(0, 0, 0)`.
    ///
    /// # Example
    /// ```rust, no_run
    /// grid.try_inflate_size((1, 1, 1), try_cell_manager(
    ///     // Load
    ///     |pos| {
    ///         println!("Load: {:?}", pos);
    ///         // return the loaded value
    ///         // Typically you wouldn't return the position,
    ///         // you would want to load a new cell here.
    ///         Ok(pos)
    ///     },
    ///     // Unload
    ///     |pos, old_value| {
    ///         println!("Unload: {:?}", pos);
    ///         Ok(())
    ///     },
    ///     // Reload
    ///     |old_pos, new_pos, cell| {
    ///         println!("Reload({:?}, {:?})")
    ///         Ok(())
    ///     }
    /// ))
    /// ```
    /// See [TryCellManage].
    pub fn try_inflate_size<E, M>(&mut self, inflate: (u32, u32, u32), manage: M) -> Result<(), E>
    where
        M: TryCellManage<(i32, i32, i32), T, E>,
    {
        // let inf = inflate as i32;
        let width = self
            .size
            .0
            .checked_add(inflate.0.checked_mul(2).expect(INFLATE_OVERFLOW.msg()))
            .expect(INFLATE_OVERFLOW.msg());
        let height = self
            .size
            .1
            .checked_add(inflate.1.checked_mul(2).expect(INFLATE_OVERFLOW.msg()))
            .expect(INFLATE_OVERFLOW.msg());
        let depth = self
            .size
            .2
            .checked_add(inflate.2.checked_mul(2).expect(INFLATE_OVERFLOW.msg()))
            .expect(INFLATE_OVERFLOW.msg());
        let off_x = self.grid_offset.0 as i64;
        let off_y = self.grid_offset.1 as i64;
        let off_z = self.grid_offset.2 as i64;
        let pos_x = off_x - inflate.0 as i64;
        INFLATE_OVERFLOW.panic_if(pos_x < i32::MIN as i64);
        let pos_y = off_y - inflate.1 as i64;
        INFLATE_OVERFLOW.panic_if(pos_y < i32::MIN as i64);
        let pos_z = off_z - inflate.2 as i64;
        INFLATE_OVERFLOW.panic_if(pos_z < i32::MIN as i64);
        let right = pos_x + width as i64;
        INFLATE_OVERFLOW.panic_if(right > i32::MAX as i64);
        let bottom = pos_y + height as i64;
        INFLATE_OVERFLOW.panic_if(bottom > i32::MAX as i64);
        let back = pos_z + depth as i64;
        INFLATE_OVERFLOW.panic_if(back > i32::MAX as i64);
        let position = (pos_x as i32, pos_y as i32, pos_z as i32);
        self.try_resize_and_reposition((width, height, depth), position, manage)
    }

    /// Deflate the size by `deflate`, keeping the bounds centered.
    ///
    /// If the size is `(4, 4, 4)` with an offset of `(0, 0, 0)`, and you want to deflate by `(1, 1, 1)`.
    /// The result of that operation would have a size of `(2, 2, 2)` and an offset of `(1, 1, 1)`.
    ///
    /// # Example
    /// ```rust, no_run
    /// grid.deflate_size((1, 1, 1), cell_manager(
    ///     // Load
    ///     |pos| {
    ///         println!("Load: {:?}", pos);
    ///         // return the loaded value
    ///         // Typically you wouldn't return the position,
    ///         // you would want to load a new cell here.
    ///         pos
    ///     },
    ///     // Unload
    ///     |pos, old_value| {
    ///         println!("Unload: {:?}", pos);
    ///     },
    ///     // Reload
    ///     |old_pos, new_pos, cell| {
    ///         println!("Reload({:?}, {:?})")
    ///     }
    /// ))
    /// ```
    /// See [CellManage].
    pub fn deflate_size<M>(&mut self, deflate: (u32, u32, u32), manage: M)
    where
        M: CellManage<(i32, i32, i32), T>,
    {
        let width = self
            .size
            .0
            .checked_sub(deflate.0.checked_mul(2).expect(DEFLATE_OVERFLOW.msg()))
            .expect(DEFLATE_OVERFLOW.msg());
        let height = self
            .size
            .1
            .checked_sub(deflate.1.checked_mul(2).expect(DEFLATE_OVERFLOW.msg()))
            .expect(DEFLATE_OVERFLOW.msg());
        let depth = self
            .size
            .2
            .checked_sub(deflate.2.checked_mul(2).expect(DEFLATE_OVERFLOW.msg()))
            .expect(DEFLATE_OVERFLOW.msg());
        VOLUME_IS_ZERO.panic_if(width == 0 || height == 0 || depth == 0);
        let (off_x, off_y, off_z): (i64, i64, i64) = self.grid_offset.convert();
        let pos_x = off_x + deflate.0 as i64;
        DEFLATE_OVERFLOW.panic_if(pos_x > i32::MAX as i64);
        let pos_y = off_y + deflate.1 as i64;
        DEFLATE_OVERFLOW.panic_if(pos_y > i32::MAX as i64);
        let pos_z = off_z + deflate.2 as i64;
        DEFLATE_OVERFLOW.panic_if(pos_z > i32::MAX as i64);
        let position = (
            self.grid_offset.0 + deflate.0 as i32,
            self.grid_offset.1 + deflate.1 as i32,
            self.grid_offset.2 + deflate.2 as i32,
        );
        self.resize_and_reposition((width, height, depth), position, manage);
    }

    /// Try to deflate the size by `deflate` using a fallible function, keeping the bounds centered.
    ///
    /// If the size is `(4, 4, 4)` with an offset of `(0, 0, 0)`, and you want to deflate by `(1, 1, 1)`.
    /// The result of that operation would have a size of `(2, 2, 2)` and an offset of `(1, 1, 1)`.
    ///
    /// # Example
    /// ```rust, no_run
    /// grid.try_deflate_size((1, 1, 1), try_cell_manager(
    ///     // Load
    ///     |pos| {
    ///         println!("Load: {:?}", pos);
    ///         // return the loaded value
    ///         // Typically you wouldn't return the position,
    ///         // you would want to load a new cell here.
    ///         Ok(pos)
    ///     },
    ///     // Unload
    ///     |pos, old_value| {
    ///         println!("Unload: {:?}", pos);
    ///         Ok(())
    ///     },
    ///     // Reload
    ///     |old_pos, new_pos, cell| {
    ///         println!("Reload({:?}, {:?})")
    ///         Ok(())
    ///     }
    /// ))
    /// ```
    /// See [TryCellManage].
    pub fn try_deflate_size<E, M>(&mut self, deflate: (u32, u32, u32), manage: M) -> Result<(), E>
    where
        M: TryCellManage<(i32, i32, i32), T, E>,
    {
        let width = self
            .size
            .0
            .checked_sub(deflate.0.checked_mul(2).expect(DEFLATE_OVERFLOW.msg()))
            .expect(DEFLATE_OVERFLOW.msg());
        let height = self
            .size
            .1
            .checked_sub(deflate.1.checked_mul(2).expect(DEFLATE_OVERFLOW.msg()))
            .expect(DEFLATE_OVERFLOW.msg());
        let depth = self
            .size
            .2
            .checked_sub(deflate.2.checked_mul(2).expect(DEFLATE_OVERFLOW.msg()))
            .expect(DEFLATE_OVERFLOW.msg());
        VOLUME_IS_ZERO.panic_if(width == 0 || height == 0 || depth == 0);
        let (off_x, off_y, off_z): (i64, i64, i64) = self.grid_offset.convert();
        let pos_x = off_x + deflate.0 as i64;
        DEFLATE_OVERFLOW.panic_if(pos_x > i32::MAX as i64);
        let pos_y = off_y + deflate.1 as i64;
        DEFLATE_OVERFLOW.panic_if(pos_y > i32::MAX as i64);
        let pos_z = off_z + deflate.2 as i64;
        DEFLATE_OVERFLOW.panic_if(pos_z > i32::MAX as i64);
        let position = (
            self.grid_offset.0 + deflate.0 as i32,
            self.grid_offset.1 + deflate.1 as i32,
            self.grid_offset.2 + deflate.2 as i32,
        );
        self.try_resize_and_reposition((width, height, depth), position, manage)
    }

    /// Resize the grid without changing the offset.
    ///
    /// # Example
    /// ```rust, no_run
    /// grid.resize(1, 1, 1, cell_manager(
    ///     // Load
    ///     |pos| {
    ///         println!("Load: {:?}", pos);
    ///         // return the loaded value
    ///         // Typically you wouldn't return the position,
    ///         // you would want to load a new cell here.
    ///         pos
    ///     },
    ///     // Unload
    ///     |pos, old_value| {
    ///         println!("Unload: {:?}", pos);
    ///     },
    ///     // Reload
    ///     |old_pos, new_pos, cell| {
    ///         println!("Reload({:?}, {:?})")
    ///     }
    /// ))
    /// ```
    /// See [CellManage].
    pub fn resize<M>(&mut self, size: (u32, u32, u32), manage: M)
    where
        M: CellManage<(i32, i32, i32), T>,
    {
        self.resize_and_reposition(size, self.grid_offset, manage);
    }

    /// Try to resize the grid with a fallible function without changing the offset.
    ///
    /// # Example
    /// ```rust, no_run
    /// grid.try_resize(1, 1, 1, cell_manager(
    ///     // Load
    ///     |pos| {
    ///         println!("Load: {:?}", pos);
    ///         // return the loaded value
    ///         // Typically you wouldn't return the position,
    ///         // you would want to load a new cell here.
    ///         Ok(pos)
    ///     },
    ///     // Unload
    ///     |pos, old_value| {
    ///         println!("Unload: {:?}", pos);
    ///         Ok(())
    ///     },
    ///     // Reload
    ///     |old_pos, new_pos, cell| {
    ///         println!("Reload({:?}, {:?})")
    ///         Ok(())
    ///     }
    /// ))
    /// ```
    /// See [TryCellManage].
    pub fn try_resize<E, M>(&mut self, size: (u32, u32, u32), manage: M) -> Result<(), E>
    where
        M: TryCellManage<(i32, i32, i32), T, E>,
    {
        self.try_resize_and_reposition(size, self.grid_offset, manage)
    }

    /// Resize and reposition the grid simultaneously.
    ///
    /// # Example
    /// ```rust, no_run
    /// grid.resize_and_reposition(3, 3, 3, (4, 4, 4), cell_manager(
    ///     // Load
    ///     |pos| {
    ///         println!("Load: {:?}", pos);
    ///         // return the loaded value
    ///         // Typically you wouldn't return the position,
    ///         // you would want to load a new cell here.
    ///         pos
    ///     },
    ///     // Unload
    ///     |pos, old_value| {
    ///         println!("Unload: {:?}", pos);
    ///     },
    ///     // Reload
    ///     |old_pos, new_pos, cell| {
    ///         println!("Reload({:?}, {:?})")
    ///     }
    /// ))
    /// ```
    /// See [CellManage].
    pub fn resize_and_reposition<M>(
        &mut self,
        size: (u32, u32, u32),
        new_position: (i32, i32, i32),
        manage: M,
    ) where
        M: CellManage<(i32, i32, i32), T>,
    {
        let mut manage = manage;
        let (width, height, depth) = size;
        if size == self.size {
            if new_position != self.grid_offset {
                self.reposition(new_position, |old_pos, new_pos, cell| {
                    manage.reload(old_pos, new_pos, cell);
                });
            }
            return;
        }
        let volume = (width as usize)
            .checked_mul(height as usize)
            .expect(SIZE_TOO_LARGE.msg())
            .checked_mul(depth as usize)
            .expect(SIZE_TOO_LARGE.msg());
        VOLUME_IS_ZERO.panic_if(volume == 0);
        let (new_x, new_y, new_z) = new_position;
        let right = RESIZE_OVERFLOW.expect(checked_add_u32_to_i32(new_x, width));
        let bottom = RESIZE_OVERFLOW.expect(checked_add_u32_to_i32(new_y, height));
        let back = RESIZE_OVERFLOW.expect(checked_add_u32_to_i32(new_z, depth));
        let old_bounds = self.bounds();
        let new_bounds = Bounds3D::new((new_x, new_y, new_z), (right, bottom, back));
        if old_bounds.intersects(new_bounds) {
            macro_rules! unload_bounds {
                ($cond:expr => xmin = $xmin:expr; ymin = $ymin:expr; zmin = $zmin:expr; xmax = $xmax:expr; ymax = $ymax:expr; zmax = $zmax:expr;) => {
                    if $cond {
                        Bounds3D::new(($xmin, $ymin, $zmin), ($xmax, $ymax, $zmax))
                            .iter()
                            .for_each(|pos| {
                                let index = self.offset_index(pos).expect(OUT_OF_BOUNDS.msg());
                                unsafe {
                                    manage.unload(pos, self.cells.read(index));
                                }
                            });
                    }
                };
            }
            // Y+ region
            unload_bounds!(old_bounds.y_max() > new_bounds.y_max() =>
                xmin = old_bounds.x_min();
                ymin = new_bounds.y_max();
                zmin = old_bounds.z_min();
                xmax = old_bounds.x_max();
                ymax = old_bounds.y_max();
                zmax = old_bounds.z_max();
            );
            // Y- region
            unload_bounds!(old_bounds.y_min() < new_bounds.y_min() =>
                xmin = old_bounds.x_min();
                ymin = old_bounds.y_min();
                zmin = old_bounds.z_min();
                xmax = old_bounds.x_max();
                ymax = new_bounds.y_min();
                zmax = old_bounds.z_max();
            );
            // Z+ region (row)
            unload_bounds!(old_bounds.z_max() > new_bounds.z_max() =>
                xmin = old_bounds.x_min();
                ymin = new_bounds.y_min().max(old_bounds.y_min());
                zmin = new_bounds.z_max();
                xmax = old_bounds.x_max();
                ymax = new_bounds.y_max().min(old_bounds.y_max());
                zmax = old_bounds.z_max();
            );
            // Z- region (row)
            unload_bounds!(old_bounds.z_min() < new_bounds.z_min() =>
                xmin = old_bounds.x_min();
                ymin = new_bounds.y_min().max(old_bounds.y_min());
                zmin = old_bounds.z_min();
                xmax = old_bounds.x_max();
                ymax = new_bounds.y_max().min(old_bounds.y_max());
                zmax = new_bounds.z_min();
            );
            // X+ region (cube)
            unload_bounds!(old_bounds.x_max() > new_bounds.x_max() =>
                xmin = new_bounds.x_max();
                ymin = new_bounds.y_min().max(old_bounds.y_min());
                zmin = new_bounds.z_min().max(old_bounds.z_min());
                xmax = old_bounds.x_max();
                ymax = new_bounds.y_max().min(old_bounds.y_max());
                zmax = new_bounds.z_max().min(old_bounds.z_max());
            );
            // X- region (cube)
            unload_bounds!(old_bounds.x_min() < new_bounds.x_min() =>
                xmin = old_bounds.x_min();
                ymin = new_bounds.y_min().max(old_bounds.y_min());
                zmin = new_bounds.z_min().max(old_bounds.z_min());
                xmax = new_bounds.x_min();
                ymax = new_bounds.y_max().min(old_bounds.y_max());
                zmax = new_bounds.z_max().min(old_bounds.z_max());
            );
            let new_grid = FixedArray::new_3d(size, new_position, |pos| {
                if old_bounds.contains(pos) {
                    let index = self.offset_index(pos).expect(OUT_OF_BOUNDS.msg());
                    unsafe { self.cells.read(index) }
                } else {
                    manage.load(pos)
                }
            });
            self.size = size;
            self.grid_offset = new_position;
            unsafe {
                self.cells.forget_dealloc();
            }
            self.cells = new_grid;
            self.wrap_offset = (0, 0, 0);
        } else {
            // !old_bounds.intersects(new_bounds)
            old_bounds.iter().for_each(|pos| {
                let index = self.offset_index(pos).expect(OUT_OF_BOUNDS.msg());
                unsafe {
                    manage.unload(pos, self.cells.read(index));
                }
            });
            let new_grid = FixedArray::new_3d(size, new_position, |pos| manage.load(pos));
            self.size = size;
            self.grid_offset = new_position;
            unsafe {
                self.cells.forget_dealloc();
            }
            self.cells = new_grid;
            self.wrap_offset = (0, 0, 0);
        }
    }

    /// Try to resize and reposition the grid using a fallible function.
    ///
    /// # Example
    /// ```rust, no_run
    /// grid.try_resize_and_reposition(3, 3, 3, (4, 4, 4), try_cell_manager(
    ///     // Load
    ///     |pos| {
    ///         println!("Load: {:?}", pos);
    ///         // return the loaded value
    ///         // Typically you wouldn't return the position,
    ///         // you would want to load a new cell here.
    ///         Ok(pos)
    ///     },
    ///     // Unload
    ///     |pos, old_value| {
    ///         println!("Unload: {:?}", pos);
    ///         Ok(())
    ///     },
    ///     // Reload
    ///     |old_pos, new_pos, cell| {
    ///         println!("Reload({:?}, {:?})")
    ///         Ok(())
    ///     }
    /// ))
    /// ```
    /// See [TryCellManage].
    pub fn try_resize_and_reposition<E, M>(
        &mut self,
        size: (u32, u32, u32),
        new_position: (i32, i32, i32),
        manage: M,
    ) -> Result<(), E>
    where
        M: TryCellManage<(i32, i32, i32), T, E>,
    {
        let mut manage = manage;
        let (width, height, depth) = size;
        if (width, height, depth) == self.size {
            if new_position != self.grid_offset {
                self.try_reposition(new_position, |old_pos, new_pos, cell| {
                    manage.try_reload(old_pos, new_pos, cell)
                })?;
            }
            return Ok(());
        }
        let volume = (width as usize)
            .checked_mul(height as usize)
            .expect(SIZE_TOO_LARGE.msg())
            .checked_mul(depth as usize)
            .expect(SIZE_TOO_LARGE.msg());
        VOLUME_IS_ZERO.panic_if(volume == 0);
        let (new_x, new_y, new_z) = new_position;
        let right = RESIZE_OVERFLOW.expect(checked_add_u32_to_i32(new_x, width));
        let bottom = RESIZE_OVERFLOW.expect(checked_add_u32_to_i32(new_y, height));
        let back = RESIZE_OVERFLOW.expect(checked_add_u32_to_i32(new_z, depth));
        let old_bounds = self.bounds();
        let new_bounds = Bounds3D::new((new_x, new_y, new_z), (right, bottom, back));
        if old_bounds.intersects(new_bounds) {
            macro_rules! unload_bounds {
                ($cond:expr => xmin = $xmin:expr; ymin = $ymin:expr; zmin = $zmin:expr; xmax = $xmax:expr; ymax = $ymax:expr; zmax = $zmax:expr;) => {
                    if $cond {
                        Bounds3D::new(($xmin, $ymin, $zmin), ($xmax, $ymax, $zmax))
                            .iter()
                            .try_for_each(|pos| {
                                let index = self.offset_index(pos).expect(OUT_OF_BOUNDS.msg());
                                unsafe { manage.try_unload(pos, self.cells.read(index))? }
                                Ok(())
                            })?;
                    }
                };
            }
            // Y+ region
            unload_bounds!(old_bounds.y_max() > new_bounds.y_max() =>
                xmin = old_bounds.x_min();
                ymin = new_bounds.y_max();
                zmin = old_bounds.z_min();
                xmax = old_bounds.x_max();
                ymax = old_bounds.y_max();
                zmax = old_bounds.z_max();
            );
            // Y- region
            unload_bounds!(old_bounds.y_min() < new_bounds.y_min() =>
                xmin = old_bounds.x_min();
                ymin = old_bounds.y_min();
                zmin = old_bounds.z_min();
                xmax = old_bounds.x_max();
                ymax = new_bounds.y_min();
                zmax = old_bounds.z_max();
            );
            // Z+ region (row)
            unload_bounds!(old_bounds.z_max() > new_bounds.z_max() =>
                xmin = old_bounds.x_min();
                ymin = new_bounds.y_min().max(old_bounds.y_min());
                zmin = new_bounds.z_max();
                xmax = old_bounds.x_max();
                ymax = new_bounds.y_max().min(old_bounds.y_max());
                zmax = old_bounds.z_max();
            );
            // Z- region (row)
            unload_bounds!(old_bounds.z_min() < new_bounds.z_min() =>
                xmin = old_bounds.x_min();
                ymin = new_bounds.y_min().max(old_bounds.y_min());
                zmin = old_bounds.z_min();
                xmax = old_bounds.x_max();
                ymax = new_bounds.y_max().min(old_bounds.y_max());
                zmax = new_bounds.z_min();
            );
            // X+ region (cube)
            unload_bounds!(old_bounds.x_max() > new_bounds.x_max() =>
                xmin = new_bounds.x_max();
                ymin = new_bounds.y_min().max(old_bounds.y_min());
                zmin = new_bounds.z_min().max(old_bounds.z_min());
                xmax = old_bounds.x_max();
                ymax = new_bounds.y_max().min(old_bounds.y_max());
                zmax = new_bounds.z_max().min(old_bounds.z_max());
            );
            // X- region (cube)
            unload_bounds!(old_bounds.x_min() < new_bounds.x_min() =>
                xmin = old_bounds.x_min();
                ymin = new_bounds.y_min().max(old_bounds.y_min());
                zmin = new_bounds.z_min().max(old_bounds.z_min());
                xmax = new_bounds.x_min();
                ymax = new_bounds.y_max().min(old_bounds.y_max());
                zmax = new_bounds.z_max().min(old_bounds.z_max());
            );
            let size = (width, height, depth);
            let new_grid = FixedArray::try_new_3d(size, new_position, |pos| {
                if old_bounds.contains(pos) {
                    let index = self.offset_index(pos).expect(OUT_OF_BOUNDS.msg());
                    unsafe { Ok(self.cells.read(index)) }
                } else {
                    manage.try_load(pos)
                }
            })?;
            self.size = size;
            self.grid_offset = new_position;
            unsafe {
                self.cells.forget_dealloc();
            }
            self.cells = new_grid;
            self.wrap_offset = (0, 0, 0);
        } else {
            // !old_bounds.intersects(new_bounds)
            old_bounds.iter().try_for_each(|pos| {
                let index = self.offset_index(pos).expect(OUT_OF_BOUNDS.msg());
                unsafe {
                    manage.try_unload(pos, self.cells.read(index))?;
                }
                Ok(())
            })?;
            let size = (width, height, depth);
            let new_grid = FixedArray::try_new_3d(size, new_position, |pos| manage.try_load(pos))?;
            self.size = size;
            self.grid_offset = new_position;
            unsafe {
                self.cells.forget_dealloc();
            }
            self.cells = new_grid;
            self.wrap_offset = (0, 0, 0);
        }
        Ok(())
    }

    /// Translate the grid by offset amount using a reload function.
    ///
    /// The reload function takes the old position, the new position, and
    /// a mutable reference to the cell where the initial value of the cell
    /// when called is the value at `old_position`. You want to change the
    /// cell to the correct value for a cell at `new_position`.
    ///
    /// # Example
    /// ```rust, no_run
    /// grid.translate((2, 3, 4), |old_position, new_position, cell_mut| {
    ///     *cell_mut = new_position;
    /// })
    /// ```
    pub fn translate<F>(&mut self, offset: (i32, i32, i32), reload: F)
    where
        F: FnMut((i32, i32, i32), (i32, i32, i32), &mut T),
    {
        let (curx, cury, curz) = offset;
        let (ox, oy, oz) = offset;
        let new_x = X_MAX_EXCEEDS_MAXIMUM.expect(curx.checked_add(ox));
        let new_y = Y_MAX_EXCEEDS_MAXIMUM.expect(cury.checked_add(oy));
        let new_z = Z_MAX_EXCEEDS_MAXIMUM.expect(curz.checked_add(oz));
        self.reposition((new_x, new_y, new_z), reload);
    }

    /// Try to translate the grid by offset amount using a fallible reload function.
    ///
    /// # Example
    /// ```rust, no_run
    /// grid.try_translate((2, 3, 4), |old_position, new_position, cell_mut| {
    ///     *cell_mut = new_position;
    ///     Ok(())
    /// })
    /// ```
    pub fn try_translate<E, F>(&mut self, offset: (i32, i32, i32), reload: F) -> Result<(), E>
    where
        F: FnMut((i32, i32, i32), (i32, i32, i32), &mut T) -> Result<(), E>,
    {
        let (curx, cury, curz) = offset;
        let (ox, oy, oz) = offset;
        let new_x = X_MAX_EXCEEDS_MAXIMUM.expect(curx.checked_add(ox));
        let new_y = Y_MAX_EXCEEDS_MAXIMUM.expect(cury.checked_add(oy));
        let new_z = Z_MAX_EXCEEDS_MAXIMUM.expect(curz.checked_add(oz));
        self.try_reposition((new_x, new_y, new_z), reload)
    }

    /// Reposition the offset of the grid and reload the slots that are changed.
    ///
    /// The reload function takes the old position, the new position, and
    /// a mutable reference to the cell where the initial value of the cell
    /// when called is the value at `old_position`. You want to change the
    /// cell to the correct value for a cell at `new_position`.
    ///
    /// # Example
    /// ```rust, no_run
    /// grid.reposition((2, 3, 4), |old_position, new_position, cell_mut| {
    ///     *cell_mut = new_position;
    /// })
    /// ```
    pub fn reposition<F>(&mut self, position: (i32, i32, i32), reload: F)
    where
        F: FnMut((i32, i32, i32), (i32, i32, i32), &mut T),
    {
        let mut reload = reload;
        if self.grid_offset == position {
            return;
        }
        let (old_x, old_y, old_z) = self.grid_offset;
        let (new_x, new_y, new_z) = position;
        let offset = (
            new_x as i64 - old_x as i64,
            new_y as i64 - old_y as i64,
            new_z as i64 - old_z as i64,
        );
        let width = self.size.0 as i64;
        let height = self.size.1 as i64;
        let depth = self.size.2 as i64;
        let (offset_x, offset_y, offset_z) = offset;
        let old_bounds = self.bounds();
        let right = X_MAX_EXCEEDS_MAXIMUM.expect(checked_add_u32_to_i32(new_x, self.size.0));
        let bottom = Y_MAX_EXCEEDS_MAXIMUM.expect(checked_add_u32_to_i32(new_y, self.size.1));
        let back = Z_MAX_EXCEEDS_MAXIMUM.expect(checked_add_u32_to_i32(new_z, self.size.2));
        let new_bounds = Bounds3D::new((new_x, new_y, new_z), (right, bottom, back));
        // A cool trick to test whether the translation moves out of bounds.
        if offset_x.abs() < width && offset_y.abs() < height && offset_z.abs() < depth {
            // translation in bounds, the hard part.
            // My plan is to subdivide the reload region into (upto) three parts.
            // It's very difficult to visualize this stuff, so I used Minecraft to create a rudimentary visualization.
            // https://i.imgur.com/FdlQTyS.png
            // There are three pieces. The half piece, the eighth piece, and the quarter piece. (not actual sizes, just representative)
            // not all three of these regions will be present. There will be cases where only one or two are present.
            // I'll make the side piece on the y/z axes.
            // After doing some thinking, I decided I should determine the best place to put the half_region.
            // Check if it can fit at x_min or x_max
            // Otherwise check if it can fit in z_min or z_max
            // Finally check if it can fit in y_min or y_max
            let (half_region, quarter_region, eighth_region) = if new_bounds.x_min()
                < old_bounds.x_min()
            {
                // -X
                let half_region = {
                    let x_min = new_bounds.x_min();
                    let y_min = new_bounds.y_min();
                    let z_min = new_bounds.z_min();
                    let x_max = old_bounds.x_min();
                    let y_max = new_bounds.y_max();
                    let z_max = new_bounds.z_max();
                    Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                };
                let (quarter_region, eighth_region) = if new_bounds.z_min() < old_bounds.z_min() {
                    // -X -Z
                    let quarter_region = {
                        let x_min = old_bounds.x_min();
                        let y_min = new_bounds.y_min();
                        let z_min = new_bounds.z_min();
                        let x_max = new_bounds.x_max();
                        let y_max = new_bounds.y_max();
                        let z_max = old_bounds.z_min();
                        Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                    };
                    let eighth_region = if new_bounds.y_min() < old_bounds.y_min() {
                        // eighth: -X -Y -Z
                        let x_min = old_bounds.x_min();
                        let y_min = new_bounds.y_min();
                        let z_min = old_bounds.z_min();
                        let x_max = new_bounds.x_max();
                        let y_max = old_bounds.y_min();
                        let z_max = new_bounds.z_max();
                        Some(Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max)))
                    } else if new_bounds.y_max() > old_bounds.y_max() {
                        // eighth: -X +Y -Z
                        let x_min = old_bounds.x_min();
                        let y_min = old_bounds.y_max();
                        let z_min = old_bounds.z_min();
                        let x_max = new_bounds.x_max();
                        let y_max = new_bounds.y_max();
                        let z_max = new_bounds.z_max();
                        Some(Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max)))
                    } else {
                        // eighth: -X =Y -Z
                        None
                    };
                    (Some(quarter_region), eighth_region)
                } else if new_bounds.z_max() > old_bounds.z_max() {
                    // -X +Z
                    let quarter_region = {
                        let x_min = old_bounds.x_min();
                        let y_min = new_bounds.y_min();
                        let z_min = old_bounds.z_max();
                        let x_max = new_bounds.x_max();
                        let y_max = new_bounds.y_max();
                        let z_max = new_bounds.z_max();
                        Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                    };
                    let eighth_region = if new_bounds.y_min() < old_bounds.y_min() {
                        // eighth: -X -Y +Z
                        let x_min = old_bounds.x_min();
                        let y_min = new_bounds.y_min();
                        let z_min = new_bounds.z_min();
                        let x_max = new_bounds.x_max();
                        let y_max = old_bounds.y_min();
                        let z_max = old_bounds.z_max();
                        Some(Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max)))
                    } else if new_bounds.y_max() > old_bounds.y_max() {
                        // eighth: -X +Y +Z
                        let x_min = old_bounds.x_min();
                        let y_min = old_bounds.y_max();
                        let z_min = new_bounds.z_min();
                        let x_max = new_bounds.x_max();
                        let y_max = new_bounds.y_max();
                        let z_max = old_bounds.z_max();
                        Some(Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max)))
                    } else {
                        // eighth: -X =Y +Z
                        None
                    };
                    (Some(quarter_region), eighth_region)
                } else {
                    // z is same, x is less
                    // -X =Z
                    let quarter_region = if new_bounds.y_min() < old_bounds.y_min() {
                        // quarter: -X -Y =Z
                        let x_min = old_bounds.x_min();
                        let y_min = new_bounds.y_min();
                        let z_min = new_bounds.z_min();
                        let x_max = new_bounds.x_max();
                        let y_max = old_bounds.y_min();
                        let z_max = new_bounds.z_max();
                        Some(Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max)))
                    } else if new_bounds.y_max() > old_bounds.y_max() {
                        // quarter: -X +Y =Z
                        let x_min = old_bounds.x_min();
                        let y_min = old_bounds.y_max();
                        let z_min = new_bounds.z_min();
                        let x_max = new_bounds.x_max();
                        let y_max = new_bounds.y_max();
                        let z_max = new_bounds.z_max();
                        Some(Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max)))
                    } else {
                        None
                    };
                    (quarter_region, None)
                };
                (half_region, quarter_region, eighth_region)
            } else if new_bounds.x_max() > old_bounds.x_max() {
                // (half, quarter, eighth) = if
                // +X
                let half_region = {
                    let x_min = old_bounds.x_max();
                    let y_min = new_bounds.y_min();
                    let z_min = new_bounds.z_min();
                    let x_max = new_bounds.x_max();
                    let y_max = new_bounds.y_max();
                    let z_max = new_bounds.z_max();
                    Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                };
                let (quarter_region, eighth_region) = if new_bounds.z_min() < old_bounds.z_min() {
                    // +X -Z
                    let quarter_region = {
                        let x_min = new_bounds.x_min();
                        let y_min = new_bounds.y_min();
                        let z_min = new_bounds.z_min();
                        let x_max = old_bounds.x_max();
                        let y_max = new_bounds.y_max();
                        let z_max = old_bounds.z_min();
                        Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                    };
                    let eighth_region = if new_bounds.y_min() < old_bounds.y_min() {
                        // eighth: +X -Y -Z
                        let x_min = new_bounds.x_min();
                        let y_min = new_bounds.y_min();
                        let z_min = old_bounds.z_min();
                        let x_max = old_bounds.x_max();
                        let y_max = old_bounds.y_min();
                        let z_max = new_bounds.z_max();
                        Some(Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max)))
                    } else if new_bounds.y_max() > old_bounds.y_max() {
                        // eighth: +X +Y -Z
                        let x_min = new_bounds.x_min();
                        let y_min = old_bounds.y_max();
                        let z_min = old_bounds.z_min();
                        let x_max = old_bounds.x_max();
                        let y_max = new_bounds.y_max();
                        let z_max = new_bounds.z_max();
                        Some(Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max)))
                    } else {
                        None
                    };
                    (Some(quarter_region), eighth_region)
                } else if new_bounds.z_max() > old_bounds.z_max() {
                    // +X +Z
                    let quarter_region = {
                        let x_min = new_bounds.x_min();
                        let y_min = new_bounds.y_min();
                        let z_min = old_bounds.z_max();
                        let x_max = old_bounds.x_max();
                        let y_max = new_bounds.y_max();
                        let z_max = new_bounds.z_max();
                        Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                    };
                    let eighth_region = if new_bounds.y_min() < old_bounds.y_min() {
                        // eighth: +X -Y +Z
                        let x_min = new_bounds.x_min();
                        let y_min = new_bounds.y_min();
                        let z_min = new_bounds.z_min();
                        let x_max = old_bounds.x_max();
                        let y_max = old_bounds.y_min();
                        let z_max = old_bounds.z_max();
                        Some(Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max)))
                    } else if new_bounds.y_max() > old_bounds.y_max() {
                        // eighth: +X +Y +Z
                        let x_min = new_bounds.x_min();
                        let y_min = old_bounds.y_max();
                        let z_min = new_bounds.z_min();
                        let x_max = old_bounds.x_max();
                        let y_max = new_bounds.y_max();
                        let z_max = old_bounds.z_max();
                        Some(Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max)))
                    } else {
                        None
                    };
                    (Some(quarter_region), eighth_region)
                } else {
                    // z is equal, x is greater
                    // +X =Z
                    let quarter_region = if new_bounds.y_min() < old_bounds.y_min() {
                        // quarter: +X -Y =Z
                        let x_min = new_bounds.x_min();
                        let y_min = new_bounds.y_min();
                        let z_min = new_bounds.z_min();
                        let x_max = old_bounds.x_max();
                        let y_max = old_bounds.y_min();
                        let z_max = new_bounds.z_max();
                        Some(Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max)))
                    } else if new_bounds.y_max() > old_bounds.y_max() {
                        // quarter: +X +Y =Z
                        let x_min = new_bounds.x_min();
                        let y_min = old_bounds.y_max();
                        let z_min = new_bounds.z_min();
                        let x_max = old_bounds.x_max();
                        let y_max = new_bounds.y_max();
                        let z_max = new_bounds.z_max();
                        Some(Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max)))
                    } else {
                        // quarter: +X =Y =Z
                        None
                    };
                    (quarter_region, None)
                };
                (half_region, quarter_region, eighth_region)
            } else {
                // x is equal
                // =X
                // (half, quarter, eighth) = if
                let (half_region, quarter_region) = if new_bounds.z_min() < old_bounds.z_min() {
                    // =X -Z
                    if new_bounds.y_min() < old_bounds.y_min() {
                        // =X -Y -Z
                        let half_region = {
                            let x_min = new_bounds.x_min();
                            let y_min = new_bounds.y_min();
                            let z_min = new_bounds.z_min();
                            let x_max = new_bounds.x_max();
                            let y_max = new_bounds.y_max();
                            let z_max = old_bounds.z_min();
                            Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                        };
                        let quarter_region = {
                            let x_min = new_bounds.x_min();
                            let y_min = new_bounds.y_min();
                            let z_min = old_bounds.z_min();
                            let x_max = new_bounds.x_max();
                            let y_max = old_bounds.y_min();
                            let z_max = new_bounds.z_max();
                            Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                        };
                        (half_region, Some(quarter_region))
                    } else if new_bounds.y_max() > old_bounds.y_max() {
                        // =X +Y -Z
                        let half_region = {
                            let x_min = new_bounds.x_min();
                            let y_min = new_bounds.y_min();
                            let z_min = new_bounds.z_min();
                            let x_max = new_bounds.x_max();
                            let y_max = new_bounds.y_max();
                            let z_max = old_bounds.z_min();
                            Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                        };
                        let quarter_region = {
                            let x_min = new_bounds.x_min();
                            let y_min = old_bounds.y_max();
                            let z_min = old_bounds.z_min();
                            let x_max = new_bounds.x_max();
                            let y_max = new_bounds.y_max();
                            let z_max = new_bounds.z_max();
                            Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                        };
                        (half_region, Some(quarter_region))
                    } else {
                        // x is equal, y is equal, z is less
                        // =X =Y -Z
                        // create only half_region
                        let x_min = new_bounds.x_min();
                        let y_min = new_bounds.y_min();
                        let z_min = new_bounds.z_min();
                        let x_max = new_bounds.x_max();
                        let y_max = new_bounds.y_max();
                        let z_max = old_bounds.z_min();
                        let half_region =
                            Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max));
                        (half_region, None)
                    }
                } else if new_bounds.z_max() > old_bounds.z_max() {
                    // (half, quarter) = if
                    // =X
                    if new_bounds.y_min() < old_bounds.y_min() {
                        // x is equal, z is greater
                        // =X -Y +Z
                        // (half, Option<quarter>) = if; return (half, quarter)
                        let half_region = {
                            let x_min = new_bounds.x_min();
                            let y_min = new_bounds.y_min();
                            let z_min = old_bounds.z_max();
                            let x_max = new_bounds.x_max();
                            let y_max = new_bounds.y_max();
                            let z_max = new_bounds.z_max();
                            Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                        };
                        let quarter_region = {
                            let x_min = new_bounds.x_min();
                            let y_min = new_bounds.y_min();
                            let z_min = new_bounds.z_min();
                            let x_max = new_bounds.x_max();
                            let y_max = old_bounds.y_min();
                            let z_max = old_bounds.z_max();
                            Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                        };
                        (half_region, Some(quarter_region))
                    } else if new_bounds.y_max() > old_bounds.y_max() {
                        // x is equal, z is greater
                        // =X +Y +Z
                        // (half, Option<quarter>) = if; return (half, quarter)
                        let half_region = {
                            let x_min = new_bounds.x_min();
                            let y_min = new_bounds.y_min();
                            let z_min = old_bounds.z_max();
                            let x_max = new_bounds.x_max();
                            let y_max = new_bounds.y_max();
                            let z_max = new_bounds.z_max();
                            Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                        };
                        let quarter_region = {
                            let x_min = new_bounds.x_min();
                            let y_min = old_bounds.y_max();
                            let z_min = new_bounds.z_min();
                            let x_max = new_bounds.x_max();
                            let y_max = new_bounds.y_max();
                            let z_max = old_bounds.z_max();
                            Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                        };
                        (half_region, Some(quarter_region))
                    } else {
                        // x is equal, y is equal, z is greater
                        // =X =Y +Z
                        // (half, Option<quarter>) = if; return (half, quarter)
                        // no quarter_region
                        let x_min = new_bounds.x_min();
                        let y_min = new_bounds.y_min();
                        let z_min = old_bounds.z_max();
                        let x_max = new_bounds.x_max();
                        let y_max = new_bounds.y_max();
                        let z_max = new_bounds.z_max();
                        let half_region =
                            Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max));
                        (half_region, None)
                    }
                } else {
                    // x is equal, z is equal
                    // =X =Z
                    // (half, Option<quarter>) = if; return (half, quarter)
                    let half_region = if new_bounds.y_min() < old_bounds.y_min() {
                        // =X -Y =Z
                        let x_min = new_bounds.x_min();
                        let y_min = new_bounds.y_min();
                        let z_min = new_bounds.z_min();
                        let x_max = new_bounds.x_max();
                        let y_max = old_bounds.y_min();
                        let z_max = new_bounds.z_max();
                        Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                    } else if new_bounds.y_max() > old_bounds.y_max() {
                        // =X +Y =Z
                        let x_min = new_bounds.x_min();
                        let y_min = old_bounds.y_max();
                        let z_min = new_bounds.z_min();
                        let x_max = new_bounds.x_max();
                        let y_max = new_bounds.y_max();
                        let z_max = new_bounds.z_max();
                        Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                    } else {
                        // =X =Y =Z: unreachable
                        // It has already been determined that the bounds
                        // are offset, therefore this branch is unreachable.
                        unreachable!()
                    };
                    (half_region, None)
                };
                (half_region, quarter_region, None)
            };
            // Calculate new wrap_offset
            let (wrap_x, wrap_y, wrap_z) = (
                self.wrap_offset.0 as i64,
                self.wrap_offset.1 as i64,
                self.wrap_offset.2 as i64,
            );
            let (wrapped_offset_x, wrapped_offset_y, wrapped_offset_z) = (
                offset_x.rem_euclid(width),
                offset_y.rem_euclid(height),
                offset_z.rem_euclid(depth),
            );
            let new_wrap_x = (wrap_x + wrapped_offset_x).rem_euclid(width);
            let new_wrap_y = (wrap_y + wrapped_offset_y).rem_euclid(height);
            let new_wrap_z = (wrap_z + wrapped_offset_z).rem_euclid(depth);
            struct OffsetFix {
                /// the old grid offset that we can use to
                /// create a relational offset
                offset: (i64, i64, i64),
                size: (i64, i64, i64),
            }
            impl OffsetFix {
                fn wrap(&self, pos: (i32, i32, i32)) -> (i32, i32, i32) {
                    let x = (pos.0 as i64 - self.offset.0).rem_euclid(self.size.0) + self.offset.0;
                    let y = (pos.1 as i64 - self.offset.1).rem_euclid(self.size.1) + self.offset.1;
                    let z = (pos.2 as i64 - self.offset.2).rem_euclid(self.size.2) + self.offset.2;
                    (x as i32, y as i32, z as i32)
                }
            }
            let fix = OffsetFix {
                offset: self.grid_offset.convert(),
                size: (width, height, depth),
            };
            self.wrap_offset = (new_wrap_x as u32, new_wrap_y as u32, new_wrap_z as u32);
            self.grid_offset = (new_x, new_y, new_z);
            // Now that we have the regions, we can iterate over them to reload cells.
            // iterate regions and reload cells
            half_region.iter().for_each(|pos| {
                let old_pos = fix.wrap(pos);
                let index = self.offset_index(pos).expect(OUT_OF_BOUNDS.msg());
                reload(old_pos, pos, &mut self.cells[index]);
            });
            if let Some(quarter) = quarter_region {
                quarter.iter().for_each(|pos| {
                    let old_pos = fix.wrap(pos);
                    let index = self.offset_index(pos).expect(OUT_OF_BOUNDS.msg());
                    reload(old_pos, pos, &mut self.cells[index]);
                });
            }
            if let Some(eighth) = eighth_region {
                eighth.iter().for_each(|pos| {
                    let old_pos = fix.wrap(pos);
                    let index = self.offset_index(pos).expect(OUT_OF_BOUNDS.msg());
                    reload(old_pos, pos, &mut self.cells[index]);
                });
            }
        } else {
            // translation out of bounds, reload everything
            self.grid_offset = (new_x, new_y, new_z);
            let new_x = new_x as i64;
            let new_y = new_y as i64;
            let new_z = new_z as i64;
            let old_x = old_x as i64;
            let old_y = old_y as i64;
            let old_z = old_z as i64;
            for (yi, y) in (new_y..new_y + height).enumerate() {
                for (zi, z) in (new_z..new_z + depth).enumerate() {
                    for (xi, x) in (new_x..new_x + width).enumerate() {
                        let prior_x = (old_x + xi as i64) as i32;
                        let prior_y = (old_y + yi as i64) as i32;
                        let prior_z = (old_z + zi as i64) as i32;
                        let offset = (x as i32, y as i32, z as i32);
                        let index = self.offset_index(offset).expect(OUT_OF_BOUNDS.msg());
                        reload((prior_x, prior_y, prior_z), offset, &mut self.cells[index]);
                    }
                }
            }
        }
    }

    /// Try to reposition the offset of the grid and reload the slots that are changed.
    ///
    /// The reload function takes the old position, the new position, and
    /// a mutable reference to the cell where the initial value of the cell
    /// when called is the value at `old_position`. You want to change the
    /// cell to the correct value for a cell at `new_position`.
    ///
    /// # Example
    /// ```rust, no_run
    /// grid.try_reposition((2, 3, 4), |old_position, new_position, cell_mut| {
    ///     *cell_mut = new_position;
    ///     Ok(())
    /// })
    /// ```
    pub fn try_reposition<E, F>(&mut self, position: (i32, i32, i32), reload: F) -> Result<(), E>
    where
        F: FnMut((i32, i32, i32), (i32, i32, i32), &mut T) -> Result<(), E>,
    {
        let mut reload = reload;
        if self.grid_offset == position {
            return Ok(());
        }
        let (old_x, old_y, old_z) = self.grid_offset;
        let (new_x, new_y, new_z) = position;
        let offset = (
            new_x as i64 - old_x as i64,
            new_y as i64 - old_y as i64,
            new_z as i64 - old_z as i64,
        );
        let width = self.size.0 as i64;
        let height = self.size.1 as i64;
        let depth = self.size.2 as i64;
        let (offset_x, offset_y, offset_z) = offset;
        let old_bounds = self.bounds();
        let right = X_MAX_EXCEEDS_MAXIMUM.expect(checked_add_u32_to_i32(new_x, self.size.0));
        let bottom = Y_MAX_EXCEEDS_MAXIMUM.expect(checked_add_u32_to_i32(new_y, self.size.1));
        let back = Z_MAX_EXCEEDS_MAXIMUM.expect(checked_add_u32_to_i32(new_z, self.size.2));
        let new_bounds = Bounds3D::new((new_x, new_y, new_z), (right, bottom, back));
        // A cool trick to test whether the translation moves out of bounds.
        if offset_x.abs() < width && offset_y.abs() < height && offset_z.abs() < depth {
            // translation in bounds, the hard part.
            // My plan is to subdivide the reload region into (upto) three parts.
            // It's very difficult to visualize this stuff, so I used Minecraft to create a rudimentary visualization.
            // https://i.imgur.com/FdlQTyS.png
            // There are three pieces. The half piece, the eighth piece, and the quarter piece. (not actual sizes, just representative)
            // not all three of these regions will be present. There will be cases where only one or two are present.
            // I'll make the side piece on the y/z axes.
            // After doing some thinking, I decided I should determine the best place to put the half_region.
            // Check if it can fit at x_min or x_max
            // Otherwise check if it can fit in z_min or z_max
            // Finally check if it can fit in y_min or y_max
            let (half_region, quarter_region, eighth_region) = if new_bounds.x_min()
                < old_bounds.x_min()
            {
                // -X
                let half_region = {
                    let x_min = new_bounds.x_min();
                    let y_min = new_bounds.y_min();
                    let z_min = new_bounds.z_min();
                    let x_max = old_bounds.x_min();
                    let y_max = new_bounds.y_max();
                    let z_max = new_bounds.z_max();
                    Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                };
                let (quarter_region, eighth_region) = if new_bounds.z_min() < old_bounds.z_min() {
                    // -X -Z
                    let quarter_region = {
                        let x_min = old_bounds.x_min();
                        let y_min = new_bounds.y_min();
                        let z_min = new_bounds.z_min();
                        let x_max = new_bounds.x_max();
                        let y_max = new_bounds.y_max();
                        let z_max = old_bounds.z_min();
                        Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                    };
                    let eighth_region = if new_bounds.y_min() < old_bounds.y_min() {
                        // eighth: -X -Y -Z
                        let x_min = old_bounds.x_min();
                        let y_min = new_bounds.y_min();
                        let z_min = old_bounds.z_min();
                        let x_max = new_bounds.x_max();
                        let y_max = old_bounds.y_min();
                        let z_max = new_bounds.z_max();
                        Some(Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max)))
                    } else if new_bounds.y_max() > old_bounds.y_max() {
                        // eighth: -X +Y -Z
                        let x_min = old_bounds.x_min();
                        let y_min = old_bounds.y_max();
                        let z_min = old_bounds.z_min();
                        let x_max = new_bounds.x_max();
                        let y_max = new_bounds.y_max();
                        let z_max = new_bounds.z_max();
                        Some(Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max)))
                    } else {
                        // eighth: -X =Y -Z
                        None
                    };
                    (Some(quarter_region), eighth_region)
                } else if new_bounds.z_max() > old_bounds.z_max() {
                    // -X +Z
                    let quarter_region = {
                        let x_min = old_bounds.x_min();
                        let y_min = new_bounds.y_min();
                        let z_min = old_bounds.z_max();
                        let x_max = new_bounds.x_max();
                        let y_max = new_bounds.y_max();
                        let z_max = new_bounds.z_max();
                        Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                    };
                    let eighth_region = if new_bounds.y_min() < old_bounds.y_min() {
                        // eighth: -X -Y +Z
                        let x_min = old_bounds.x_min();
                        let y_min = new_bounds.y_min();
                        let z_min = new_bounds.z_min();
                        let x_max = new_bounds.x_max();
                        let y_max = old_bounds.y_min();
                        let z_max = old_bounds.z_max();
                        Some(Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max)))
                    } else if new_bounds.y_max() > old_bounds.y_max() {
                        // eighth: -X +Y +Z
                        let x_min = old_bounds.x_min();
                        let y_min = old_bounds.y_max();
                        let z_min = new_bounds.z_min();
                        let x_max = new_bounds.x_max();
                        let y_max = new_bounds.y_max();
                        let z_max = old_bounds.z_max();
                        Some(Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max)))
                    } else {
                        // eighth: -X =Y +Z
                        None
                    };
                    (Some(quarter_region), eighth_region)
                } else {
                    // z is same, x is less
                    // -X =Z
                    let quarter_region = if new_bounds.y_min() < old_bounds.y_min() {
                        // quarter: -X -Y =Z
                        let x_min = old_bounds.x_min();
                        let y_min = new_bounds.y_min();
                        let z_min = new_bounds.z_min();
                        let x_max = new_bounds.x_max();
                        let y_max = old_bounds.y_min();
                        let z_max = new_bounds.z_max();
                        Some(Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max)))
                    } else if new_bounds.y_max() > old_bounds.y_max() {
                        // quarter: -X +Y =Z
                        let x_min = old_bounds.x_min();
                        let y_min = old_bounds.y_max();
                        let z_min = new_bounds.z_min();
                        let x_max = new_bounds.x_max();
                        let y_max = new_bounds.y_max();
                        let z_max = new_bounds.z_max();
                        Some(Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max)))
                    } else {
                        None
                    };
                    (quarter_region, None)
                };
                (half_region, quarter_region, eighth_region)
            } else if new_bounds.x_max() > old_bounds.x_max() {
                // (half, quarter, eighth) = if
                // +X
                let half_region = {
                    let x_min = old_bounds.x_max();
                    let y_min = new_bounds.y_min();
                    let z_min = new_bounds.z_min();
                    let x_max = new_bounds.x_max();
                    let y_max = new_bounds.y_max();
                    let z_max = new_bounds.z_max();
                    Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                };
                let (quarter_region, eighth_region) = if new_bounds.z_min() < old_bounds.z_min() {
                    // +X -Z
                    let quarter_region = {
                        let x_min = new_bounds.x_min();
                        let y_min = new_bounds.y_min();
                        let z_min = new_bounds.z_min();
                        let x_max = old_bounds.x_max();
                        let y_max = new_bounds.y_max();
                        let z_max = old_bounds.z_min();
                        Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                    };
                    let eighth_region = if new_bounds.y_min() < old_bounds.y_min() {
                        // eighth: +X -Y -Z
                        let x_min = new_bounds.x_min();
                        let y_min = new_bounds.y_min();
                        let z_min = old_bounds.z_min();
                        let x_max = old_bounds.x_max();
                        let y_max = old_bounds.y_min();
                        let z_max = new_bounds.z_max();
                        Some(Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max)))
                    } else if new_bounds.y_max() > old_bounds.y_max() {
                        // eighth: +X +Y -Z
                        let x_min = new_bounds.x_min();
                        let y_min = old_bounds.y_max();
                        let z_min = old_bounds.z_min();
                        let x_max = old_bounds.x_max();
                        let y_max = new_bounds.y_max();
                        let z_max = new_bounds.z_max();
                        Some(Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max)))
                    } else {
                        None
                    };
                    (Some(quarter_region), eighth_region)
                } else if new_bounds.z_max() > old_bounds.z_max() {
                    // +X +Z
                    let quarter_region = {
                        let x_min = new_bounds.x_min();
                        let y_min = new_bounds.y_min();
                        let z_min = old_bounds.z_max();
                        let x_max = old_bounds.x_max();
                        let y_max = new_bounds.y_max();
                        let z_max = new_bounds.z_max();
                        Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                    };
                    let eighth_region = if new_bounds.y_min() < old_bounds.y_min() {
                        // eighth: +X -Y +Z
                        let x_min = new_bounds.x_min();
                        let y_min = new_bounds.y_min();
                        let z_min = new_bounds.z_min();
                        let x_max = old_bounds.x_max();
                        let y_max = old_bounds.y_min();
                        let z_max = old_bounds.z_max();
                        Some(Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max)))
                    } else if new_bounds.y_max() > old_bounds.y_max() {
                        // eighth: +X +Y +Z
                        let x_min = new_bounds.x_min();
                        let y_min = old_bounds.y_max();
                        let z_min = new_bounds.z_min();
                        let x_max = old_bounds.x_max();
                        let y_max = new_bounds.y_max();
                        let z_max = old_bounds.z_max();
                        Some(Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max)))
                    } else {
                        None
                    };
                    (Some(quarter_region), eighth_region)
                } else {
                    // z is equal, x is greater
                    // +X =Z
                    let quarter_region = if new_bounds.y_min() < old_bounds.y_min() {
                        // quarter: +X -Y =Z
                        let x_min = new_bounds.x_min();
                        let y_min = new_bounds.y_min();
                        let z_min = new_bounds.z_min();
                        let x_max = old_bounds.x_max();
                        let y_max = old_bounds.y_min();
                        let z_max = new_bounds.z_max();
                        Some(Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max)))
                    } else if new_bounds.y_max() > old_bounds.y_max() {
                        // quarter: +X +Y =Z
                        let x_min = new_bounds.x_min();
                        let y_min = old_bounds.y_max();
                        let z_min = new_bounds.z_min();
                        let x_max = old_bounds.x_max();
                        let y_max = new_bounds.y_max();
                        let z_max = new_bounds.z_max();
                        Some(Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max)))
                    } else {
                        // quarter: +X =Y =Z
                        None
                    };
                    (quarter_region, None)
                };
                (half_region, quarter_region, eighth_region)
            } else {
                // x is equal
                // =X
                // (half, quarter, eighth) = if
                let (half_region, quarter_region) = if new_bounds.z_min() < old_bounds.z_min() {
                    // =X -Z
                    if new_bounds.y_min() < old_bounds.y_min() {
                        // =X -Y -Z
                        let half_region = {
                            let x_min = new_bounds.x_min();
                            let y_min = new_bounds.y_min();
                            let z_min = new_bounds.z_min();
                            let x_max = new_bounds.x_max();
                            let y_max = new_bounds.y_max();
                            let z_max = old_bounds.z_min();
                            Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                        };
                        let quarter_region = {
                            let x_min = new_bounds.x_min();
                            let y_min = new_bounds.y_min();
                            let z_min = old_bounds.z_min();
                            let x_max = new_bounds.x_max();
                            let y_max = old_bounds.y_min();
                            let z_max = new_bounds.z_max();
                            Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                        };
                        (half_region, Some(quarter_region))
                    } else if new_bounds.y_max() > old_bounds.y_max() {
                        // =X +Y -Z
                        let half_region = {
                            let x_min = new_bounds.x_min();
                            let y_min = new_bounds.y_min();
                            let z_min = new_bounds.z_min();
                            let x_max = new_bounds.x_max();
                            let y_max = new_bounds.y_max();
                            let z_max = old_bounds.z_min();
                            Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                        };
                        let quarter_region = {
                            let x_min = new_bounds.x_min();
                            let y_min = old_bounds.y_max();
                            let z_min = old_bounds.z_min();
                            let x_max = new_bounds.x_max();
                            let y_max = new_bounds.y_max();
                            let z_max = new_bounds.z_max();
                            Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                        };
                        (half_region, Some(quarter_region))
                    } else {
                        // x is equal, y is equal, z is less
                        // =X =Y -Z
                        // create only half_region
                        let x_min = new_bounds.x_min();
                        let y_min = new_bounds.y_min();
                        let z_min = new_bounds.z_min();
                        let x_max = new_bounds.x_max();
                        let y_max = new_bounds.y_max();
                        let z_max = old_bounds.z_min();
                        let half_region =
                            Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max));
                        (half_region, None)
                    }
                } else if new_bounds.z_max() > old_bounds.z_max() {
                    // (half, quarter) = if
                    // =X
                    if new_bounds.y_min() < old_bounds.y_min() {
                        // x is equal, z is greater
                        // =X -Y +Z
                        // (half, Option<quarter>) = if; return (half, quarter)
                        let half_region = {
                            let x_min = new_bounds.x_min();
                            let y_min = new_bounds.y_min();
                            let z_min = old_bounds.z_max();
                            let x_max = new_bounds.x_max();
                            let y_max = new_bounds.y_max();
                            let z_max = new_bounds.z_max();
                            Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                        };
                        let quarter_region = {
                            let x_min = new_bounds.x_min();
                            let y_min = new_bounds.y_min();
                            let z_min = new_bounds.z_min();
                            let x_max = new_bounds.x_max();
                            let y_max = old_bounds.y_min();
                            let z_max = old_bounds.z_max();
                            Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                        };
                        (half_region, Some(quarter_region))
                    } else if new_bounds.y_max() > old_bounds.y_max() {
                        // x is equal, z is greater
                        // =X +Y +Z
                        // (half, Option<quarter>) = if; return (half, quarter)
                        let half_region = {
                            let x_min = new_bounds.x_min();
                            let y_min = new_bounds.y_min();
                            let z_min = old_bounds.z_max();
                            let x_max = new_bounds.x_max();
                            let y_max = new_bounds.y_max();
                            let z_max = new_bounds.z_max();
                            Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                        };
                        let quarter_region = {
                            let x_min = new_bounds.x_min();
                            let y_min = old_bounds.y_max();
                            let z_min = new_bounds.z_min();
                            let x_max = new_bounds.x_max();
                            let y_max = new_bounds.y_max();
                            let z_max = old_bounds.z_max();
                            Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                        };
                        (half_region, Some(quarter_region))
                    } else {
                        // x is equal, y is equal, z is greater
                        // =X =Y +Z
                        // (half, Option<quarter>) = if; return (half, quarter)
                        // no quarter_region
                        let x_min = new_bounds.x_min();
                        let y_min = new_bounds.y_min();
                        let z_min = old_bounds.z_max();
                        let x_max = new_bounds.x_max();
                        let y_max = new_bounds.y_max();
                        let z_max = new_bounds.z_max();
                        let half_region =
                            Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max));
                        (half_region, None)
                    }
                } else {
                    // x is equal, z is equal
                    // =X =Z
                    // (half, Option<quarter>) = if; return (half, quarter)
                    let half_region = if new_bounds.y_min() < old_bounds.y_min() {
                        // =X -Y =Z
                        let x_min = new_bounds.x_min();
                        let y_min = new_bounds.y_min();
                        let z_min = new_bounds.z_min();
                        let x_max = new_bounds.x_max();
                        let y_max = old_bounds.y_min();
                        let z_max = new_bounds.z_max();
                        Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                    } else if new_bounds.y_max() > old_bounds.y_max() {
                        // =X +Y =Z
                        let x_min = new_bounds.x_min();
                        let y_min = old_bounds.y_max();
                        let z_min = new_bounds.z_min();
                        let x_max = new_bounds.x_max();
                        let y_max = new_bounds.y_max();
                        let z_max = new_bounds.z_max();
                        Bounds3D::new((x_min, y_min, z_min), (x_max, y_max, z_max))
                    } else {
                        // =X =Y =Z: unreachable
                        // It has already been determined that the bounds
                        // are offset, therefore this branch is unreachable.
                        unreachable!()
                    };
                    (half_region, None)
                };
                (half_region, quarter_region, None)
            };
            // Calculate new wrap_offset
            let (wrap_x, wrap_y, wrap_z) = (
                self.wrap_offset.0 as i64,
                self.wrap_offset.1 as i64,
                self.wrap_offset.2 as i64,
            );
            let (wrapped_offset_x, wrapped_offset_y, wrapped_offset_z) = (
                offset_x.rem_euclid(width),
                offset_y.rem_euclid(height),
                offset_z.rem_euclid(depth),
            );
            let new_wrap_x = (wrap_x + wrapped_offset_x).rem_euclid(width);
            let new_wrap_y = (wrap_y + wrapped_offset_y).rem_euclid(height);
            let new_wrap_z = (wrap_z + wrapped_offset_z).rem_euclid(depth);
            struct OffsetFix {
                /// the old grid offset that we can use to
                /// create a relational offset
                offset: (i64, i64, i64),
                size: (i64, i64, i64),
            }
            impl OffsetFix {
                fn wrap(&self, pos: (i32, i32, i32)) -> (i32, i32, i32) {
                    let x = (pos.0 as i64 - self.offset.0).rem_euclid(self.size.0) + self.offset.0;
                    let y = (pos.1 as i64 - self.offset.1).rem_euclid(self.size.1) + self.offset.1;
                    let z = (pos.2 as i64 - self.offset.2).rem_euclid(self.size.2) + self.offset.2;
                    (x as i32, y as i32, z as i32)
                }
            }
            let fix = OffsetFix {
                offset: self.grid_offset.convert(),
                size: (width, height, depth),
            };
            self.wrap_offset = (new_wrap_x as u32, new_wrap_y as u32, new_wrap_z as u32);
            self.grid_offset = (new_x, new_y, new_z);
            // Now that we have the regions, we can iterate over them to reload cells.
            // iterate regions and reload cells
            half_region.iter().try_for_each(|pos| {
                let old_pos = fix.wrap(pos);
                let index = self.offset_index(pos).expect(OUT_OF_BOUNDS.msg());
                reload(old_pos, pos, &mut self.cells[index])?;
                Ok(())
            })?;
            if let Some(quarter) = quarter_region {
                quarter.iter().try_for_each(|pos| {
                    let old_pos = fix.wrap(pos);
                    let index = self.offset_index(pos).expect(OUT_OF_BOUNDS.msg());
                    reload(old_pos, pos, &mut self.cells[index])?;
                    Ok(())
                })?;
            }
            if let Some(eighth) = eighth_region {
                eighth.iter().try_for_each(|pos| {
                    let old_pos = fix.wrap(pos);
                    let index = self.offset_index(pos).expect(OUT_OF_BOUNDS.msg());
                    reload(old_pos, pos, &mut self.cells[index])?;
                    Ok(())
                })?;
            }
        } else {
            // translation out of bounds, reload everything
            self.grid_offset = (new_x, new_y, new_z);
            let new_x = new_x as i64;
            let new_y = new_y as i64;
            let new_z = new_z as i64;
            let old_x = old_x as i64;
            let old_y = old_y as i64;
            let old_z = old_z as i64;
            for (yi, y) in (new_y..new_y + height).enumerate() {
                for (zi, z) in (new_z..new_z + depth).enumerate() {
                    for (xi, x) in (new_x..new_x + width).enumerate() {
                        let prior_x = (old_x + xi as i64) as i32;
                        let prior_y = (old_y + yi as i64) as i32;
                        let prior_z = (old_z + zi as i64) as i32;
                        let offset = (x as i32, y as i32, z as i32);
                        let index = self.offset_index(offset).expect(OUT_OF_BOUNDS.msg());
                        reload((prior_x, prior_y, prior_z), offset, &mut self.cells[index])?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Get the offset relative to the grid's offset.
    pub fn relative_offset(&self, coord: (i32, i32, i32)) -> (i64, i64, i64) {
        let (x, y, z) = coord.convert::<(i64, i64, i64)>();
        let (ox, oy, oz) = self.grid_offset.convert::<(i64, i64, i64)>();
        (x - ox, y - oy, z - oz)
    }

    /// The grid has a wrapping offset, which dictates the lookup order of cells.
    /// This method allows to find the index of a particular offset in the grid.
    /// Offsets are relative to the world origin `(0, 0, 0)`, and must account for
    /// the grid offset.
    fn offset_index(&self, (x, y, z): (i32, i32, i32)) -> Option<usize> {
        let (x, y, z): (i64, i64, i64) = (x, y, z).convert();
        let (off_x, off_y, off_z): (i64, i64, i64) = self.grid_offset.convert();
        let width = self.size.0 as i64;
        let height = self.size.1 as i64;
        let depth = self.size.2 as i64;
        if x < off_x
            || y < off_y
            || z < off_z
            || x >= off_x + width
            || y >= off_y + height
            || z >= off_z + depth
        {
            return None;
        }
        // Adjust x, y, and z
        let adj_x = x - off_x;
        let adj_y = y - off_y;
        let adj_z = z - off_z;
        // Wrap x, y, and z
        let (wx, wy, wz) = (
            self.wrap_offset.0 as i64,
            self.wrap_offset.1 as i64,
            self.wrap_offset.2 as i64,
        );
        let wx = (adj_x + wx).rem_euclid(width);
        let wy = (adj_y + wy).rem_euclid(height);
        let wz = (adj_z + wz).rem_euclid(depth);
        let plane = self.size.0 * self.size.2;
        Some(wy as usize * plane as usize + wz as usize * self.size.0 as usize + wx as usize)
    }

    /// Replace item at `coord` using `replace` function that takes as
    /// input the old value and returns the new value. This will swap the
    /// value in-place.
    pub fn replace_with<F: FnOnce(T) -> T>(&mut self, coord: (i32, i32, i32), replace: F) {
        let index = self.offset_index(coord).expect(OUT_OF_BOUNDS.msg());
        self.cells.replace_with(index, replace);
    }

    /// Replace item at `coord` using [std::mem::replace] and then returns
    /// the old value.
    pub fn replace(&mut self, coord: (i32, i32, i32), value: T) -> T {
        let index = self.offset_index(coord).expect(OUT_OF_BOUNDS.msg());
        self.cells.replace(index, value)
    }

    /// Reads the value from the cell without moving it. This leaves the memory in the cell unchanged.
    pub unsafe fn read(&self, coord: (i32, i32, i32)) -> T {
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
    pub unsafe fn write(&mut self, coord: (i32, i32, i32), value: T) {
        let index = self.offset_index(coord).expect(OUT_OF_BOUNDS.msg());
        self.cells.write(index, value);
    }

    /// Get a reference to the cell's value if the coord is in bounds, otherwise return `None`.
    pub fn get(&self, coord: (i32, i32, i32)) -> Option<&T> {
        let index = self.offset_index(coord)?;
        Some(&self.cells[index])
    }

    /// Get a mutable reference to the cell's value if the coord is in bounds, otherwise return `None`.
    pub fn get_mut(&mut self, coord: (i32, i32, i32)) -> Option<&mut T> {
        let index = self.offset_index(coord)?;
        Some(&mut self.cells[index])
    }

    /// Set the cell's value, returning the old value in the process.
    pub fn set(&mut self, coord: (i32, i32, i32), value: T) -> Option<T> {
        let index = self.offset_index(coord)?;
        let dest = &mut self.cells[index];
        Some(std::mem::replace(dest, value))
    }

    /// Get a subsection of the grid.
    pub fn subgrid<'a>(&'a self, bounds: Bounds3D) -> Grid3D<&'a T> {
        let self_bounds = self.bounds();
        if bounds.x_min() < self_bounds.x_min()
            || bounds.y_min() < self_bounds.y_min()
            || bounds.z_min() < self_bounds.z_min()
            || bounds.x_max() > self_bounds.x_max()
            || bounds.y_max() > self_bounds.y_max()
            || bounds.z_max() > self_bounds.z_max()
        {
            OUT_OF_BOUNDS.panic();
        }
        unsafe {
            let ptr = self.cells.as_ptr();
            let grid = Grid3D::new(bounds.size(), bounds.min, |pos| {
                let index = self.offset_index(pos).unwrap();
                let cell_ptr = ptr.add(index);
                cell_ptr.as_ref().unwrap()
            });
            grid
        }
    }

    /// Get a mutable subsection of the grid.
    pub fn subgrid_mut<'a>(&'a mut self, bounds: Bounds3D) -> Grid3D<&'a mut T> {
        let self_bounds = self.bounds();
        if bounds.x_min() < self_bounds.x_min()
            || bounds.y_min() < self_bounds.y_min()
            || bounds.z_min() < self_bounds.z_min()
            || bounds.x_max() > self_bounds.x_max()
            || bounds.y_max() > self_bounds.y_max()
            || bounds.z_max() > self_bounds.z_max()
        {
            OUT_OF_BOUNDS.panic();
        }
        unsafe {
            let ptr = self.cells.as_ptr();
            let grid = Grid3D::new(bounds.size(), bounds.min, |pos| {
                let index = self.offset_index(pos).unwrap();
                let cell_ptr = ptr.add(index);
                cell_ptr.cast_mut().as_mut().unwrap()
            });
            grid
        }
    }

    /// Get the dimensions of the grid.
    pub fn size(&self) -> (u32, u32, u32) {
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

    /// The size along the Z axis.
    pub fn depth(&self) -> u32 {
        self.size.2
    }

    /// Get the offset of the grid.
    pub fn offset(&self) -> (i32, i32, i32) {
        self.grid_offset
    }

    /// Get the minimum bound on the `X` axis.
    pub fn x_min(&self) -> i32 {
        self.grid_offset.0
    }

    /// Get the maximum bound on the `X` axis.
    pub fn x_max(&self) -> i32 {
        add_u32_to_i32(self.grid_offset.0, self.size.0)
    }

    /// Get the minimum bound on the `Y` axis.
    pub fn y_min(&self) -> i32 {
        self.grid_offset.1
    }

    /// Get the maximum bound on the `Y` axis.
    pub fn y_max(&self) -> i32 {
        add_u32_to_i32(self.grid_offset.1, self.size.1)
    }

    /// Get the minimum bound on the `Z` axis.
    pub fn z_min(&self) -> i32 {
        self.grid_offset.2
    }

    /// Get the maximum bound on the `Z` axis.
    pub fn z_max(&self) -> i32 {
        add_u32_to_i32(self.grid_offset.2, self.size.2)
    }

    /// Get the bounds of the grid.
    pub fn bounds(&self) -> Bounds3D {
        Bounds3D {
            min: self.grid_offset,
            max: (self.x_max(), self.y_max(), self.z_max()),
        }
    }

    /// This is equivalent to the volume (width * height * depth).
    pub fn len(&self) -> usize {
        self.size.0 as usize * self.size.1 as usize * self.size.2 as usize
    }

    /// Get an iterator over the cells in the grid.
    pub fn iter(&self) -> RollGrid3DIterator<T> {
        RollGrid3DIterator {
            bounds_iter: self.bounds().iter(),
            grid: self,
        }
    }

    /// Get a mutable iterator over the cells in the grid.
    pub fn iter_mut(&mut self) -> RollGrid3DMutIterator<T> {
        RollGrid3DMutIterator {
            bounds_iter: self.bounds().iter(),
            grid: self,
        }
    }
}

impl<T: Copy> RollGrid3D<T> {
    /// Get a copy of the grid value.
    pub fn get_copy(&self, coord: (i32, i32, i32)) -> Option<T> {
        let index = self.offset_index(coord)?;
        Some(self.cells[index])
    }

    /// Copy a subsection of the grid.
    pub fn copy_subgrid(&self, bounds: Bounds3D) -> Grid3D<T> {
        let self_bounds = self.bounds();
        if bounds.x_min() < self_bounds.x_min()
            || bounds.y_min() < self_bounds.y_min()
            || bounds.z_min() < self_bounds.z_min()
            || bounds.x_max() > self_bounds.x_max()
            || bounds.y_max() > self_bounds.y_max()
            || bounds.z_max() > self_bounds.z_max()
        {
            OUT_OF_BOUNDS.panic();
        }
        Grid3D::new(bounds.size(), bounds.min, |pos| self[pos])
    }
}

impl<T: Clone> RollGrid3D<T> {
    /// Get a clone of the grid value.
    pub fn get_clone(&self, coord: (i32, i32, i32)) -> Option<T> {
        let index = self.offset_index(coord)?;
        Some(self.cells[index].clone())
    }

    /// Clone a subsection of the grid.
    pub fn clone_subgrid(&self, bounds: Bounds3D) -> Grid3D<T> {
        let self_bounds = self.bounds();
        if bounds.x_min() < self_bounds.x_min()
            || bounds.y_min() < self_bounds.y_min()
            || bounds.z_min() < self_bounds.z_min()
            || bounds.x_max() > self_bounds.x_max()
            || bounds.y_max() > self_bounds.y_max()
            || bounds.z_max() > self_bounds.z_max()
        {
            OUT_OF_BOUNDS.panic();
        }
        Grid3D::new(bounds.size(), bounds.min, |pos| self[pos].clone())
    }
}

impl<T: Clone> Clone for RollGrid3D<T> {
    fn clone(&self) -> Self {
        Self {
            cells: self.cells.clone(),
            size: self.size,
            wrap_offset: self.wrap_offset,
            grid_offset: self.grid_offset,
        }
    }
}

impl<T> std::ops::Index<(i32, i32, i32)> for RollGrid3D<T> {
    type Output = T;
    fn index(&self, index: (i32, i32, i32)) -> &Self::Output {
        let index = OUT_OF_BOUNDS.expect(self.offset_index(index));
        &self.cells[index]
    }
}

impl<T> std::ops::IndexMut<(i32, i32, i32)> for RollGrid3D<T> {
    fn index_mut(&mut self, index: (i32, i32, i32)) -> &mut Self::Output {
        let index = OUT_OF_BOUNDS.expect(self.offset_index(index));
        &mut self.cells[index]
    }
}

/// Iterator over all cells in a [RollGrid3D].
pub struct RollGrid3DIterator<'a, T> {
    grid: &'a RollGrid3D<T>,
    bounds_iter: Bounds3DIter,
}

impl<'a, T> Iterator for RollGrid3DIterator<'a, T> {
    type Item = ((i32, i32, i32), &'a T);

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.bounds_iter.size_hint()
    }

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.bounds_iter.next()?;
        let index = self.grid.offset_index(next)?;
        Some((next, &self.grid.cells[index]))
    }
}

/// Mutable iterator over all cells in the [RollGrid3D].
pub struct RollGrid3DMutIterator<'a, T> {
    grid: &'a mut RollGrid3D<T>,
    bounds_iter: Bounds3DIter,
}

impl<'a, T> Iterator for RollGrid3DMutIterator<'a, T> {
    type Item = ((i32, i32, i32), &'a mut T);

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.bounds_iter.size_hint()
    }

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.bounds_iter.next()?;
        let index = self.grid.offset_index(next)?;
        // Only way to do this is with unsafe code.
        unsafe {
            let cells_ptr = self.grid.cells.as_mut_ptr();
            let cell_ptr = cells_ptr.add(index);
            Some((next, cell_ptr.as_mut().unwrap()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iter_test() {
        let mut grid = RollGrid3D::new((2, 2, 2), (0, 0, 0), |pos: (i32, i32, i32)| pos);
        grid.iter().for_each(|(pos, cell)| {
            assert_eq!(pos, *cell);
        });
        grid.iter_mut().for_each(|(_, cell)| {
            cell.0 += 1;
            cell.1 += 1;
            cell.2 += 1;
        });
        grid.iter().for_each(|(pos, cell)| {
            let pos = (pos.0 + 1, pos.1 + 1, pos.2 + 1);
            assert_eq!(*cell, pos);
        });
    }

    #[test]
    fn reposition_test() {
        fn verify_grid(grid: &RollGrid3D<(i32, i32, i32)>) {
            for y in grid.y_min()..grid.y_max() {
                for z in grid.z_min()..grid.z_max() {
                    for x in grid.x_min()..grid.x_max() {
                        let pos = (x, y, z);
                        let cell = grid.get(pos).unwrap();
                        assert_eq!(pos, *cell);
                    }
                }
            }
        }
        fn reload(old: (i32, i32, i32), new: (i32, i32, i32), cell: &mut (i32, i32, i32)) {
            assert_eq!(old, *cell);
            *cell = new;
        }
        let mut grid = RollGrid3D::new((4, 4, 4), (0, 0, 0), |pos| pos);
        verify_grid(&grid);
        for y in -10..11 {
            for z in -10..11 {
                for x in -10..11 {
                    grid.translate((x, y, z), reload);
                    verify_grid(&grid);
                }
            }
        }
    }

    #[test]
    fn resize_and_reposition_test() {
        struct DropCoord {
            coord: (i32, i32, i32),
            unloaded: bool,
        }
        impl From<(i32, i32, i32)> for DropCoord {
            fn from(value: (i32, i32, i32)) -> Self {
                Self {
                    coord: value,
                    unloaded: false,
                }
            }
        }
        impl Drop for DropCoord {
            fn drop(&mut self) {
                assert!(self.unloaded);
            }
        }
        fn verify_grid(grid: &RollGrid3D<DropCoord>) {
            for y in grid.y_min()..grid.y_max() {
                for z in grid.z_min()..grid.z_max() {
                    for x in grid.x_min()..grid.x_max() {
                        let pos = (x, y, z);
                        let cell = grid.get(pos).expect("Cell was None");
                        assert_eq!(pos, cell.coord);
                    }
                }
            }
        }
        for y in -7..7 {
            for z in -7..7 {
                for x in -7..7 {
                    for ny in -7..7 {
                        for nz in -7..7 {
                            for nx in -7..7 {
                                let mut grid = RollGrid3D::new((2, 2, 2), (x, y, z), |pos| {
                                    DropCoord::from(pos)
                                });
                                grid.reposition((nx, ny, nz), |old_pos, new_pos, cell| {
                                    assert_eq!(cell.coord, old_pos);
                                    cell.coord = new_pos;
                                });
                                verify_grid(&grid);
                                grid.iter_mut().for_each(|(pos, cell)| {
                                    assert_eq!(pos, cell.coord);
                                    cell.unloaded = true;
                                });
                            }
                        }
                    }
                }
            }
        }
        for height in 1..7 {
            for depth in 1..7 {
                for width in 1..7 {
                    for y in -1..6 {
                        for z in -1..6 {
                            for x in -1..6 {
                                let mut grid = RollGrid3D::new(
                                    (4, 4, 4),
                                    (0, 0, 0),
                                    |pos: (i32, i32, i32)| DropCoord::from(pos),
                                );
                                // reposition to half point to ensure wrapping doesn't cause lookup invalidation.
                                grid.reposition((2, 2, 2), |old_pos, new_pos, cell| {
                                    assert_eq!(old_pos, cell.coord);
                                    cell.coord = new_pos;
                                });
                                grid.resize_and_reposition(
                                    (width, height, depth),
                                    (x, y, z),
                                    cell_manager(
                                        // Load
                                        |pos| DropCoord::from(pos),
                                        // Unload
                                        |pos, mut old_value| {
                                            assert_eq!(pos, old_value.coord);
                                            old_value.unloaded = true;
                                        },
                                        // Reload
                                        |old_pos, new_pos, cell| {
                                            cell.unloaded = true;
                                            assert_eq!(old_pos, cell.coord);
                                            let mut old =
                                                std::mem::replace(cell, DropCoord::from(new_pos));
                                            old.unloaded = true;
                                        },
                                    ),
                                );
                                grid.iter_mut().for_each(|(_, cell)| {
                                    cell.unloaded = true;
                                });
                                verify_grid(&grid);
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn offsetfix_test() {
        struct OffsetFix {
            /// the old grid offset that we can use to
            /// create a relational offset
            offset: (i32, i32, i32),
            size: (i32, i32, i32),
        }
        impl OffsetFix {
            fn wrap(&self, pos: (i32, i32, i32)) -> (i32, i32, i32) {
                let x = (pos.0 - self.offset.0).rem_euclid(self.size.0) + self.offset.0;
                let y = (pos.1 - self.offset.1).rem_euclid(self.size.1) + self.offset.1;
                let z = (pos.2 - self.offset.2).rem_euclid(self.size.2) + self.offset.2;
                (x, y, z)
            }
        }
        let fix = OffsetFix {
            offset: (5, 5, 5),
            size: (4, 4, 4),
        };
        let (x, y, z) = fix.wrap((9, 9, 9));
        println!("({x}, {y}, {z})");
    }

    #[test]
    fn offset_index_test() {
        struct Grid {
            offset: (i32, i32, i32),
            size: (i32, i32, i32),
        }
        impl Grid {
            fn offset_index(&self, x: i32, y: i32, z: i32) -> Option<usize> {
                if x < self.offset.0
                    || y < self.offset.1
                    || z < self.offset.2
                    || x > self.offset.0 + self.size.0
                    || y > self.offset.1 + self.size.1
                    || z > self.offset.2 + self.size.2
                {
                    return None;
                }
                let x = x - self.offset.0;
                let y = y - self.offset.1;
                let z = z - self.offset.2;
                let wd = self.size.0 * self.size.2;
                Some((y * wd + z * self.size.0 + x) as usize)
            }
            fn index_offset(&self, index: usize) -> Option<(i32, i32, i32)> {
                let volume = (self.size.0 * self.size.1 * self.size.2) as usize;
                if index >= volume {
                    return None;
                }
                let index = index as i32;
                let wd = self.size.0 * self.size.2;
                let y = index / wd;
                let xz_rem = index.rem_euclid(wd);
                let z = xz_rem / self.size.0;
                let x = xz_rem.rem_euclid(self.size.0);
                Some((x + self.offset.0, y + self.offset.1, z + self.offset.2))
            }
        }

        let grid = Grid {
            offset: (-3, -1, -5),
            size: (23, 32, 18),
        };
        let index = grid.offset_index(0, 0, 0).expect(OUT_OF_BOUNDS.msg());
        assert_eq!(index, 532);
        let (x, y, z) = grid.index_offset(index).expect(OUT_OF_BOUNDS.msg());
        assert_eq!((x, y, z), (0, 0, 0));
        for y in grid.offset.1..grid.offset.1 + grid.size.1 {
            for z in grid.offset.2..grid.offset.2 + grid.size.2 {
                for x in grid.offset.0..grid.offset.0 + grid.size.0 {
                    let index = grid.offset_index(x, y, z).expect(OUT_OF_BOUNDS.msg());
                    let (rx, ry, rz) = grid.index_offset(index).expect(OUT_OF_BOUNDS.msg());
                    assert_eq!((rx, ry, rz), (x, y, z));
                }
            }
        }
    }

    #[test]
    fn bounds_test() {
        let max_bounds = Bounds3D::new(
            (i32::MIN, i32::MIN, i32::MIN),
            (i32::MAX, i32::MAX, i32::MAX),
        );
        println!("{}", max_bounds.volume());
    }
}
