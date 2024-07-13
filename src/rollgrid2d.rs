#![allow(unused)]
use super::*;
const AREA_IS_ZERO: &'static str = "Width/Height cannot be 0";
type Coord = (i32, i32);

struct TempGrid2D<T> {
    pub cells: Box<[Option<T>]>,
    pub size: (usize, usize),
    pub offset: (i32, i32),
}

impl<T> TempGrid2D<T> {
    pub fn new(size: (usize, usize), offset: (i32, i32)) -> Self {
        Self {
            cells: (0..size.0*size.1).map(|_| None).collect(),
            size,
            offset
        }
    }

    fn offset_index(&self, (x, y): (i32, i32)) -> Option<usize> {
        let (mx, my) = self.offset;
        let width = self.size.0 as i32;
        let height = self.size.1 as i32;
        if x < mx
        || y < my
        || x >= mx + width
        || y >= my + height {
            return None;
        }
        // Adjust x and y
        let nx = x - mx;
        let ny = y - my;
        Some((ny as usize * self.size.0) + nx as usize)
    }

    pub fn get(&self, coord: (i32, i32)) -> Option<&T> {
        let index = self.offset_index(coord)?;
        if let Some(cell) = &self.cells[index] {
            Some(cell)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, coord: (i32, i32)) -> Option<&mut T> {
        let index = self.offset_index(coord)?;
        if let Some(cell) = &mut self.cells[index] {
            Some(cell)
        } else {
            None
        }
    }

    pub fn set(&mut self, coord: (i32, i32), value: T) -> Option<T> {
        let cell = self.get_mut(coord)?;
        let mut old = value;
        std::mem::swap(&mut old, cell);
        Some(old)
    }
    
    pub fn get_opt(&self, pos: (i32, i32)) -> Option<&Option<T>> {
        let index = self.offset_index(pos)?;
        Some(&self.cells[index])
    }

    pub fn get_opt_mut(&mut self, pos: (i32, i32)) -> Option<&mut Option<T>> {
        let index = self.offset_index(pos)?;
        Some(&mut self.cells[index])
    }

    pub fn set_opt(&mut self, pos: (i32, i32), value: Option<T>) -> Option<Option<T>> {
        let cell = self.get_opt_mut(pos)?;
        let mut old = value;
        std::mem::swap(&mut old, cell);
        Some(old)
    }

    pub fn take_cells(self) -> Box<[Option<T>]> {
        self.cells
    }
}

pub struct RollGrid2D<T> {
    cells: Box<[Option<T>]>,
    size: (usize, usize),
    wrap_offset: (i32, i32),
    grid_offset: (i32, i32),
}

impl<T: Default> RollGrid2D<T> {
    pub fn new_default(width: usize, height: usize, grid_offset: (i32, i32)) -> Self {
        let area = width.checked_mul(height).expect(SIZE_TOO_LARGE);
        Self {
            cells: (0..area).map(|_| Some(T::default())).collect(),
            size: (width, height),
            grid_offset: grid_offset,
            wrap_offset: (0, 0),
        }
    }
}

impl<T> RollGrid2D<T> {

    // Constructors
    /// Create a new [RollGrid2D] with all the elements set to None.
    pub fn new(width: usize, height: usize, grid_offset: (i32, i32)) -> Self {
        let area = width.checked_mul(height).expect(SIZE_TOO_LARGE);
        if area == 0 { panic!("{}", AREA_IS_ZERO); }
        #[cfg(target_pointer_width = "64")]
        if area > i32::MAX as usize { panic!("{}", SIZE_TOO_LARGE); }
        if grid_offset.0.checked_add(width as i32).is_none()
        || grid_offset.1.checked_add(height as i32).is_none() {
            panic!("{}", OFFSET_TOO_CLOSE_TO_MAX);
        }
        Self {
            cells: (0..area).map(|_| None).collect(),
            size: (width, height),
            grid_offset: grid_offset,
            wrap_offset: (0, 0),
        }
    }

