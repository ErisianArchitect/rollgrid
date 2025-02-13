use crate::{bounds2d::*, error_messages::*, fixedarray::FixedArray, math::*, *};

/// A 2D implementation of a rolling grid. It's a data structure similar
/// to a circular buffer in the sense that cells can wrap around.
/// It uses the modulus operator combined with an internal wrap offset to
/// create the illusion that cells are being moved while the cells remain
/// in the same position in the underlying array.
pub struct RollGrid2D<T: Sized> {
    cells: FixedArray<T>,
    size: (u32, u32),
    wrap_offset: (u32, u32),
    grid_offset: (i32, i32),
}

unsafe impl<T: Send> Send for RollGrid2D<T> {}
unsafe impl<T: Sync> Sync for RollGrid2D<T> {}

// TODO: Operations that take dimensions should take tuple instead of individual dimension arguments.

impl<T: Default> RollGrid2D<T> {
    /// Create a new [RollGrid2D] with all the cells set to the default for `T`.
    pub fn new_default(width: u32, height: u32, grid_offset: (i32, i32)) -> Self {
        Self {
            cells: FixedArray::new_2d((width, height), grid_offset, |_| T::default()),
            size: (width, height),
            grid_offset: grid_offset,
            wrap_offset: (0, 0),
        }
    }
}

impl RollGrid2D<()> {
    /// Creates a new grid of unit types.
    pub fn new_zst(width: u32, height: u32, grid_offset: (i32, i32)) -> Self {
        let size = (width, height);
        RollGrid2D {
            cells: FixedArray::new_2d(size, grid_offset, |_| ()),
            size,
            grid_offset,
            wrap_offset: (0, 0),
        }
    }
}

impl<T> RollGrid2D<T> {
    /// Create a new [RollGrid2D] using an initialize function to initialize cells.
    ///
    /// The init function should take as input the coordinate that is being
    /// initialized, and should return the desired value for the cell.
    pub fn new<F: FnMut((i32, i32)) -> T>(
        width: u32,
        height: u32,
        grid_offset: (i32, i32),
        init: F,
    ) -> Self {
        Self {
            cells: FixedArray::new_2d((width, height), grid_offset, init),
            size: (width, height),
            wrap_offset: (0, 0),
            grid_offset: grid_offset,
        }
    }

    /// Try to create a new [RollGrid2D] using a fallible initialize function to initialize elements.
    ///
    /// The init function should take as input the coordinate that is being
    /// initialized, and should return the desired value for the cell.
    pub fn try_new<E, F: FnMut((i32, i32)) -> Result<T, E>>(
        width: u32,
        height: u32,
        grid_offset: (i32, i32),
        init: F,
    ) -> Result<Self, E> {
        Ok(Self {
            cells: FixedArray::try_new_2d((width, height), grid_offset, init)?,
            size: (width, height),
            wrap_offset: (0, 0),
            grid_offset: grid_offset,
        })
    }

    /// Inflate the size by `inflate`, keeping the bounds centered.
    ///
    /// If the size is `(2, 2)` with an offset of `(1, 1)`, and you want to inflate by `(1, 1)`.
    /// The result of that operation would have a size of `(4, 4)` and an offset of `(0, 0)`.
    ///
    /// # Example
    /// ```rust, no_run
    /// grid.inflate_size((1, 1), cell_manager(
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
    pub fn inflate_size<M>(&mut self, inflate: (u32, u32), manage: M)
    where
        M: CellManage<(i32, i32), T>,
    {
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
        // FIXME: check for overflow on max bounds.
        let off_x = self.grid_offset.0 as i64;
        let off_y = self.grid_offset.1 as i64;
        let pos_x = off_x - inflate.0 as i64;
        INFLATE_OVERFLOW.panic_if(pos_x < i32::MIN as i64);
        let pos_y = off_y - inflate.1 as i64;
        INFLATE_OVERFLOW.panic_if(pos_y < i32::MIN as i64);
        let position = (
            pos_x as i32,
            pos_y as i32,
        );
        self.resize_and_reposition(width, height, position, manage);
    }

    /// Try to inflate the size by `inflate` using a fallible function, keeping the bounds centered.
    ///
    /// If the size is `(2, 2)` with an offset of `(1, 1)`, and you want to inflate by `(1, 1)`.
    /// The result of that operation would have a size of `(4, 4)` and an offset of `(0, 0)`.
    ///
    /// # Panics
    /// - If either dimension of `inflate` exceeds `i32::MAX`.
    /// - If either dimension of the inflated size exceeds `u32::MAX`
    /// # Example
    /// ```rust, no_run
    /// grid.try_inflate_size((1, 1), try_cell_manager(
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
    pub fn try_inflate_size<E, M>(&mut self, inflate: (u32, u32), manage: M) -> Result<(), E>
    where
        M: TryCellManage<(i32, i32), T, E>,
    {
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
        // FIXME: check for overflow on max bounds.
        let off_x = self.grid_offset.0 as i64;
        let off_y = self.grid_offset.1 as i64;
        let pos_x = off_x - inflate.0 as i64;
        INFLATE_OVERFLOW.panic_if(pos_x < i32::MIN as i64);
        let pos_y = off_y - inflate.1 as i64;
        INFLATE_OVERFLOW.panic_if(pos_y < i32::MIN as i64);
        let position = (
            pos_x as i32,
            pos_y as i32,
        );
        self.try_resize_and_reposition(width, height, position, manage)
    }

