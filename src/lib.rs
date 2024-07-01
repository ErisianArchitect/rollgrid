// use itertools::Itertools;

const SIZE_TOO_LARGE: &'static str = "Size is too large";
const AREA_IS_ZERO: &'static str = "Width/Height cannot be 0";
const OFFSET_TOO_CLOSE_TO_MAX: &'static str = "Offset is too close to maximum bound";
const OUT_OF_BOUNDS: &'static str = "Out of bounds";
pub type Coord = (i32, i32);

// #[inline(always)]
// fn iproduct_arg_rev<T>(input: (T, T)) -> (T, T) {
//     (input.1, input.0)
// }

/// 
pub enum CellManage<C, T> {
    Load(C),
    /// For when a cell is unloaded.
    /// The callback should return `None`.
    Unload(C, Option<T>)
}

struct TempGrid<T> {
    cells: Vec<Option<T>>,
    size: (usize, usize),
    offset: (i32, i32),
}

impl<T> TempGrid<T> {
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
        // let (wrap_x, wrap_y) = (self.wrap_offset.0 as i32, self.wrap_offset.1 as i32);
        // let wx = (nx + wrap_x).rem_euclid(width);
        // let wy = (ny + wrap_y).rem_euclid(height);
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
}

pub struct RollGrid2D<T> {
    cells: Vec<Option<T>>,
    size: (usize, usize),
    wrap_offset: (usize, usize),
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