    /// Create a new [RollGrid2D] using an initialize function to initialize elements.
    pub fn new_with_init<C: From<(i32, i32)>, F: FnMut(C) -> Option<T>>(
        width: usize,
        height: usize,
        grid_offset: (i32, i32),
        init: F
    ) -> Self {
        let area = width.checked_mul(height).expect(SIZE_TOO_LARGE);
        if area == 0 { panic!("{}", AREA_IS_ZERO); }
        #[cfg(target_pointer_width = "64")]
        if area > i32::MAX as usize { panic!("{}", SIZE_TOO_LARGE); }
        if grid_offset.0.checked_add(width as i32).is_none()
        || grid_offset.1.checked_add(height as i32).is_none() {
            panic!("{}", OFFSET_TOO_CLOSE_TO_MAX);
        }
        Self {
            cells: itertools::iproduct!(0..height as i32, 0..width as i32)
                .map(|(y, x)| (x + grid_offset.0, y + grid_offset.1))
                .map(C::from)
                .map(init)
                .collect(),
            size: (width, height),
            wrap_offset: (0, 0),
            grid_offset: grid_offset,
        }
    }

    /// Inflate the size by `inflate`.
    pub fn inflate_size<C, F>(&mut self, inflate: usize, manage: F)
    where
        C: From<Coord> + Into<Coord>,
        F: FnMut(CellManage<C, T>) -> Option<T> {
            let inf = inflate as i32;
            let new_offset = (self.grid_offset.0 - inf, self.grid_offset.1 - inf);
            let new_width = self.size.0 + inflate * 2;
            let new_height = self.size.1 + inflate * 2;
            self.resize_and_reposition(new_width, new_height, C::from(new_offset), manage);
    }

    /// Deflate the size by `defalte`.
    pub fn deflate_size<C, F>(&mut self, deflate: usize, manage: F)
    where
        C: From<Coord> + Into<Coord>,
        F: FnMut(CellManage<C, T>) -> Option<T> {
            let def = deflate as i32;
            let new_position = C::from((self.grid_offset.0 + def, self.grid_offset.1 + def));
            let new_width = self.size.0 - deflate * 2;
            let new_height = self.size.1 - deflate * 2;
            if new_width * new_height == 0 {
                panic!("{AREA_IS_ZERO}");
            }
            self.resize_and_reposition(new_width, new_height, new_position, manage);
    }

    /// Resize the grid, keeping it in the same position.
    pub fn resize<C, F>(&mut self, new_width: usize, new_height: usize, manage: F)
    where
        C: From<Coord> + Into<Coord>,
        F: FnMut(CellManage<C, T>) -> Option<T> {
            self.resize_and_reposition::<C, F>(new_width, new_height, C::from(self.grid_offset), manage);
    }