    /// Deflate the size by `deflate`, keeping the bounds centered.
    ///
    /// If the size is `(4, 4)` with an offset of `(0, 0)`, and you want to deflate by `(1, 1)`.
    /// The result of that operation would have a size of `(2, 2)` and an offset of `(1, 1)`.
    ///
    /// # Panics
    /// - If either dimension of `inflate` exceeds `i32::MAX`.
    /// - If either dimension of the inflated size exceeds `u32::MAX`
    /// # Example
    /// ```rust, no_run
    /// grid.deflate_size((1, 1), cell_manager(
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
    pub fn deflate_size<M>(&mut self, deflate: (u32, u32), manage: M)
    where
        M: CellManage<(i32, i32), T>,
    {
        let width = self
            .size
            .0
            .checked_sub(deflate.0.checked_mul(2).expect(DEFLATE_OVERFLOW.msg()))
            .expect(DEFLATE_OVERFLOW.msg());
        let height = self
            .size
            .0
            .checked_sub(deflate.0.checked_mul(2).expect(DEFLATE_OVERFLOW.msg()))
            .expect(DEFLATE_OVERFLOW.msg());
        AREA_IS_ZERO.panic_if(width == 0 || height == 0);
        let (off_x, off_y): (i64, i64) = self.grid_offset.convert();
        let pos_x = off_x + deflate.0 as i64;
        DEFLATE_OVERFLOW.panic_if(pos_x > i32::MAX as i64);
        let pos_y = off_y + deflate.1 as i64;
        DEFLATE_OVERFLOW.panic_if(pos_y > i32::MAX as i64);
        let position = (
            pos_x as i32,
            pos_y as i32,
        );
        self.resize_and_reposition(width, height, position, manage);
    }

    /// Try to deflate the size by `deflate` using a fallible function, keeping the bounds centered.
    ///
    /// If the size is `(4, 4)` with an offset of `(0, 0)`, and you want to deflate by `(1, 1)`.
    /// The result of that operation would have a size of `(2, 2)` and an offset of `(1, 1)`.
    ///
    /// # Panics
    /// - If either dimension of `inflate` exceeds `i32::MAX`.
    /// - If either dimension of the inflated size exceeds `u32::MAX`
    /// # Example
    /// ```rust, no_run
    /// grid.try_deflate_size((1, 1), try_cell_manager(
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
    pub fn try_deflate_size<E, M>(&mut self, deflate: (u32, u32), manage: M) -> Result<(), E>
    where
        M: TryCellManage<(i32, i32), T, E>,
    {
        let width = self
            .size
            .0
            .checked_sub(deflate.0.checked_mul(2).expect(DEFLATE_OVERFLOW.msg()))
            .expect(DEFLATE_OVERFLOW.msg());
        let height = self
            .size
            .0
            .checked_sub(deflate.0.checked_mul(2).expect(DEFLATE_OVERFLOW.msg()))
            .expect(DEFLATE_OVERFLOW.msg());
        AREA_IS_ZERO.panic_if(width == 0 || height == 0);
        let (off_x, off_y): (i64, i64) = self.grid_offset.convert();
        let pos_x = off_x + deflate.0 as i64;
        DEFLATE_OVERFLOW.panic_if(pos_x > i32::MAX as i64);
        let pos_y = off_y + deflate.1 as i64;
        DEFLATE_OVERFLOW.panic_if(pos_y > i32::MAX as i64);
        let position = (
            pos_x as i32,
            pos_y as i32,
        );
        self.try_resize_and_reposition(width, height, position, manage)
    }

    /// Resize the grid without changing the offset.
    ///
    /// # Panics
    /// - If either dimension of `inflate` exceeds `i32::MAX`.
    /// - If either dimension of the inflated size exceeds `u32::MAX`
    /// # Example
    /// ```no_run
    /// grid.resize(3, 3, cell_manager(
    ///     // Load
    ///     |pos| {
    ///         println!("Load: {:?}", pos);
    ///         // return the loaded value
    ///         // Typically you wouldn't return the position,
    ///         // you would want to load a new cell here.
    ///         pos
    ///     },
    ///     // Unload
    ///     |pos, value| {
    ///         println!("Unload: {:?}", pos);
    ///     },
    ///     // Reload
    ///     |old_pos, new_pos, value| {
    ///         println!("Reload({:?}, {:?})")
    ///     }
    /// ));
    /// ```
    /// See [CellManage].
    pub fn resize<M>(&mut self, new_width: u32, new_height: u32, manage: M)
    where
        M: CellManage<(i32, i32), T>,
    {
        self.resize_and_reposition(new_width, new_height, self.grid_offset, manage);
    }