    // Resize
    pub fn resize_and_reposition<C, F>(&mut self, new_width: usize, new_height: usize, new_position: C, manage: F)
    where
        C: Into<Coord> + From<Coord>,
        F: FnMut(CellManage<C, T>) -> Option<T> {
            #![allow(unused)]
            let area = new_width.checked_mul(new_height).expect(SIZE_TOO_LARGE);
            if area == 0 { panic!("{AREA_IS_ZERO}"); }
            #[cfg(target_pointer_width = "64")]
            if area > i32::MAX as usize { panic!("{SIZE_TOO_LARGE}"); }
            let (new_x, new_y): Coord = new_position.into();
            let nw = new_width as i32;
            let nh = new_height as i32;
            // Determine what needs to be unloaded
            let (left, right, top, bottom) = (self.left(), self.right(), self.top(), self.bottom());
            let old_bounds: Bounds2D = self.bounds();
            let new_bounds = Bounds2D::new((new_x, new_y), (new_x + nw, new_y + nh));
            if old_bounds.intersects(new_bounds) {
                let unload_top = if old_bounds.top() < new_bounds.top() {
                    Some({
                        let left = old_bounds.left();
                        let right = new_bounds.right().min(old_bounds.right());
                        let top = old_bounds.top();
                        let bottom = new_bounds.top().min(old_bounds.bottom());
                        Bounds2D::new((left, top), (right, bottom))
                    })
                } else {
                    None
                };
                let unload_right = if old_bounds.right() > new_bounds.right() {
                    Some({
                        let left = new_bounds.right();
                        let right = old_bounds.right();
                        let top = old_bounds.top();
                        let bottom = new_bounds.bottom().min(new_bounds.bottom());
                        Bounds2D::new((left, top), (right, bottom))
                    })
                } else {
                    None
                };
                let unload_bottom = if old_bounds.bottom() > new_bounds.bottom() {
                    Some({
                        let left = new_bounds.left().max(old_bounds.left());
                        let right = old_bounds.right();
                        let top = new_bounds.bottom();
                        let bottom = old_bounds.bottom();
                        Bounds2D::new((left, top), (right, bottom))
                    })
                } else {
                    None
                };
                let unload_left = if old_bounds.left() < new_bounds.left() {
                    Some({
                        let left = old_bounds.left();
                        let right = new_bounds.left();
                        let top = new_bounds.top().max(old_bounds.top());
                        let bottom = old_bounds.bottom();
                        Bounds2D::new((left, top), (right, bottom))
                    })
                } else {
                    None
                };
            } else {

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
            let (curx, cury) = self.grid_offset;
            let (px, py): (i32, i32) = position.into();
            let offset = (
                px - curx,
                py - cury
            );
            if offset == (0, 0) {
                return;
            }
            let mut reload = reload;
            let width = self.size.0 as i32;
            let height = self.size.1 as i32;
            let (offset_x, offset_y) = offset;
            let (old_x, old_y) = self.grid_offset;
            let (new_x, new_y) = (old_x + offset_x, old_y + offset_y);
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
                self.wrap_offset = (new_rolled_x as usize, new_rolled_y as usize);
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

    pub fn get_opt<C: Into<(i32, i32)>>(&self, coord: C) -> Option<&Option<T>> {
        let index = self.offset_index(coord.into())?;
        Some(&self.cells[index])
    }

    pub fn get_opt_mut<C: Into<(i32, i32)>>(&mut self, coord: C) -> Option<&mut Option<T>> {
        let index = self.offset_index(coord.into())?;
        Some(&mut self.cells[index])
    }

    pub fn set_opt<C: Into<(i32, i32)>>(&mut self, coord: C, value: Option<T>) -> Option<Option<T>> {
        let cell = self.get_opt_mut(coord.into())?;
        let mut old = value;
        std::mem::swap(&mut old, cell);
        Some(old)
    }

    pub fn relative_offset<C: Into<(i32, i32)> + From<(i32, i32)>>(&self, coord: C) -> C {
        let (x, y): (i32, i32) = coord.into();
        C::from((
            x - self.grid_offset.0,
            y - self.grid_offset.1
        ))
    }

    pub fn get<C: Into<(i32, i32)>>(&self, coord: C) -> Option<&T> {
        let index = self.offset_index(coord.into())?;
        if let Some(cell) = &self.cells[index] {
            Some(cell)
        } else {
            None
        }
    }

    pub fn get_mut<C: Into<(i32, i32)>>(&mut self, coord: C) -> Option<&mut T> {
        let index = self.offset_index(coord.into())?;
        if let Some(cell) = &mut self.cells[index] {
            Some(cell)
        } else {
            None
        }
    }

    pub fn set<C: Into<(i32, i32)>>(&mut self, coord: C, value: T) -> Option<T> {
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

    pub fn wrap_offset(&self) -> (usize, usize) {
        self.wrap_offset
    }

    pub fn grid_offset(&self) -> (i32, i32) {
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

}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bounds2D {
    /// Inclusive minimum.
    pub min: (i32, i32),
    /// Exclusive maximum.
    pub max: (i32, i32),
}

impl Bounds2D {
    pub fn new(min: (i32, i32), max: (i32, i32)) -> Self {
        Self {
            min, 
            max
        }
    }

    pub fn width(&self) -> i32 {
        self.max.0 - self.min.0
    }

    pub fn height(&self) -> i32 {
        self.max.1 - self.min.1
    }

    pub fn area(&self) -> i32 {
        self.width() * self.height()
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

    pub fn from_bounds(a: (i32, i32), b: (i32, i32)) -> Self {
        let (ax, ay) = a;
        let (bx, by) = b;
        let min = (ax.min(bx), ay.min(by));
        let max = (ax.max(bx), ay.max(by));
        Self {
            min,
            max
        }
    }

    pub fn intersects(self, other: Bounds2D) -> bool {
        let ((aleft, atop), (aright, abottom)) = (self.min, self.max);
        let ((bleft, btop), (bright, bbottom)) = (other.min, other.max);
        aleft < bright
        && bleft < aright
        && atop < bbottom
        && btop < abottom
    }

    pub fn iter(self) -> BoundsIter {
        BoundsIter {
            bounds: self,
            current: self.min,
        }
    }
}

pub struct BoundsIter {
    bounds: Bounds2D,
    current: (i32, i32),
}

impl Iterator for BoundsIter {
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

#[cfg(test)]
mod tests {
    #![allow(unused)]
    use super::*;

    #[test]
    pub fn roll_test() {
        const HEX_CHARS: [char; 16] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F'];
        let mut hex = HEX_CHARS.into_iter();
        let mut grid = RollGrid2D::new_with_init(4, 4, (0, 0), |pos: (i32, i32)| {
            hex.next()
        });
        fn print_grid(grid: &RollGrid2D<char>) {
            for y in grid.top()..grid.bottom() {
                for x in grid.left()..grid.right() {
                    if let Some(c) = grid.get((x, y)) {
                        print!("{}", *c);
                    }
                }
                println!();
            }
        }
        print_grid(&grid);
        grid.translate((1, 1), |old_pos, new_pos, old_value| {
            old_value
        });
        print_grid(&grid);
    }

    #[test]
    pub fn bounds_test() {
        // let a = Bounds2D::from_bounds((0, 0), (3, 3));
        // a.iter().for_each(|(x, y)| {
        //     println!("({x}, {y})");
        // });
        macro_rules! intersect {
            (($a_min:expr, $a_max:expr) -=> ($b_min:expr, $b_max:expr)) => {
                assert!(
                    Bounds2D::from_bounds($a_min, $a_max).intersects(
                        Bounds2D::from_bounds($b_min, $b_max)
                    )
                );
            };
            (($a_min:expr, $a_max:expr) -!> ($b_min:expr, $b_max:expr)) => {
                assert!(
                    !Bounds2D::from_bounds($a_min, $a_max).intersects(
                        Bounds2D::from_bounds($b_min, $b_max)
                    )
                );
            };
        }
        intersect!(((0, 0), (3, 3)) -!> ((3, 0), (6, 3)));
        intersect!(((0, 0), (1, 1)) -=> ((0, 0), (1, 1)));
        intersect!(((-1, -1), (0, 0)) -=> ((-1, -1), (0, 0)));
        intersect!(((0, 0), (3, 3)) -=> ((1, 1), (2, 2)));
        intersect!(((1, 1), (2, 2)) -=> ((0, 0), (3, 3)));
        intersect!(((0, 0), (1, 1)) -!> ((1, 0), (2, 1)));
        intersect!(((1, 0), (2, 1)) -!> ((0, 0), (1, 1)));
        intersect!(((0, 0), (1, 1)) -!> ((0, 1), (1, 2)));
        intersect!(((0, 1), (1, 2)) -!> ((0, 0), (1, 1)));
        
    }

    #[test]
    pub fn rollgrid2d_test() {
        let mut grid = RollGrid2D::new_with_init(2, 2, (-1, -1), |coord: (i32, i32)| {
            Some(coord)
        });
        grid.translate((-3, -15), |old_pos, new_pos, old_value| {
            let (old_x, old_y) = old_pos;
            let (new_x, new_y) = new_pos;
            println!("({old_x},{old_y}) -> ({new_x},{new_y})");
            Some(new_pos)
        });
        if let Some(&(x, y)) = grid.get((-5, -16)) {
            println!("({x}, {y})");
        } else {
            println!("None");
        }
    }
}