    // Resize
    /// Resize and reposition the grid.
    /// ```no_run
    /// grid.resize_and_reposition(3, 3, (4, 4), |action| {
    ///     match action {
    ///         CellManage::Load(pos) => {
    ///             println!("Load: ({},{})", pos.0, pos.1);
    ///             // The loaded value
    ///             Some(pos)
    ///         }
    ///         CellManage::Unload(pos, old) => {
    ///             println!("Unload: ({},{})", pos.0, pos.1);
    ///             // Return None for Unload.
    ///             None
    ///         }
    ///     }
    /// });
    /// ```
    pub fn resize_and_reposition<C, F>(
        &mut self,
        new_width: usize,
        new_height: usize,
        new_position: C,
        manage: F
    )
    where
        C: Into<Coord> + From<Coord>,
        F: FnMut(CellManage<C, T>) -> Option<T>
    {
        #![allow(unused)]
        let mut manage = manage;
        if new_width == self.size.0
        && new_height == self.size.1 {
            self.reposition(new_position, |old_pos, new_pos, old_value| {
                manage(CellManage::Unload(old_pos, old_value));
                manage(CellManage::Load(new_pos))
            });
            return;
        }
        let new_position: Coord = new_position.into();
        let area = new_width.checked_mul(new_height).expect(SIZE_TOO_LARGE);
        if area == 0 { panic!("{AREA_IS_ZERO}"); }
        #[cfg(target_pointer_width = "64")]
        if area > i32::MAX as usize { panic!("{SIZE_TOO_LARGE}"); }
        let (new_x, new_y): Coord = new_position.into();
        if new_position == self.grid_offset
        && (new_width, new_height) == self.size {
            return;
        }
        let nw = new_width as i32;
        let nh = new_height as i32;
        // Determine what needs to be unloaded
        let old_bounds: Bounds2D = self.bounds();
        let new_bounds = Bounds2D::new((new_x, new_y), (new_x + nw, new_y + nh));
        if old_bounds.intersects(new_bounds) {
            let unload_left = if old_bounds.left() < new_bounds.left() {
                let left = old_bounds.left();
                let top = new_bounds.top().max(old_bounds.top());
                let right = new_bounds.left();
                let bottom = old_bounds.bottom();
                Some(Bounds2D::new((left, top), (right, bottom)))
            } else {
                None
            };
            let unload_top = if old_bounds.top() < new_bounds.top() {
                let left = old_bounds.left();
                let top = old_bounds.top();
                let right = new_bounds.right().min(old_bounds.right());
                let bottom = new_bounds.top();
                Some(Bounds2D::new((left, top), (right, bottom)))
            } else {
                None
            };
            let unload_right = if old_bounds.right() > new_bounds.right() {
                let left = new_bounds.right();
                let top = old_bounds.top();
                let right = old_bounds.right();
                let bottom = new_bounds.bottom().min(new_bounds.bottom());
                Some(Bounds2D::new((left, top), (right, bottom)))
            } else {
                None
            };
            let unload_bottom = if old_bounds.bottom() > new_bounds.bottom() {
                let left = new_bounds.left().max(old_bounds.left());
                let top = new_bounds.bottom();
                let right = old_bounds.right();
                let bottom = old_bounds.bottom();
                Some(Bounds2D::new((left, top), (right, bottom)))
            } else {
                None
            };
            let load_left = if new_bounds.left() < old_bounds.left() {
                let left = new_bounds.left();
                let top = old_bounds.top().max(new_bounds.top());
                let right = old_bounds.left();
                let bottom = new_bounds.bottom();
                Some(Bounds2D::new((left, top), (right, bottom)))
            } else {
                None
            };
            let load_top = if new_bounds.top() < old_bounds.top() {
                let left = new_bounds.left();
                let top = new_bounds.top();
                let right = old_bounds.right().min(new_bounds.right());
                let bottom = old_bounds.top();
                Some(Bounds2D::new((left, top), (right, bottom)))
            } else {
                None
            };
            let load_right = if new_bounds.right() > old_bounds.right() {
                let left = old_bounds.right();
                let top = new_bounds.top();
                let right = new_bounds.right();
                let bottom = old_bounds.bottom().min(new_bounds.bottom());
                Some(Bounds2D::new((left, top), (right, bottom)))
            } else {
                None
            };
            let load_bottom = if new_bounds.bottom() > old_bounds.bottom() {
                let left = old_bounds.left().max(new_bounds.left());
                let top = old_bounds.bottom();
                let right = new_bounds.right();
                let bottom = new_bounds.bottom();
                Some(Bounds2D::new((left, top), (right, bottom)))
            } else {
                None
            };
            // This is the region that is kept. It is not unloaded or loaded. It is just copied.
            let keep = {
                let left = new_bounds.left().max(old_bounds.left());
                let top = new_bounds.top().max(old_bounds.top());
                let right = old_bounds.right().min(new_bounds.right());
                let bottom = old_bounds.bottom().min(new_bounds.bottom());
                Bounds2D::new((left, top), (right, bottom))
            };
            let mut temp_grid = TempGrid2D::<T>::new((new_width, new_height), new_position);
            keep.iter().for_each(|pos| {
                let self_index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                let other_index = temp_grid.offset_index(pos).expect(OUT_OF_BOUNDS);
                let cell = self.cells[self_index].take();
                temp_grid.cells[other_index] = cell;
            });
            macro_rules! unload_region {
                ($region:expr) => {
                    if let Some(region) = $region {
                        region.iter().for_each(|pos| {
                            let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                            let cell = self.cells[index].take();
                            manage(CellManage::Unload(C::from(pos), cell));
                        });
                    }
                };
            }
            macro_rules! load_region {
                ($region:expr) => {
                    if let Some(region) = $region {
                        region.iter().for_each(|pos| {
                            let index = temp_grid.offset_index(pos).expect(OUT_OF_BOUNDS);
                            let new_value = manage(CellManage::Load(C::from(pos)));
                            temp_grid.cells[index] = new_value;
                        });
                    }
                };
            }
            unload_region!(unload_left);
            unload_region!(unload_top);
            unload_region!(unload_right);
            unload_region!(unload_bottom);
            load_region!(load_left);
            load_region!(load_top);
            load_region!(load_right);
            load_region!(load_bottom);
            self.size = (new_width, new_height);
            self.grid_offset = new_position;
            self.cells = temp_grid.take_cells();
        } else { // !old_bounds.intersects(new_bounds)
            old_bounds.iter().for_each(|pos| {
                let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                let value = self.cells[index].take();
                manage(CellManage::Unload(C::from(pos), value));
            });
            let mut temp_grid = TempGrid2D::<T>::new((new_width, new_height), new_position);
            new_bounds.iter().for_each(|pos| {
                let index = temp_grid.offset_index(pos).expect(OUT_OF_BOUNDS);
                let new_value = manage(CellManage::Load(C::from(pos)));
                temp_grid.cells[index] = new_value;
            });
            self.size = (new_width, new_height);
            self.grid_offset = new_position;
            self.cells = temp_grid.take_cells();
        }
    }

