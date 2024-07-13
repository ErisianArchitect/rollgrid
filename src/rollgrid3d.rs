#![allow(unused)]
use crate::{CellManage, OFFSET_TOO_CLOSE_TO_MAX, OUT_OF_BOUNDS, SIZE_TOO_LARGE};
const VOLUME_IS_ZERO: &'static str = "Width/Height/Depth cannot be 0";

type Coord = (i32, i32, i32);

pub struct RollGrid3D<T> {
    cells: Box<[Option<T>]>,
    size: (usize, usize, usize),
    wrap_offset: (i32, i32, i32),
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

    pub fn new_with_init<C: From<Coord>, F: FnMut(C) -> Option<T>>(
        width: usize,
        height: usize,
        depth: usize,
        grid_offset: (i32, i32, i32),
        init: F
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
            cells: itertools::iproduct!(0..height as i32, 0..depth as i32, 0..width as i32)
                .map(|(y, z, x)| C::from((
                    x + grid_offset.0,
                    y + grid_offset.1,
                    z + grid_offset.2
                )))
                .map(init)
                .collect(),
            size: (width, height, depth),
            wrap_offset: (0, 0, 0),
            grid_offset
        }
    }

    /// Inflate the size by `inflate`, keeping the bounds centered.
    /// If the size is `(2, 2, 2)` with an offset of `(1, 1, 1)`, and you want to inflate by `(1, 1, 1)`.
    /// The result of that operation would have a size of `(4, 4, 4)` and an offset of `(0, 0, 0)`.
    pub fn inflate_size<C, F>(&mut self, inflate: (usize, usize, usize), manage: F)
    where
        C: From<Coord> + Into<Coord>,
        F: FnMut(CellManage<C, T>) -> Option<T> {
            const INFLATE_TOO_LARGE: &'static str = "Cannot inflate more than i32::MAX";
            const INFLATE_OVERFLOW: &'static str = "Inflate operation results in integer overflow";
            if inflate.0 > i32::MAX as usize { panic!("{INFLATE_TOO_LARGE}"); }
            if inflate.1 > i32::MAX as usize { panic!("{INFLATE_TOO_LARGE}"); }
            if inflate.2 > i32::MAX as usize { panic!("{INFLATE_TOO_LARGE}"); }
            // let inf = inflate as i32;
            let position = C::from((
                self.grid_offset.0 - inflate.0 as i32,
                self.grid_offset.1 - inflate.1 as i32,
                self.grid_offset.2 - inflate.2 as i32,
            ));
            let width = self.size.0.checked_add(inflate.0.checked_mul(2).expect(INFLATE_OVERFLOW)).expect(INFLATE_OVERFLOW);
            let height = self.size.1.checked_add(inflate.1.checked_mul(2).expect(INFLATE_OVERFLOW)).expect(INFLATE_OVERFLOW);
            let depth = self.size.2.checked_add(inflate.2.checked_mul(2).expect(INFLATE_OVERFLOW)).expect(INFLATE_OVERFLOW);
            self.resize_and_reposition(width, height, depth, position, manage);
        }
    
    /// Deflate the size by `deflate`, keeping the bounds centered.
    /// If the size is `(4, 4, 4)` with an offset of `(0, 0, 0)`, and you want to deflate by `(1, 1, 1)`.
    /// The result of that operation would have a size of `(2, 2, 2)` and an offset of `(1, 1, 1)`.
    pub fn deflate_size<C, F>(&mut self, deflate: (usize, usize, usize), manage: F)
    where
        C: From<Coord> + Into<Coord>,
        F: FnMut(CellManage<C, T>) -> Option<T> {
            const DEFLATE_PAST_I32_MAX: &'static str = "Cannot deflate more than i32::MAX";
            const DEFLATE_OVERFLOW: &'static str = "Deflate operation results in integer overflow";
            if deflate.0 > i32::MAX as usize { panic!("{DEFLATE_PAST_I32_MAX}"); }
            if deflate.1 > i32::MAX as usize { panic!("{DEFLATE_PAST_I32_MAX}"); }
            if deflate.2 > i32::MAX as usize { panic!("{DEFLATE_PAST_I32_MAX}"); }
            let position = C::from((
                self.grid_offset.0 + deflate.0 as i32,
                self.grid_offset.1 + deflate.1 as i32,
                self.grid_offset.2 + deflate.2 as i32,
            ));
            let width = self.size.0.checked_sub(deflate.0.checked_mul(2).expect(DEFLATE_OVERFLOW)).expect(DEFLATE_OVERFLOW);
            let height = self.size.1.checked_sub(deflate.1.checked_mul(2).expect(DEFLATE_OVERFLOW)).expect(DEFLATE_OVERFLOW);
            let depth = self.size.2.checked_sub(deflate.2.checked_mul(2).expect(DEFLATE_OVERFLOW)).expect(DEFLATE_OVERFLOW);
            let volume = width.checked_mul(height).expect(SIZE_TOO_LARGE).checked_mul(depth).expect(SIZE_TOO_LARGE);
            if volume == 0 {
                panic!("{VOLUME_IS_ZERO}");
            }
            self.resize_and_reposition(width, height, depth, position, manage);
        }
    
    /// Resize the grid, keeping the offset in the same place.
    pub fn resize<C, F>(&mut self, width: usize, height: usize, depth: usize, manage: F)
    where
        C: From<Coord> + Into<Coord>,
        F: FnMut(CellManage<C, T>) -> Option<T> {
            self.resize_and_reposition(width, height, depth, C::from(self.grid_offset), manage);
        }

    /// Resize and reposition the grid.
    /// ```no_run
    /// grid.resize_and_reposition(3, 3, 3, (4, 4, 4), |action| {
    ///     match action {
    ///         CellManage::Load(pos) => {
    ///             println!("Load: ({},{},{})", pos.0, pos.1, pos.2);
    ///             // The loaded value
    ///             Some(pos)
    ///         }
    ///         CellManage::Unload(pos, old) => {
    ///             println!("Unload: ({},{},{})", pos.0, pos.1, pos.2);
    ///             // Return None for Unload.
    ///             None
    ///         }
    ///     }
    /// });
    /// ```
    pub fn resize_and_reposition<C, F>(
        &mut self,
        width: usize,
        height: usize,
        depth: usize,
        position: C,
        manage: F,
    )
    where
        C: Into<Coord> + From<Coord>,
        F: FnMut(CellManage<C, T>) -> Option<T> {
            #![allow(unused)]
            let mut manage = manage;
            if width == self.size.0
            && height == self.size.1
            && depth == self.size.2 {
                self.reposition(position, |old_pos, new_pos, old_value| {
                    manage(CellManage::Unload(old_pos, old_value));
                    manage(CellManage::Load(new_pos))
                });
                return;
            }
            let new_position: Coord = position.into();
            if new_position == self.grid_offset
            && (width, height, depth) == self.size {
                return;
            }
            let volume = width.checked_mul(height).expect(SIZE_TOO_LARGE).checked_mul(depth).expect(SIZE_TOO_LARGE);
            if volume == 0 { panic!("{VOLUME_IS_ZERO}"); };
            #[cfg(target_pointer_width = "64")]
            if volume > i32::MAX as usize { panic!("{SIZE_TOO_LARGE}"); }
            let (new_x, new_y, new_z) = new_position;
            let new_width = width as i32;
            let new_height = height as i32;
            let new_depth = depth as i32;
            let old_bounds = self.bounds();
            let new_bounds = Bounds3D::new(
                (new_x, new_y, new_z),
                (new_x + new_width, new_y + new_height, new_z + new_depth)
            );
            if old_bounds.intersects(new_bounds) {
                macro_rules! unload_bounds {
                    ($cond:expr => xmin = $xmin:expr; ymin = $ymin:expr; zmin = $zmin:expr; xmax = $xmax:expr; ymax = $ymax:expr; zmax = $zmax:expr;) => {
                        if $cond {
                            Bounds3D::new(
                                ($xmin, $ymin, $zmin),
                                ($xmax, $ymax, $zmax)
                            ).iter().for_each(|pos| {
                                let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                                manage(CellManage::Unload(C::from(pos), self.cells[index].take()));
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
                let mut temp_grid = TempGrid3D::new_with_init((width, height, depth), new_position, |pos| {
                    if old_bounds.contains(pos) {
                        let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                        self.cells[index].take()
                    } else {
                        manage(CellManage::Load(C::from(pos)))
                    }
                });
                self.size = temp_grid.size;
                self.grid_offset = temp_grid.offset;
                self.cells = temp_grid.cells;
                self.wrap_offset = (0, 0, 0);
            } else { // !old_bounds.intersects(new_bounds)
                old_bounds.iter().for_each(|pos| {
                    let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                    manage(CellManage::Unload(C::from(pos), self.cells[index].take()));
                });
                let mut temp_grid = TempGrid3D::new_with_init((width, height, depth), new_position, |pos| {
                    manage(CellManage::Load(C::from(pos)))
                });
                self.size = temp_grid.size;
                self.grid_offset = temp_grid.offset;
                self.cells = temp_grid.cells;
                self.wrap_offset = (0, 0, 0);
            }
        }
    
    pub fn translate<C, F>(&mut self, offset: C, reload: F)
    where
        C: Into<Coord> + From<Coord>,
        F: FnMut(C, C, Option<T>) -> Option<T> {
            let (off_x, off_y, off_z): Coord = offset.into();
            let new_pos = C::from((
                self.grid_offset.0 + off_x,
                self.grid_offset.1 + off_y,
                self.grid_offset.2 + off_z,
            ));
            self.reposition(new_pos, reload);
        }

    pub fn reposition<C, F>(&mut self, position: C, reload: F)
    where
        C: Into<Coord> + From<Coord>,
        F: FnMut(C, C, Option<T>) -> Option<T> {
            let (old_x, old_y, old_z) = self.grid_offset;
            let (new_x, new_y, new_z): (i32, i32, i32) = position.into();
            let offset = (
                new_x - old_x,
                new_y - old_y,
                new_z - old_z
            );
            if offset == (0, 0, 0) {
                return;
            }
            let mut reload = reload;
            let width = self.size.0 as i32;
            let height = self.size.1 as i32;
            let depth = self.size.2 as i32;
            let (offset_x, offset_y, offset_z) = offset;
            let old_bounds = self.bounds();
            let new_bounds = Bounds3D::new(
                (new_x, new_y, new_z),
                (new_x + width, new_y + height, new_z + depth)
            );
            // A cool trick to test whether the translation moves out of bounds.
            if offset_x.abs() < width
            && offset_y.abs() < height
            && offset_z.abs() < depth { // translation in bounds, the hard part.
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
                let (half_region, quarter_region, eighth_region) = if new_bounds.x_min() < old_bounds.x_min() {
                    // -X
                    let half_region = {
                        let x_min = new_bounds.x_min();
                        let y_min = new_bounds.y_min();
                        let z_min = new_bounds.z_min();
                        let x_max = old_bounds.x_min();
                        let y_max = new_bounds.y_max();
                        let z_max = new_bounds.z_max();
                        Bounds3D::new(
                            (x_min, y_min, z_min),
                            (x_max, y_max, z_max)
                        )
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
                            Bounds3D::new(
                                (x_min, y_min, z_min),
                                (x_max, y_max, z_max)
                            )
                        };
                        let eighth_region = if new_bounds.y_min() < old_bounds.y_min() {
                            // eighth: -X -Y -Z
                            let x_min = old_bounds.x_min();
                            let y_min = new_bounds.y_min();
                            let z_min = old_bounds.z_min();
                            let x_max = new_bounds.x_max();
                            let y_max = old_bounds.y_min();
                            let z_max = new_bounds.z_max();
                            Some(Bounds3D::new(
                                (x_min, y_min, z_min),
                                (x_max, y_max, z_max)
                            ))
                        } else if new_bounds.y_max() > old_bounds.y_max() {
                            // eighth: -X +Y -Z
                            let x_min = old_bounds.x_min();
                            let y_min = old_bounds.y_max();
                            let z_min = old_bounds.z_min();
                            let x_max = new_bounds.x_max();
                            let y_max = new_bounds.y_max();
                            let z_max = new_bounds.z_max();
                            Some(Bounds3D::new(
                                (x_min, y_min, z_min),
                                (x_max, y_max, z_max)
                            ))
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
                            Bounds3D::new(
                                (x_min, y_min, z_min),
                                (x_max, y_max, z_max)
                            )
                        };
                        let eighth_region = if new_bounds.y_min() < old_bounds.y_min() {
                            // eighth: -X -Y +Z
                            let x_min = old_bounds.x_min();
                            let y_min = new_bounds.y_min();
                            let z_min = new_bounds.z_min();
                            let x_max = new_bounds.x_max();
                            let y_max = old_bounds.y_min();
                            let z_max = old_bounds.z_max();
                            Some(Bounds3D::new(
                                (x_min, y_min, z_min),
                                (x_max, y_max, z_max)
                            ))
                        } else if new_bounds.y_max() > old_bounds.y_max() {
                            // eighth: -X +Y +Z
                            let x_min = old_bounds.x_min();
                            let y_min = old_bounds.y_max();
                            let z_min = new_bounds.z_min();
                            let x_max = new_bounds.x_max();
                            let y_max = new_bounds.y_max();
                            let z_max = old_bounds.z_max();
                            Some(Bounds3D::new(
                                (x_min, y_min, z_min),
                                (x_max, y_max, z_max)
                            ))
                        } else {
                            // eighth: -X =Y +Z
                            None
                        };
                        (Some(quarter_region), eighth_region)
                    } else { // z is same, x is less
                        // -X =Z
                        let quarter_region = if new_bounds.y_min() < old_bounds.y_min() {
                            // quarter: -X -Y =Z
                            let x_min = old_bounds.x_min();
                            let y_min = new_bounds.y_min();
                            let z_min = new_bounds.z_min();
                            let x_max = new_bounds.x_max();
                            let y_max = old_bounds.y_min();
                            let z_max = new_bounds.z_max();
                            Some(Bounds3D::new(
                                (x_min, y_min, z_min),
                                (x_max, y_max, z_max)
                            ))
                        } else if new_bounds.y_max() > old_bounds.y_max() {
                            // quarter: -X +Y =Z
                            let x_min = old_bounds.x_min();
                            let y_min = old_bounds.y_max();
                            let z_min = new_bounds.z_min();
                            let x_max = new_bounds.x_max();
                            let y_max = new_bounds.y_max();
                            let z_max = new_bounds.z_max();
                            Some(Bounds3D::new(
                                (x_min, y_min, z_min),
                                (x_max, y_max, z_max)
                            ))
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
                        Bounds3D::new(
                            (x_min, y_min, z_min),
                            (x_max, y_max, z_max)
                        )
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
                            Bounds3D::new(
                                (x_min, y_min, z_min),
                                (x_max, y_max, z_max)
                            )
                        };
                        let eighth_region = if new_bounds.y_min() < old_bounds.y_min() {
                            // eighth: +X -Y -Z
                            let x_min = new_bounds.x_min();
                            let y_min = new_bounds.y_min();
                            let z_min = old_bounds.z_min();
                            let x_max = old_bounds.x_max();
                            let y_max = old_bounds.y_min();
                            let z_max = new_bounds.z_max();
                            Some(Bounds3D::new(
                                (x_min, y_min, z_min),
                                (x_max, y_max, z_max)
                            ))
                        }else if new_bounds.y_max() > old_bounds.y_max() {
                            // eighth: +X +Y -Z
                            let x_min = new_bounds.x_min();
                            let y_min = old_bounds.y_max();
                            let z_min = old_bounds.z_min();
                            let x_max = old_bounds.x_max();
                            let y_max = new_bounds.y_max();
                            let z_max = new_bounds.z_max();
                            Some(Bounds3D::new(
                                (x_min, y_min, z_min),
                                (x_max, y_max, z_max)
                            ))
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
                            Bounds3D::new(
                                (x_min, y_min, z_min),
                                (x_max, y_max, z_max)
                            )
                        };
                        let eighth_region = if new_bounds.y_min() < old_bounds.y_min() {
                            // eighth: +X -Y +Z
                            let x_min = new_bounds.x_min();
                            let y_min = new_bounds.y_min();
                            let z_min = new_bounds.z_min();
                            let x_max = old_bounds.x_max();
                            let y_max = old_bounds.y_min();
                            let z_max = old_bounds.z_max();
                            Some(Bounds3D::new(
                                (x_min, y_min, z_min),
                                (x_max, y_max, z_max)
                            ))
                        } else if new_bounds.y_max() > old_bounds.y_max() {
                            // eighth: +X +Y +Z
                            let x_min = new_bounds.x_min();
                            let y_min = old_bounds.y_max();
                            let z_min = new_bounds.z_min();
                            let x_max = old_bounds.x_max();
                            let y_max = new_bounds.y_max();
                            let z_max = old_bounds.z_max();
                            Some(Bounds3D::new(
                                (x_min, y_min, z_min),
                                (x_max, y_max, z_max)
                            ))
                        } else {
                            None
                        };
                        (Some(quarter_region), eighth_region)
                    } else { // z is equal, x is greater
                        // +X =Z
                        let quarter_region = if new_bounds.y_min() < old_bounds.y_min() {
                            // quarter: +X -Y =Z
                            let x_min = new_bounds.x_min();
                            let y_min = new_bounds.y_min();
                            let z_min = new_bounds.z_min();
                            let x_max = old_bounds.x_max();
                            let y_max = old_bounds.y_min();
                            let z_max = new_bounds.z_max();
                            Some(Bounds3D::new(
                                (x_min, y_min, z_min),
                                (x_max, y_max, z_max)
                            ))
                        } else if new_bounds.y_max() > old_bounds.y_max() {
                            // quarter: +X +Y =Z
                            let x_min = new_bounds.x_min();
                            let y_min = old_bounds.y_max();
                            let z_min = new_bounds.z_min();
                            let x_max = old_bounds.x_max();
                            let y_max = new_bounds.y_max();
                            let z_max = new_bounds.z_max();
                            Some(Bounds3D::new(
                                (x_min, y_min, z_min),
                                (x_max, y_max, z_max)
                            ))
                        } else {
                            // quarter: +X =Y =Z
                            None
                        };
                        (quarter_region, None)
                    };
                    (half_region, quarter_region, eighth_region)
                } else { // x is equal
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
                                Bounds3D::new(
                                    (x_min, y_min, z_min),
                                    (x_max, y_max, z_max)
                                )
                            };
                            let quarter_region = {
                                let x_min = new_bounds.x_min();
                                let y_min = new_bounds.y_min();
                                let z_min = old_bounds.z_min();
                                let x_max = new_bounds.x_max();
                                let y_max = old_bounds.y_min();
                                let z_max = new_bounds.z_max();
                                Bounds3D::new(
                                    (x_min, y_min, z_min),
                                    (x_max, y_max, z_max)
                                )
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
                                Bounds3D::new(
                                    (x_min, y_min, z_min),
                                    (x_max, y_max, z_max)
                                )
                            };
                            let quarter_region = {
                                let x_min = new_bounds.x_min();
                                let y_min = old_bounds.y_max();
                                let z_min = old_bounds.z_min();
                                let x_max = new_bounds.x_max();
                                let y_max = new_bounds.y_max();
                                let z_max = new_bounds.z_max();
                                Bounds3D::new(
                                    (x_min, y_min, z_min),
                                    (x_max, y_max, z_max)
                                )
                            };
                            (half_region, Some(quarter_region))
                        } else { // x is equal, y is equal, z is less
                            // =X =Y -Z
                            // create only half_region
                            let x_min = new_bounds.x_min();
                            let y_min = new_bounds.y_min();
                            let z_min = new_bounds.z_min();
                            let x_max = new_bounds.x_max();
                            let y_max = new_bounds.y_max();
                            let z_max = old_bounds.z_min();
                            let half_region = Bounds3D::new(
                                (x_min, y_min, z_min),
                                (x_max, y_max, z_max)
                            );
                            (half_region, None)
                        }
                    } else if new_bounds.z_max() > old_bounds.z_max() { // (half, quarter) = if
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
                                Bounds3D::new(
                                    (x_min, y_min, z_min),
                                    (x_max, y_max, z_max)
                                )
                            };
                            let quarter_region = {
                                let x_min = new_bounds.x_min();
                                let y_min = new_bounds.y_min();
                                let z_min = new_bounds.z_min();
                                let x_max = new_bounds.x_max();
                                let y_max = old_bounds.y_min();
                                let z_max = old_bounds.z_max();
                                Bounds3D::new(
                                    (x_min, y_min, z_min),
                                    (x_max, y_max, z_max)
                                )
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
                                Bounds3D::new(
                                    (x_min, y_min, z_min),
                                    (x_max, y_max, z_max)
                                )
                            };
                            let quarter_region = {
                                let x_min = new_bounds.x_min();
                                let y_min = old_bounds.y_max();
                                let z_min = new_bounds.z_min();
                                let x_max = new_bounds.x_max();
                                let y_max = new_bounds.y_max();
                                let z_max = old_bounds.z_max();
                                Bounds3D::new(
                                    (x_min, y_min, z_min),
                                    (x_max, y_max, z_max)
                                )
                            };
                            (half_region, Some(quarter_region))
                        } else { // x is equal, y is equal, z is greater
                            // =X =Y +Z
                            // (half, Option<quarter>) = if; return (half, quarter)
                            // no quarter_region
                            let x_min = new_bounds.x_min();
                            let y_min = new_bounds.y_min();
                            let z_min = old_bounds.z_max();
                            let x_max = new_bounds.x_max();
                            let y_max = new_bounds.y_max();
                            let z_max = new_bounds.z_max();
                            let half_region = Bounds3D::new(
                                (x_min, y_min, z_min),
                                (x_max, y_max, z_max)
                            );
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
                            Bounds3D::new(
                                (x_min, y_min, z_min),
                                (x_max, y_max, z_max)
                            )
                        } else if new_bounds.y_max() > old_bounds.y_max() {
                            // =X +Y =Z
                            let x_min = new_bounds.x_min();
                            let y_min = old_bounds.y_max();
                            let z_min = new_bounds.z_min();
                            let x_max = new_bounds.x_max();
                            let y_max = new_bounds.y_max();
                            let z_max = new_bounds.z_max();
                            Bounds3D::new(
                                (x_min, y_min, z_min),
                                (x_max, y_max, z_max)
                            )
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
                    self.wrap_offset.0,
                    self.wrap_offset.1,
                    self.wrap_offset.2
                );
                let (wrapped_offset_x, wrapped_offset_y, wrapped_offset_z) = (
                    offset_x.rem_euclid(width),
                    offset_y.rem_euclid(height),
                    offset_z.rem_euclid(depth)
                );
                let new_wrap_x = (wrap_x + wrapped_offset_x).rem_euclid(width);
                let new_wrap_y = (wrap_y + wrapped_offset_y).rem_euclid(height);
                let new_wrap_z = (wrap_z + wrapped_offset_z).rem_euclid(depth);
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
                    offset: self.grid_offset,
                    size: (width, height, depth)
                };
                self.wrap_offset = (new_wrap_x, new_wrap_y, new_wrap_z);
                self.grid_offset = (new_x, new_y, new_z);
                // iterate regions and reload cells
                half_region.iter().for_each(|pos| {
                    let old_pos = fix.wrap(pos);
                    let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                    self.cells[index] = reload(C::from(old_pos), C::from(pos), self.cells[index].take());
                });
                if let Some(quarter) = quarter_region {
                    quarter.iter().for_each(|pos| {
                        let old_pos = fix.wrap(pos);
                        let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                        self.cells[index] = reload(C::from(old_pos), C::from(pos), self.cells[index].take());
                    });
                }
                if let Some(eighth) = eighth_region {
                    eighth.iter().for_each(|pos| {
                        let old_pos = fix.wrap(pos);
                        let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                        self.cells[index] = reload(C::from(old_pos), C::from(pos), self.cells[index].take());
                    });
                }
                // Now that we have the regions, we can iterate over them to reload cells.
            } else { // translation out of bounds, reload everything
                self.grid_offset = (new_x, new_y, new_z);
                for (yi, y) in (new_y..new_y + height).enumerate() {
                    for (zi, z) in (new_z..new_z + depth).enumerate() {
                        for (xi, x) in (new_x..new_x + width).enumerate() {
                            let prior_x = old_x + xi as i32;
                            let prior_y = old_y + yi as i32;
                            let prior_z = old_z + zi as i32;
                            let index = self.offset_index((x, y, z)).expect(OUT_OF_BOUNDS);
                            self.cells[index] = reload(
                                C::from((prior_x, prior_y, prior_z)),
                                C::from((x, y, z)),
                                self.cells[index].take()
                            );
                        }
                    }
                }
            }
        }

    pub fn relative_offset<C: Into<Coord> + From<Coord> + Copy>(&self, coord: C) -> C {
        let (x, y, z): (i32, i32, i32) = coord.into();
        C::from((
            x - self.grid_offset.0,
            y - self.grid_offset.1,
            z - self.grid_offset.2
        ))
    }

    fn offset_index(&self, (x, y, z): (i32, i32, i32)) -> Option<usize> {
        let (mx, my, mz) = self.grid_offset;
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
        // Adjust x, y, and z
        let nx = x - mx;
        let ny = y - my;
        let nz = z - mz;
        // Wrap x, y, and z
        let (wx, wy, wz) = (
            self.wrap_offset.0 as i32,
            self.wrap_offset.1 as i32,
            self.wrap_offset.2 as i32,
        );
        let wx = (nx + wx).rem_euclid(width);
        let wy = (ny + wy).rem_euclid(height);
        let wz = (nz + wz).rem_euclid(depth);
        let plane = (self.size.0 * self.size.2);
        Some(wy as usize * plane + wz as usize * self.size.0 + wx as usize)
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

    pub fn size(&self) -> (usize, usize, usize) {
        self.size
    }

    /// The size along the X axis.
    pub fn width(&self) -> usize {
        self.size.0
    }

    /// The size along the Y axis.
    pub fn height(&self) -> usize {
        self.size.1
    }

    /// The size along the Z axis.
    pub fn depth(&self) -> usize {
        self.size.2
    }

    pub fn offset(&self) -> (i32, i32, i32) {
        self.grid_offset
    }

    pub fn x_min(&self) -> i32 {
        self.grid_offset.0
    }

    pub fn y_min(&self) -> i32 {
        self.grid_offset.1
    }

    pub fn z_min(&self) -> i32 {
        self.grid_offset.2
    }

    pub fn x_max(&self) -> i32 {
        self.grid_offset.0 + self.size.0 as i32
    }

    pub fn y_max(&self) -> i32 {
        self.grid_offset.1 + self.size.1 as i32
    }

    pub fn z_max(&self) -> i32 {
        self.grid_offset.2 + self.size.2 as i32
    }

    pub fn bounds(&self) -> Bounds3D {
        Bounds3D {
            min: (self.x_min(), self.y_min(), self.z_min()),
            max: (self.x_max(), self.y_max(), self.z_max())
        }
    }

    /// This is equivalent to the volume (width * height * depth).
    pub fn len(&self) -> usize {
        self.size.0 * self.size.1 * self.size.2
    }

    pub fn iter<'a>(&'a self) -> RollGrid3DIterator<'a, T> {
        RollGrid3DIterator {
            bounds_iter: self.bounds().iter(),
            grid: self,
        }
    }

    pub fn iter_mut<'a>(&'a mut self) -> RollGrid3DMutIterator<'a, T> {
        RollGrid3DMutIterator {
            bounds_iter: self.bounds().iter(),
            grid: self,
        }
    }

}