    /// Try to resize the grid with a fallible function without changing the offset.
    ///
    /// # Example
    /// ```rust, no_run
    /// grid.try_resize(1, 1, cell_manager(
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
    pub fn try_resize<E, M>(&mut self, new_width: u32, new_height: u32, manage: M) -> Result<(), E>
    where
        M: TryCellManage<(i32, i32), T, E>,
    {
        self.try_resize_and_reposition(new_width, new_height, self.grid_offset, manage)
    }

    /// Resize and reposition the grid simultaneously.
    ///
    /// # Example
    /// ```rust, no_run
    /// grid.resize_and_reposition(3, 3, (4, 4), cell_manager(
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
        width: u32,
        height: u32,
        new_position: (i32, i32),
        manage: M,
    ) where
        M: CellManage<(i32, i32), T>,
    {
        let mut manage = manage;
        if (width, height) == self.size {
            if new_position != self.grid_offset {
                self.reposition(new_position, |old_pos, new_pos, cell| {
                    manage.reload(old_pos, new_pos, cell);
                });
            }
            return;
        }
        AREA_IS_ZERO.panic_if(width == 0 || height == 0);
        let (new_x, new_y) = new_position;
        let right = RESIZE_OVERFLOW.expect(checked_add_u32_to_i32(new_x, width));
        let bottom = RESIZE_OVERFLOW.expect(checked_add_u32_to_i32(new_y, height));
        // Determine what needs to be unloaded
        let old_bounds: Bounds2D = self.bounds();
        let new_bounds = Bounds2D::new((new_x, new_y), (right, bottom));
        if old_bounds.intersects(new_bounds) {
            macro_rules! unload_bounds {
                ($cond: expr => xmin = $xmin:expr; ymin = $ymin:expr; xmax = $xmax:expr; ymax = $ymax:expr;) => {
                    if $cond {
                        Bounds2D::new(($xmin, $ymin), ($xmax, $ymax))
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
            unload_bounds!(old_bounds.x_min() < new_bounds.x_min() =>
                xmin = old_bounds.x_min();
                ymin = new_bounds.y_min().max(old_bounds.y_min());
                xmax = new_bounds.x_min();
                ymax = old_bounds.y_max();
            );
            unload_bounds!(old_bounds.y_min() < new_bounds.y_min() =>
                xmin = old_bounds.x_min();
                ymin = old_bounds.y_min();
                xmax = new_bounds.x_max().min(old_bounds.x_max());
                ymax = new_bounds.y_min();
            );
            unload_bounds!(old_bounds.x_max() > new_bounds.x_max() =>
                xmin = new_bounds.x_max();
                ymin = old_bounds.y_min();
                xmax = old_bounds.x_max();
                ymax = new_bounds.y_max().min(old_bounds.y_max());
            );
            unload_bounds!(old_bounds.y_max() > new_bounds.y_max() =>
                xmin = new_bounds.x_min().max(old_bounds.x_min());
                ymin = new_bounds.y_max();
                xmax = old_bounds.x_max();
                ymax = old_bounds.y_max();
            );
            let new_grid = FixedArray::new_2d((width, height), new_position, |pos| {
                if old_bounds.contains(pos) {
                    let index = self.offset_index(pos).expect(OUT_OF_BOUNDS.msg());
                    unsafe { self.cells.read(index) }
                } else {
                    manage.load(pos)
                }
            });
            self.size = (width, height);
            self.grid_offset = new_position;
            unsafe {
                self.cells.forget_dealloc();
            }
            self.cells = new_grid;
            self.wrap_offset = (0, 0);
        } else {
            // !old_bounds.intersects(new_bounds)
            old_bounds.iter().for_each(|pos| {
                let index = self.offset_index(pos).expect(OUT_OF_BOUNDS.msg());
                unsafe {
                    manage.unload(pos, self.cells.read(index));
                }
            });
            let new_grid =
                FixedArray::new_2d((width, height), new_position, |pos| manage.load(pos));
            self.size = (width, height);
            self.grid_offset = new_position;
            unsafe {
                self.cells.forget_dealloc();
            }
            self.cells = new_grid;
            self.wrap_offset = (0, 0)
        }
    }

    /// Try to resize and reposition the grid using a fallible function.
    ///
    /// # Example
    /// ```rust, no_run
    /// grid.try_resize_and_reposition(3, 3, (4, 4), try_cell_manager(
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
        width: u32,
        height: u32,
        new_position: (i32, i32),
        manage: M,
    ) -> Result<(), E>
    where
        M: TryCellManage<(i32, i32), T, E>,
    {
        let mut manage = manage;
        if (width, height) == self.size {
            if new_position != self.grid_offset {
                self.try_reposition(new_position, |old_pos, new_pos, cell| {
                    manage.try_reload(old_pos, new_pos, cell)
                })?;
            }
            return Ok(());
        }
        AREA_IS_ZERO.panic_if(width == 0 || height == 0);
        let (new_x, new_y) = new_position;
        let right = RESIZE_OVERFLOW.expect(checked_add_u32_to_i32(new_x, width));
        let bottom = RESIZE_OVERFLOW.expect(checked_add_u32_to_i32(new_y, height));
        // Determine what needs to be unloaded
        let old_bounds: Bounds2D = self.bounds();
        let new_bounds = Bounds2D::new((new_x, new_y), (right, bottom));
        // TODO: When width and height are returned to u32, remove conversions.
        // FIXME: size is set too early. It should be set after creation of new FixedArray.
        let size = (width as u32, height as u32);
        if old_bounds.intersects(new_bounds) {
            macro_rules! unload_bounds {
                ($cond: expr => xmin = $xmin:expr; ymin = $ymin:expr; xmax = $xmax:expr; ymax = $ymax:expr;) => {
                    if $cond {
                        Bounds2D::new(($xmin, $ymin), ($xmax, $ymax))
                            .iter()
                            .try_for_each(|pos| {
                                let index = self.offset_index(pos).expect(OUT_OF_BOUNDS.msg());
                                unsafe {
                                    manage.try_unload(pos, self.cells.read(index))?;
                                }
                                Ok(())
                            })?;
                    }
                };
            }
            unload_bounds!(old_bounds.x_min() < new_bounds.x_min() =>
                xmin = old_bounds.x_min();
                ymin = new_bounds.y_min().max(old_bounds.y_min());
                xmax = new_bounds.x_min();
                ymax = old_bounds.y_max();
            );
            unload_bounds!(old_bounds.y_min() < new_bounds.y_min() =>
                xmin = old_bounds.x_min();
                ymin = old_bounds.y_min();
                xmax = new_bounds.x_max().min(old_bounds.x_max());
                ymax = new_bounds.y_min();
            );
            unload_bounds!(old_bounds.x_max() > new_bounds.x_max() =>
                xmin = new_bounds.x_max();
                ymin = old_bounds.y_min();
                xmax = old_bounds.x_max();
                ymax = new_bounds.y_max().min(old_bounds.y_max());
            );
            unload_bounds!(old_bounds.y_max() > new_bounds.y_max() =>
                xmin = new_bounds.x_min().max(old_bounds.x_min());
                ymin = new_bounds.y_max();
                xmax = old_bounds.x_max();
                ymax = old_bounds.y_max();
            );
            let new_grid = FixedArray::try_new_2d(size, new_position, |pos| {
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
            self.wrap_offset = (0, 0);
        } else {
            // !old_bounds.intersects(new_bounds)
            old_bounds.iter().try_for_each(|pos| {
                let index = self.offset_index(pos).expect(OUT_OF_BOUNDS.msg());
                unsafe {
                    manage.try_unload(pos, self.cells.read(index))?;
                }
                Ok(())
            })?;
            let new_grid = FixedArray::try_new_2d(size, new_position, |pos| manage.try_load(pos))?;
            self.size = size;
            self.grid_offset = new_position;
            unsafe {
                self.cells.forget_dealloc();
            }
            self.cells = new_grid;
            self.wrap_offset = (0, 0);
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
    /// grid.translate((2, 4), |old_position, new_position, cell_mut| {
    ///     *cell_mut = new_position;
    /// })
    /// ```
    pub fn translate<F>(&mut self, offset: (i32, i32), reload: F)
    where
        F: FnMut((i32, i32), (i32, i32), &mut T),
    {
        let (curx, cury) = self.grid_offset;
        let (ox, oy) = offset;
        // FIXME: Check for overflow.
        self.reposition((curx + ox, cury + oy), reload);
    }

    /// Try to translate the grid by offset amount using a fallible reload function.
    ///
    /// The reload function takes the old position, the new position, and
    /// a mutable reference to the cell where the initial value of the cell
    /// when called is the value at `old_position`. You want to change the
    /// cell to the correct value for a cell at `new_position`.
    ///
    /// # Example
    /// ```rust, no_run
    /// grid.try_translate((2, 3), |old_position, new_position, cell_mut| {
    ///     *cell_mut = new_position;
    ///     Ok(())
    /// })
    /// ```
    pub fn try_translate<E, F>(&mut self, offset: (i32, i32), reload: F) -> Result<(), E>
    where
        F: FnMut((i32, i32), (i32, i32), &mut T) -> Result<(), E>,
    {
        let (curx, cury) = self.grid_offset;
        let (ox, oy) = offset;
        // FIXME: Check for overflow.
        self.try_reposition((curx + ox, cury + oy), reload)
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
    /// grid.reposition((2, 3), |old_position, new_position, cell_mut| {
    ///     *cell_mut = new_position;
    /// })
    /// ```
    pub fn reposition<F>(&mut self, position: (i32, i32), reload: F)
    where
        F: FnMut((i32, i32), (i32, i32), &mut T),
    {
        let mut reload = reload;
        if self.grid_offset == position {
            return;
        }
        let (old_x, old_y) = self.grid_offset;
        let (new_x, new_y) = position;
        // TODO: Change this to work directly with the new position rather than
        //       an offset.
        // FIXME: This operation will overflow under certain conditions.
        //        Use i64 for these overflowing operations.
        let offset = (new_x - old_x, new_y - old_y);
        // FIXME: Don't convert width and height to i32. keep them as u32.
        let width = self.size.0 as i32;
        let height = self.size.1 as i32;
        let (offset_x, offset_y) = offset;
        // FIXME: grid_offset is assigned too early.
        //        also, wtf? How does this function work if grid_offset is
        //        assigned early? I believe this function relies on
        //        grid_offset remaining the same until after the reposition
        //        operation completes.
        //        Update: This function DOES NOT work. Changing the grid
        //        offset prematurely seems to break it. I don't know how
        //        I missed this before. I used this crate without problems...
        //        THIS IS A MAJOR BUG! FIX ASAP.
        self.grid_offset = (new_x, new_y);
        // Offset is within bounds, so that means that the grid will be rolled.
        // This allows for bounded reloading of the grid elements.
        // If rolling causes a section to remain on the grid, that section will not be reloaded.
        // Only the elements that are considered new will be reloaded.
        if offset_x.abs() < width && offset_y.abs() < height {
            // TODO: Work out how this works again so I can document it, and
            //       figure out edge cases.
            let (roll_x, roll_y) = (self.wrap_offset.0 as i64, self.wrap_offset.1 as i64);
            // TODO: This usage of rem_euclid might be wrong.
            let (wrapped_offset_x, wrapped_offset_y) =
                (offset_x.rem_euclid(width), offset_y.rem_euclid(height));
            // Update the roll so that we reduce reloading.
            // Without using the roll functionality, this function would demand to reload
            // every single cell, even if it only needed to reload 8 out of 64 cells.
            // TODO: Check for overflow.
            let new_rolled_x = (roll_x + wrapped_offset_x).rem_euclid(width);
            let new_rolled_y = (roll_y + wrapped_offset_y).rem_euclid(height);
            self.wrap_offset = (new_rolled_x, new_rolled_y);
            // FIXME: Use checked addition with i32 + u32. (width will be changed
            //        back to u32)
            let right = new_x + width;
            let bottom = new_y + height;
            // Calculate ranges
            // Combining new_x_range and new_y_range gets the corner.
            // The partition on either the left or right side
            // FIXME: There are overflowing subtractions ahead.
            //        Use i64 to prevent overflow.
            let new_x_range = if offset_x >= 0 {
                (right - offset_x)..right
            } else {
                new_x..new_x - offset_x
            };
            let new_x_range_y_range = if offset_y >= 0 {
                new_y..(bottom - offset_y)
            } else {
                new_y - offset_y..bottom
            };
            // The partition on either the top or the bottom.
            let new_y_range = if offset_y >= 0 {
                (bottom - offset_y)..bottom
            } else {
                new_y..new_y - offset_y
            };
            let new_y_range_x_range = if offset_x >= 0 {
                new_x..(right - offset_x)
            } else {
                new_x - offset_x..right
            };
            // The left/right partition
            for y in new_x_range_y_range.clone() {
                for (xi, x) in new_x_range.clone().enumerate() {
                    // TODO: This smells bad. Take a closer look.
                    let prior_x = if offset_x >= 0 {
                        old_x + xi as i32
                    } else {
                        old_x + width + offset_x + xi as i32
                    };
                    let prior_y = y;
                    let index = self.offset_index((x, y)).expect(OUT_OF_BOUNDS.msg());
                    reload((prior_x, prior_y), (x, y), &mut self.cells[index]);
                }
            }
            // The top/bottom partition
            for (iy, y) in new_y_range.clone().enumerate() {
                for x in new_y_range_x_range.clone() {
                    // TODO: This smells bad. Take a closer look.
                    let prior_x = x;
                    let prior_y = if offset_y >= 0 {
                        old_y + iy as i32
                    } else {
                        old_y + height + offset_y + iy as i32
                    };
                    let index = self.offset_index((x, y)).expect(OUT_OF_BOUNDS.msg());
                    reload((prior_x, prior_y), (x, y), &mut self.cells[index]);
                }
            }
            // The corner partition
            for (iy, y) in new_y_range.enumerate() {
                for (ix, x) in new_x_range.clone().enumerate() {
                    // TODO: This smells bad. Take a closer look.
                    let prior_x = if offset_x >= 0 {
                        old_x + ix as i32
                    } else {
                        old_x + width + offset_x + ix as i32
                    };
                    let prior_y = if offset_y >= 0 {
                        old_y + iy as i32
                    } else {
                        old_y + height + offset_y + iy as i32
                    };
                    let index = self.offset_index((x, y)).expect(OUT_OF_BOUNDS.msg());
                    reload((prior_x, prior_y), (x, y), &mut self.cells[index]);
                }
            }
        } else {
            // Reload everything
            for (yi, y) in (new_y..new_y + height).enumerate() {
                for (xi, x) in (new_x..new_x + width).enumerate() {
                    // TODO: This smells bad. Take a closer look.
                    let prior_x = old_x + xi as i32;
                    let prior_y = old_y + yi as i32;
                    let index = self.offset_index((x, y)).expect(OUT_OF_BOUNDS.msg());
                    reload((prior_x, prior_y), (x, y), &mut self.cells[index]);
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
    /// })
    /// ```
    pub fn try_reposition<E, F>(&mut self, position: (i32, i32), reload: F) -> Result<(), E>
    where
        F: FnMut((i32, i32), (i32, i32), &mut T) -> Result<(), E>,
    {
        if self.grid_offset == position {
            return Ok(());
        }
        let (old_x, old_y) = self.grid_offset;
        let (new_x, new_y) = position;
        // TODO: Change this to work directly with the new position rather than
        //       an offset.
        // FIXME: This operation will overflow under certain conditions.
        //        Use i64 for these overflowing operations.
        let offset = (new_x - old_x, new_y - old_y);
        let mut reload = reload;
        // FIXME: Don't convert width and height to i32. keep them as u32.
        let width = self.size.0 as i32;
        let height = self.size.1 as i32;
        let (offset_x, offset_y) = offset;
        // FIXME: grid_offset is assigned too early.
        //        also, wtf? How does this function work if grid_offset is
        //        assigned early? I believe this function relies on
        //        grid_offset remaining the same until after the reposition
        //        operation completes.
        //        Update: This function DOES NOT work. Changing the grid
        //        offset prematurely seems to break it. I don't know how
        //        I missed this before. I used this crate without problems...
        //        THIS IS A MAJOR BUG! FIX ASAP.
        self.grid_offset = (new_x, new_y);
        // Offset is within bounds, so that means that the grid will be rolled.
        // This allows for bounded reloading of the grid elements.
        // If rolling causes a section to remain on the grid, that section will not be reloaded.
        // Only the elements that are considered new will be reloaded.
        if offset_x.abs() < width && offset_y.abs() < height {
            // TODO: Work out how this works again so I can document it, and
            //       figure out edge cases.
            let (roll_x, roll_y) = (self.wrap_offset.0 as i32, self.wrap_offset.1 as i32);
            let (wrapped_offset_x, wrapped_offset_y) =
                (offset_x.rem_euclid(width), offset_y.rem_euclid(height));
            // Update the roll so that we reduce reloading.
            // Without using the roll functionality, this function would demand to reload
            // every single cell, even if it only needed to reload 8 out of 64 cells.
            // TODO: Check for overflow.
            let new_rolled_x = (roll_x + wrapped_offset_x).rem_euclid(width);
            let new_rolled_y = (roll_y + wrapped_offset_y).rem_euclid(height);
            self.wrap_offset = (new_rolled_x, new_rolled_y);
            // FIXME: Use checked addition with i32 + u32. (width will be changed
            //        back to u32)
            let right = new_x + width;
            let bottom = new_y + height;
            // Calculate ranges
            // Combining new_x_range and new_y_range gets the corner.
            // The partition on either the left or right side
            // FIXME: There are overflowing subtractions ahead.
            //        Use i64 to prevent overflow.
            let new_x_range = if offset_x >= 0 {
                (right - offset_x)..right
            } else {
                new_x..new_x - offset_x
            };
            let new_x_range_y_range = if offset_y >= 0 {
                new_y..(bottom - offset_y)
            } else {
                new_y - offset_y..bottom
            };
            // The partition on either the top or the bottom.
            let new_y_range = if offset_y >= 0 {
                (bottom - offset_y)..bottom
            } else {
                new_y..new_y - offset_y
            };
            let new_y_range_x_range = if offset_x >= 0 {
                new_x..(right - offset_x)
            } else {
                new_x - offset_x..right
            };
            // The left/right partition
            for y in new_x_range_y_range.clone() {
                for (xi, x) in new_x_range.clone().enumerate() {
                    // TODO: This smells bad. Take a closer look.
                    let prior_x = if offset_x >= 0 {
                        old_x + xi as i32
                    } else {
                        old_x + width + offset_x + xi as i32
                    };
                    let prior_y = y;
                    let index = self.offset_index((x, y)).expect(OUT_OF_BOUNDS.msg());
                    reload((prior_x, prior_y), (x, y), &mut self.cells[index])?;
                }
            }
            // The top/bottom partition
            for (iy, y) in new_y_range.clone().enumerate() {
                for x in new_y_range_x_range.clone() {
                    // TODO: This smells bad. Take a closer look.
                    let prior_x = x;
                    let prior_y = if offset_y >= 0 {
                        old_y + iy as i32
                    } else {
                        old_y + height + offset_y + iy as i32
                    };
                    let index = self.offset_index((x, y)).expect(OUT_OF_BOUNDS.msg());
                    reload((prior_x, prior_y), (x, y), &mut self.cells[index])?;
                }
            }
            // The corner partition
            for (iy, y) in new_y_range.enumerate() {
                for (ix, x) in new_x_range.clone().enumerate() {
                    // TODO: This smells bad. Take a closer look.
                    let prior_x = if offset_x >= 0 {
                        old_x + ix as i32
                    } else {
                        old_x + width + offset_x + ix as i32
                    };
                    let prior_y = if offset_y >= 0 {
                        old_y + iy as i32
                    } else {
                        old_y + height + offset_y + iy as i32
                    };
                    let index = self.offset_index((x, y)).expect(OUT_OF_BOUNDS.msg());
                    reload((prior_x, prior_y), (x, y), &mut self.cells[index])?;
                }
            }
        } else {
            // Reload everything
            for (yi, y) in (new_y..new_y + height).enumerate() {
                for (xi, x) in (new_x..new_x + width).enumerate() {
                    // TODO: This smells bad. Take a closer look.
                    let prior_x = old_x + xi as i32;
                    let prior_y = old_y + yi as i32;
                    let index = self.offset_index((x, y)).expect(OUT_OF_BOUNDS.msg());
                    reload((prior_x, prior_y), (x, y), &mut self.cells[index])?;
                }
            }
        }
        Ok(())
    }

    // TODO: This function should return (i64, i64), because subtraction
    //       with i32 can overflow. Convert i32s into i64s then perform
    //       calculation.
    /// Get the offset relative to the grid's offset.
    pub fn relative_offset(&self, coord: (i32, i32)) -> (i32, i32) {
        let (x, y) = coord;
        // FIXME: Overflow is possible here.
        (x - self.grid_offset.0, y - self.grid_offset.1)
    }

    /// The grid has a wrapping offset, which dictates the lookup order of cells.
    /// This method allows to find the index of a particular offset in the grid.
    /// Offsets are relative to the world origin `(0, 0, 0)`, and must account for
    /// the grid offset.
    fn offset_index(&self, (x, y): (i32, i32)) -> Option<usize> {
        // FIXME: Variables here have terrible names.
        let (x, y) = (x as i64, y as i64);
        let (mx, my) = self.grid_offset;
        let (mx, my) = (mx as i64, my as i64);
        let width = self.size.0 as i64;
        let height = self.size.1 as i64;
        if x >= mx + width || y >= my + height || x < mx || y < my {
            return None;
        }
        // Adjust x and y
        let nx = x - mx;
        let ny = y - my;
        // Wrap x and y
        let (wrap_x, wrap_y) = (self.wrap_offset.0 as i64, self.wrap_offset.1 as i64);
        let wx = (nx + wrap_x).rem_euclid(width);
        let wy = (ny + wrap_y).rem_euclid(height);
        // TODO: Code smell
        Some((wy as usize * self.size.0 as usize) + wx as usize)
    }

    // TODO: This can fail, so either explain that or return Option<()>.
    /// Replace item at `coord` using `replace` function that takes as
    /// input the old value and returns the new value. This will swap the
    /// value in-place.
    pub fn replace_with<F: FnOnce(T) -> T>(&mut self, coord: (i32, i32), replace: F) {
        let index = self.offset_index(coord).expect(OUT_OF_BOUNDS.msg());
        self.cells.replace_with(index, replace);
    }

    // TODO: This can fail, so either explain that or return Option<T>.
    /// Replace item at `coord` using [std::mem::replace] and then returns
    /// the old value.
    pub fn replace(&mut self, coord: (i32, i32), value: T) -> T {
        let index = self.offset_index(coord).expect(OUT_OF_BOUNDS.msg());
        self.cells.replace(index, value)
    }

    // TODO: Explain the return value. Also, this should use `must_use`.
    /// Reads the value from the cell without moving it. This leaves the memory in the cell unchanged.
    pub unsafe fn read(&self, coord: (i32, i32)) -> Option<T> {
        let index = self.offset_index(coord)?;
        Some(self.cells.read(index))
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

    /// Get a mutable reference to the cell's value if it exists and the coord is in bounds, otherwise return `None`.
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
        self.grid_offset
    }

    /// Get the minimum bound on the `X` axis.
    pub fn x_min(&self) -> i32 {
        self.grid_offset.0
    }
    /// Get the maximum bound on the `X` axis.
    pub fn x_max(&self) -> i32 {
        // FIXME: This might overflow.
        self.grid_offset.0 + self.size.0 as i32
    }

    /// Get the minimum bound on the `Y` axis.
    pub fn y_min(&self) -> i32 {
        self.grid_offset.1
    }

    /// Get the maximum bound on the `Y` axis.
    pub fn y_max(&self) -> i32 {
        // FIXME: This might overflow.
        self.grid_offset.1 + self.size.1 as i32
    }

    /// Get the bounds of the grid.
    pub fn bounds(&self) -> Bounds2D {
        Bounds2D {
            min: (self.x_min(), self.y_min()),
            max: (self.x_max(), self.y_max()),
        }
    }

    /// This is equivalent to the area (width * height).
    pub fn len(&self) -> usize {
        self.size.0 as usize * self.size.1 as usize
    }

    /// Get an iterator over the cells in the grid.
    pub fn iter<'a>(&'a self) -> RollGrid2DIterator<'a, T> {
        RollGrid2DIterator {
            bounds_iter: self.bounds().iter(),
            grid: self,
        }
    }

    /// Get a mutable iterator over the cells in the grid.
    pub fn iter_mut<'a>(&'a mut self) -> RollGrid2DMutIterator<'a, T> {
        RollGrid2DMutIterator {
            bounds_iter: self.bounds().iter(),
            grid: self,
        }
    }
}

impl<T: Copy> RollGrid2D<T> {
    /// Get a copy of the grid value.
    pub fn get_copy(&self, coord: (i32, i32)) -> Option<T> {
        let index = self.offset_index(coord)?;
        Some(self.cells[index])
    }
}

impl<T: Clone> RollGrid2D<T> {
    /// Get a clone of the grid value.
    pub fn get_clone(&self, coord: (i32, i32)) -> Option<T> {
        let index = self.offset_index(coord)?;
        Some(self.cells[index].clone())
    }
}

impl<T: Clone> Clone for RollGrid2D<T> {
    fn clone(&self) -> Self {
        Self {
            cells: self.cells.clone(),
            size: self.size,
            wrap_offset: self.wrap_offset,
            grid_offset: self.grid_offset,
        }
    }
}

/// Iterator over all cells in a [RollGrid2D].
pub struct RollGrid2DIterator<'a, T> {
    grid: &'a RollGrid2D<T>,
    bounds_iter: Bounds2DIter,
}

impl<'a, T> Iterator for RollGrid2DIterator<'a, T> {
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

/// Mutable iterator over all cells in the [RollGrid2D].
pub struct RollGrid2DMutIterator<'a, T> {
    grid: &'a mut RollGrid2D<T>,
    bounds_iter: Bounds2DIter,
}

impl<'a, T> Iterator for RollGrid2DMutIterator<'a, T> {
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

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Write more comprehensive tests.
    //       In previous versions, I missed a huge bug because the tests
    //       weren't comprehensive enough and I missed tons of edge cases.
    //       This time, think more about the edge cases.
    //       Think about overflows, repeated actions, etc.

    fn print_grid(grid: &RollGrid2D<(i32, i32)>) {
        println!("[");
        for y in grid.y_min()..grid.y_max() {
            print!("    [");
            for x in grid.x_min()..grid.x_max() {
                if let Some((cx, cy)) = grid.get_copy((x, y)) {
                    if x > grid.x_min() {
                        print!(", ");
                    }
                    print!("({cx:2}, {cy:2})");
                }
            }
            println!("]");
        }
        println!("]");
    }

    #[test]
    fn big_zst_grid() {
        let mut big_grid = RollGrid2D::new_zst(100, 100, (0, 0));
        big_grid.reposition((123, 456), |_, _, _| ());
        _ = big_grid.get((123, 456)).expect("Failed to get.");
    }

    #[test]
    fn visual_example() {
        let mut grid = RollGrid2D::new(4, 4, (0, 0), |pos: (i32, i32)| pos);
        println!("Initial grid:");
        print_grid(&grid);
        let mut iterations = 0;
        let mut changes = vec![];
        grid.reposition((1, 2), |old, new, value| {
            iterations += 1;
            changes.push((old, new));
            *value = new;
        });
        println!("Changes:");
        for (old, new) in changes {
            println!("{old:?} moved to {new:?}");
        }
        println!("Grid repositioned to (1, 2) with {iterations} iterations:");
        print_grid(&grid);
        println!("Cell at (4, 5): {:?}", grid.get_copy((4, 5)).unwrap());
        println!("Cell at (0, 0): {:?}", grid.get_copy((0, 0)));
    }

    #[test]
    fn resize_and_reposition_test() {
        struct DropCoord {
            coord: (i32, i32),
            unloaded: bool,
        }
        impl From<(i32, i32)> for DropCoord {
            fn from(value: (i32, i32)) -> Self {
                Self {
                    coord: value,
                    unloaded: false,
                }
            }
        }
        impl Drop for DropCoord {
            fn drop(&mut self) {
                // assert!(self.unloaded);
            }
        }
        fn verify_grid(grid: &RollGrid2D<DropCoord>) {
            for y in grid.y_min()..grid.y_max() {
                for x in grid.x_min()..grid.x_max() {
                    let pos = (x, y);
                    let cell = grid.get(pos).expect("Cell was None");
                    assert_eq!(pos, cell.coord);
                }
            }
        }
        for height in 1..7 {
            for width in 1..7 {
                for y in -1..6 {
                    for x in -1..6 {
                        let mut grid =
                            RollGrid2D::new(4, 4, (0, 0), |pos: (i32, i32)| DropCoord::from(pos));
                        // reposition to half point to ensure that wrapping does not cause lookup invalidation.
                        grid.reposition((2, 2), |old_pos, new_pos, cell| {
                            assert_eq!(new_pos.0 - old_pos.0, 2, "x");
                            assert_eq!(new_pos.1 - old_pos.1, 2, "y");
                            assert_eq!(old_pos, cell.coord);
                            cell.coord = new_pos;
                        });
                        grid.resize_and_reposition(
                            width,
                            height,
                            (x, y),
                            crate::cell_manager(
                                |pos| DropCoord::from(pos),
                                |pos, value| {
                                    let mut old = value;
                                    old.unloaded = true;
                                    assert_eq!(pos, old.coord);
                                },
                                |_, new_pos, value| {
                                    value.coord = new_pos;
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