    // Translation/Repositioning

    /// Translate the grid by offset amount with a reload function.
    /// Signature of the reload function is as follows:
    /// ```rust,no_run
    /// fn reload(old_position: C, new_position: C, old_value: T) -> Option<T>
    /// ```
    /// Where the return value of `reload` is the new value for that slot.
    pub fn translate<C, F>(&mut self, offset: C, reload: F)
    where
        C: Into<(i32, i32)> + From<(i32, i32)>,
        F: FnMut(C, C, Option<T>) -> Option<T> {
            let (curx, cury) = self.grid_offset;
            let (ox, oy): (i32, i32) = offset.into();
            self.reposition(C::from((curx + ox, cury + oy)), reload);
        }
    
    /// Reposition the offset of the grid and reload the slots that are changed.
    /// Signature of the reload function is as follows:
    /// ```rust,no_run
    /// fn reload(old_position: C, new_position: C, old_value: T) -> Option<T>
    /// ```
    /// Where the return value of `reload` is the new value for that slot.
    pub fn reposition<C, F>(&mut self, position: C, reload: F)
    where
        C: Into<(i32, i32)> + From<(i32, i32)>,
        F: FnMut(C, C, Option<T>) -> Option<T> {
            let (old_x, old_y) = self.grid_offset;
            let (new_x, new_y): (i32, i32) = position.into();
            let offset = (
                new_x - old_x,
                new_y - old_y
            );
            if offset == (0, 0) {
                return;
            }
            let mut reload = reload;
            let width = self.size.0 as i32;
            let height = self.size.1 as i32;
            let (offset_x, offset_y) = offset;
            self.grid_offset = (new_x, new_y);
            // Offset is within bounds, so that means that the grid will be rolled.
            // This allows for bounded reloading of the grid elements.
            // If rolling causes a section to remain on the grid, that section will not be reloaded.
            // Only the elements that are considered new will be reloaded.
            if offset_x.abs() < width && offset_y.abs() < height {
                let (roll_x, roll_y) = (
                    self.wrap_offset.0 as i32,
                    self.wrap_offset.1 as i32
                );
                let (wrapped_offset_x, wrapped_offset_y) = (
                    offset_x.rem_euclid(width),
                    offset_y.rem_euclid(height)
                );
                // Update the roll so that we reduce reloading.
                // Without using the roll functionality, this function would demand to reload
                // every single cell, even if it only needed to reload 8 out of 64 cells.
                let new_rolled_x = (roll_x + wrapped_offset_x).rem_euclid(width);
                let new_rolled_y = (roll_y + wrapped_offset_y).rem_euclid(height);
                self.wrap_offset = (new_rolled_x, new_rolled_y);
                let right = new_x + width;
                let bottom = new_y + height;
                // Calculate ranges
                // Combining new_x_range and new_y_range gets the corner.
                // The partition on either the left or right side
                let new_x_range = if offset_x >= 0 {
                    (right - offset_x)..right
                } else {
                    new_x..new_x-offset_x
                };
                let new_x_range_y_range = if offset_y >= 0 {
                    new_y..(bottom - offset_y)
                } else {
                    new_y-offset_y..bottom
                };
                // The partition on either the top or the bottom.
                let new_y_range = if offset_y >= 0 {
                    (bottom - offset_y)..bottom
                } else {
                    new_y..new_y-offset_y
                };
                let new_y_range_x_range = if offset_x >= 0 {
                    new_x..(right - offset_x)
                } else {
                    new_x-offset_x..right
                };
                // The left/right partition
                for y in new_x_range_y_range.clone() {
                    for (xi, x) in new_x_range.clone().enumerate() {
                        let prior_x = if offset_x >= 0 {
                            old_x + xi as i32
                        } else {
                            old_x + width + offset_x + xi as i32
                        };
                        let prior_y = y;
                        let index = self.offset_index((x, y)).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        let new_value = reload(C::from((prior_x, prior_y)), C::from((x, y)), old_value);
                        self.cells[index] = new_value;
                    }
                }
                // The top/bottom partition
                for (iy, y) in new_y_range.clone().enumerate() {
                    for x in new_y_range_x_range.clone() {
                        let prior_x = x;
                        let prior_y = if offset_y >= 0 {
                            old_y + iy as i32
                        } else {
                            old_y + height + offset_y + iy as i32
                        };
                        let index = self.offset_index((x, y)).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        let new_value = reload(C::from((prior_x, prior_y)), C::from((x, y)), old_value);
                        self.cells[index] = new_value;
                    }
                }
                // The corner partition
                for (iy, y) in new_y_range.enumerate() {
                    for (ix, x) in new_x_range.clone().enumerate() {
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
                        let index = self.offset_index((x, y)).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        let new_value = reload(C::from((prior_x, prior_y)), C::from((x, y)), old_value);
                        self.cells[index] = new_value;
                    }
                }
            } else {
                // Reload everything
                for (yi, y) in (new_y..new_y + height).enumerate() {
                    for (xi, x) in (new_x..new_x + width).enumerate() {
                        let prior_x = old_x + xi as i32;
                        let prior_y = old_y + yi as i32;
                        let index = self.offset_index((x, y)).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        let new_value = reload(C::from((prior_x, prior_y)), C::from((x, y)), old_value);
                        self.cells[index] = new_value;
                    }
                }
            }
        }