impl<T: Copy> RollGrid3D<T> {
    pub fn get_copy<C: Into<Coord>>(&self, coord: C) -> Option<T> {
        let coord: Coord = coord.into();
        let index = self.offset_index(coord)?;
        self.cells[index]
    }
}

struct TempGrid3D<T> {
    pub cells: Box<[Option<T>]>,
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

    pub fn new_with_init<F: FnMut(Coord) -> Option<T>>(size: (usize, usize, usize), offset: (i32, i32, i32), init: F) -> Self {
        let bounds = Bounds3D::new(
            offset,
            (
                offset.0 + size.0 as i32,
                offset.1 + size.1 as i32,
                offset.2 + size.2 as i32
            )
        );
        Self {
            cells: bounds.iter().map(init).collect(),
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
    pub fn new<C: Into<(i32, i32, i32)>>(min: C, max: C) -> Self {
        Self {
            min: min.into(),
            max: max.into()
        }
    }

    pub fn from_bounds<C: Into<(i32, i32, i32)>>(a: C, b: C) -> Self {
        let a: (i32, i32, i32) = a.into();
        let b: (i32, i32, i32) = b.into();
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

    pub fn volume(&self) -> i128 {
        self.width() as i128 * self.height() as i128 * self.depth() as i128
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

    pub fn contains<P: Into<(i32, i32, i32)>>(self, point: P) -> bool {
        let point: (i32, i32, i32) = point.into();
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

pub struct RollGrid3DIterator<'a, T> {
    grid: &'a RollGrid3D<T>,
    bounds_iter: Bounds3DIter,
}

impl<'a, T> Iterator for RollGrid3DIterator<'a, T> {
    type Item = ((i32, i32, i32), Option<&'a T>);

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

pub struct RollGrid3DMutIterator<'a, T> {
    grid: &'a mut RollGrid3D<T>,
    bounds_iter: Bounds3DIter,
}

impl<'a, T> Iterator for RollGrid3DMutIterator<'a, T> {
    type Item = ((i32, i32, i32), Option<&'a mut T>);

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
            if let Some(cell) = &mut *cell_ptr {
                Some((next, Some(cell)))
            } else {
                Some((next, None))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_test() {
        let v = i64::MAX;
        let uv = v as u64;
        println!("{}\n{}", i32::MAX as u32, i32::MIN as u32);
    }

    #[test]
    fn iter_test() {
        let mut grid = RollGrid3D::new_with_init(2, 2, 2, (0, 0, 0), |pos: (i32, i32, i32)| {
            Some(pos)
        });
        grid.iter().for_each(|(pos, cell)| {
            if let Some(&cell) = cell {
                assert_eq!(cell, pos);
            } else {
                panic!()
            }
            let (x, y, z) = pos;
        });
        grid.iter_mut().for_each(|(pos, cell)| {
            if let Some(cell) = cell {
                cell.0 += 1;
                cell.1 += 1;
                cell.2 += 1;
            }
        });
        grid.iter().for_each(|(pos, cell)| {
            if let Some(&cell) = cell {
                let (x, y, z) = cell;
                let pos = (pos.0 + 1, pos.1 + 1, pos.2 + 1);
                assert_eq!(cell, pos);
                println!("({x:2},{y:2},{z:2})");
            } else {
                panic!()
            }
        });
    }

    #[test]
    fn reposition_test() {
        fn verify_grid(grid: &RollGrid3D<(i32, i32, i32)>) {
            let offset = grid.grid_offset;
            for y in grid.y_min()..grid.y_max() {
                for z in grid.z_min()..grid.z_max() {
                    for x in grid.x_min()..grid.x_max() {
                        let pos = (x, y, z);
                        let cell = grid.get(pos).expect("Cell was None");
                        assert_eq!(pos, *cell);
                    }
                }
            }
        }
        fn reload(old: (i32, i32, i32), new: (i32, i32, i32), old_value: Option<(i32, i32, i32)>) -> Option<(i32, i32, i32)> {
            assert_eq!(Some(old), old_value);
            Some(new)
        }
        let mut grid = RollGrid3D::new_with_init(4, 4, 4, (0, 0, 0), |pos| {
            Some(pos)
        });
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
            let offset = grid.grid_offset;
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
        itertools::iproduct!(
            1..7, 1..7, 1..7,
            -1..6, -1..6, -1..6
        ).for_each(|(width, height, depth, x, y, z)| {
            let mut grid = RollGrid3D::new_with_init(4, 4, 4, (0,0,0), |pos:(i32, i32, i32)| {
                Some(DropCoord::from(pos))
            });
            grid.resize_and_reposition(width, height, depth, (x, y, z), |action| {
                match action {
                    CellManage::Load(pos) => Some(DropCoord::from(pos)),
                    CellManage::Unload(pos, old_value) => {
                        let mut old = old_value.expect("Old Value was None");
                        old.unloaded = true;
                        assert_eq!(pos, old.coord);
                        None
                    }
                }
            });
            grid.iter_mut().for_each(|(pos, cell)| {
                if let Some(cell) = cell {
                    cell.unloaded = true;
                }
            });
            verify_grid(&grid);
        });
    }
}

// fn print_grid(grid: &RollGrid3D<(i32, i32, i32)>) {
//     println!("***");
//     for y in grid.y_min()..grid.y_max() {
//         println!("### Y = {y:<3} ###");
//         for z in grid.z_min()..grid.z_max() {
//             for x in grid.x_min()..grid.x_max() {
//                 let Some(&(cx, cy, cz)) = grid.get((x, y, z)) else {
//                     continue;
//                 };
//                 print!("({cx:2},{cy:2},{cz:2})");
//             }
//             println!();
//         }
//     }
// }

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
            || z > self.offset.2 + self.size.2 {
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
        size: (23, 32, 18)
    };
    let index = grid.offset_index(0, 0, 0).expect(OUT_OF_BOUNDS);
    println!("{index}");
    let (x, y, z) = grid.index_offset(index).expect(OUT_OF_BOUNDS);
    println!("({x}, {y}, {z})");
    for y in grid.offset.1..grid.offset.1 + grid.size.1 {
        for z in grid.offset.2..grid.offset.2 + grid.size.2 {
            for x in grid.offset.0..grid.offset.0 + grid.size.0 {
                let index = grid.offset_index(x, y, z).expect(OUT_OF_BOUNDS);
                let (rx, ry, rz) = grid.index_offset(index).expect(OUT_OF_BOUNDS);
                assert_eq!((rx, ry, rz), (x, y, z));
            }
        }
    }
}