    pub fn relative_offset<C: Into<(i32, i32)> + From<(i32, i32)>>(&self, coord: C) -> C {
        let (x, y): (i32, i32) = coord.into();
        C::from((
            x - self.grid_offset.0,
            y - self.grid_offset.1
        ))
    }

    // Utility function(s)
    fn offset_index(&self, (x, y): (i32, i32)) -> Option<usize> {
        let (mx, my) = self.grid_offset;
        let width = self.size.0 as i32;
        let height = self.size.1 as i32;
        if x >= mx + width
        || y >= my + height
        || x < mx
        || y < my {
            return None;
        }
        // Adjust x and y
        let nx = x - mx;
        let ny = y - my;
        // Wrap x and y
        let (wrap_x, wrap_y) = (self.wrap_offset.0 as i32, self.wrap_offset.1 as i32);
        let wx = (nx + wrap_x).rem_euclid(width);
        let wy = (ny + wrap_y).rem_euclid(height);
        Some((wy as usize * self.size.0) + wx as usize)
    }

    pub fn get_opt<C: Into<Coord>>(&self, coord: C) -> Option<&Option<T>> {
        let index = self.offset_index(coord.into())?;
        Some(&self.cells[index])
    }

    pub fn get_opt_mut<C: Into<Coord>>(&mut self, coord: C) -> Option<&mut Option<T>> {
        let index = self.offset_index(coord.into())?;
        Some(&mut self.cells[index])
    }

    pub fn set_opt<C: Into<Coord>>(&mut self, coord: C, value: Option<T>) -> Option<Option<T>> {
        let cell = self.get_opt_mut(coord.into())?;
        let mut old = value;
        std::mem::swap(&mut old, cell);
        Some(old)
    }

    pub fn get<C: Into<Coord>>(&self, coord: C) -> Option<&T> {
        let index = self.offset_index(coord.into())?;
        if let Some(cell) = &self.cells[index] {
            Some(cell)
        } else {
            None
        }
    }

    pub fn get_mut<C: Into<Coord>>(&mut self, coord: C) -> Option<&mut T> {
        let index = self.offset_index(coord.into())?;
        if let Some(cell) = &mut self.cells[index] {
            Some(cell)
        } else {
            None
        }
    }

    pub fn set<C: Into<Coord>>(&mut self, coord: C, value: T) -> Option<T> {
        let cell = self.get_mut(coord)?;
        let mut old = value;
        std::mem::swap(&mut old, cell);
        Some(old)
    }

    // Pleasantries
    pub fn size(&self) -> (usize, usize) {
        self.size
    }

    pub fn width(&self) -> usize {
        self.size.0
    }

    pub fn height(&self) -> usize {
        self.size.1
    }

    pub fn wrap_offset(&self) -> (i32, i32) {
        self.wrap_offset
    }

    pub fn offset(&self) -> (i32, i32) {
        self.grid_offset
    }

    pub fn left(&self) -> i32 {
        self.grid_offset.0
    }

    pub fn right(&self) -> i32 {
        self.grid_offset.0 + self.size.0 as i32
    }

    pub fn top(&self) -> i32 {
        self.grid_offset.1
    }

    pub fn bottom(&self) -> i32 {
        self.grid_offset.1 + self.size.1 as i32
    }

    pub fn bounds(&self) -> Bounds2D {
        Bounds2D {
            min: (self.left(), self.top()),
            max: (self.right(), self.bottom())
        }
    }

    /// This is equivalent to the area (width * height).
    pub fn len(&self) -> usize {
        self.size.0 * self.size.1
    }

    pub fn iter<'a>(&'a self) -> RollGrid2DIterator<'a, T> {
        RollGrid2DIterator {
            bounds_iter: self.bounds().iter(),
            grid: self,
        }
    }

    pub fn iter_mut<'a>(&'a mut self) -> RollGrid2DMutIterator<'a, T> {
        RollGrid2DMutIterator {
            bounds_iter: self.bounds().iter(),
            grid: self,
        }
    }

}


impl<T: Copy> RollGrid2D<T> {
    pub fn get_copy<C: Into<Coord>>(&self, coord: C) -> Option<T> {
        let coord: Coord = coord.into();
        let index = self.offset_index(coord)?;
        self.cells[index]
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bounds2D {
    /// Inclusive minimum.
    pub min: (i32, i32),
    /// Exclusive maximum.
    pub max: (i32, i32),
}

impl Bounds2D {
    pub fn new<C: Into<(i32, i32)>>(min: C, max: C) -> Self {
        Self {
            min: min.into(),
            max: max.into()
        }
    }

    pub fn from_bounds<C: Into<(i32, i32)>>(a: C, b: C) -> Self {
        let a: (i32, i32) = a.into();
        let b: (i32, i32) = b.into();
        let (ax, ay) = a;
        let (bx, by) = b;
        let min = (ax.min(bx), ay.min(by));
        let max = (ax.max(bx), ay.max(by));
        Self {
            min,
            max
        }
    }

    pub fn width(&self) -> u32 {
        (self.max.0 as i64 - self.min.0 as i64) as u32
    }

    pub fn height(&self) -> u32 {
        (self.max.1 as i64 - self.min.1 as i64) as u32
    }

    pub fn area(&self) -> i64 {
        self.width() as i64 * self.height() as i64
    }

    pub fn left(&self) -> i32 {
        self.min.0
    }

    pub fn top(&self) -> i32 {
        self.min.1
    }

    pub fn right(&self) -> i32 {
        self.max.0
    }

    pub fn bottom(&self) -> i32 {
        self.max.1
    }

    // intersects would need to copy self and other anyway, so
    // just accept copied values rather than references.
    pub fn intersects(self, other: Bounds2D) -> bool {
        let ((ax_min, ay_min), (ax_max, ay_max)) = (self.min, self.max);
        let ((bx_min, by_min), (bx_max, by_max)) = (other.min, other.max);
        ax_min < bx_max
        && bx_min < ax_max
        && ay_min < by_max
        && by_min < ay_max
    }

    pub fn contains<P: Into<(i32, i32)>>(self, point: P) -> bool {
        let point: (i32, i32) = point.into();
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
            self.current.1 - self.bounds.min.1
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

pub struct RollGrid2DIterator<'a, T> {
    grid: &'a RollGrid2D<T>,
    bounds_iter: Bounds2DIter,
}

impl<'a, T> Iterator for RollGrid2DIterator<'a, T> {
    type Item = ((i32, i32), Option<&'a T>);

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.bounds_iter.size_hint()
    }

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.bounds_iter.next()?;
        let index = self.grid.offset_index(next)?;
        if let Some(cell) = &self.grid.cells[index] {
            // I know this looks wonky, but I promise this is correct.
            Some((next, Some(cell)))
        } else {
            Some((next, None))
        }
    }
}

pub struct RollGrid2DMutIterator<'a, T> {
    grid: &'a mut RollGrid2D<T>,
    bounds_iter: Bounds2DIter,
}

impl<'a, T> Iterator for RollGrid2DMutIterator<'a, T> {
    type Item = ((i32, i32), Option<&'a mut T>);

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.bounds_iter.size_hint()
    }

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.bounds_iter.next()?;
        let index = self.grid.offset_index(next)?;
        unsafe {
            let cells_ptr = self.grid.cells.as_mut_ptr();
            let cell_ptr = cells_ptr.add(index);
            if let Some(cell) = &mut *cell_ptr {
                Some((next, Some(cell)))
            } else {
                Some((next, None))
            }
        }
    }
}