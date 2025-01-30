use std::marker::PhantomData;

pub mod bounds2d;
pub mod bounds3d;
pub(crate) mod cells;
pub mod rollgrid2d;
pub mod rollgrid3d;
pub mod grid2d;
pub mod grid3d;
pub mod math;

mod error_messages {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct PanicMsg(&'static str);

    impl PanicMsg {
        pub fn panic(self) -> ! {
            panic!("{}", self.0);
        }

        pub fn assert(self, condition: bool) {
            if condition {
                self.panic();
            }
        }

        pub fn debug_assert(self, condition: bool) {
            #[cfg(debug_assertions)]
            if condition {
                self.panic();
            }
        }

        pub fn msg(self) -> &'static str {
            self.0
        }
    }

    impl std::fmt::Display for PanicMsg {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::ops::Deref for PanicMsg {
        type Target = str;
        fn deref(&self) -> &Self::Target {
            self.0
        }
    }

    macro_rules! msg {
        ($name:ident = $msg:literal) => {
            pub const $name: PanicMsg = PanicMsg($msg);
        };
    }
    msg!(UNALLOCATED_BUFFER = "Buffer was not allocated.");
    msg!(X_MAX_EXCEEDS_MAXIMUM = "X max bound exceeds maximum.");
    msg!(Y_MAX_EXCEEDS_MAXIMUM = "Y max bound exceeds maximum.");
    msg!(Z_MAX_EXCEEDS_MAXIMUM = "Z max bound exceeds maximum.");
    msg!(NOT_ALLOCATED = "Not allocated.");
    msg!(SIZE_TOO_LARGE = "Size is too large");
    msg!(OFFSET_TOO_CLOSE_TO_MAX = "Offset is too close to maximum bound");
    msg!(OUT_OF_BOUNDS = "Out of bounds");
    msg!(INDEX_OUT_OF_BOUNDS = "Index is out of bounds.");
    msg!(AREA_IS_ZERO = "Width/Height cannot be 0");
    msg!(VOLUME_IS_ZERO = "Width/Height/Depth cannot be 0");
    msg!(INFLATE_PAST_I32_MAX = "Cannot inflate more than i32::MAX");
    msg!(INFLATE_OVERFLOW = "Inflate operation results in integer overflow");
    msg!(DEFLATE_PAST_I32_MAX = "Cannot deflate more than i32::MAX");
    msg!(DEFLATE_OVERFLOW = "Deflate operation results in integer overflow");
}

/// A trait for managing cells during resize operations on grids.
///
/// You can easily create a [CellManager] to use as a [CellManage].
pub trait CellManage<C, T> {
    fn load(&mut self, position: C) -> T;
    fn unload(&mut self, position: C, old_value: T);
    fn reload(&mut self, old_position: C, new_position: C, value: &mut T);
}

/// A trait for managing cells during fallible resize operations on grids.
pub trait TryCellManage<C, T, E> {
    fn try_load(&mut self, position: C) -> Result<T, E>;
    fn try_unload(&mut self, position: C, old_value: T) -> Result<(), E>;
    fn try_reload(&mut self, old_position: C, new_position: C, value: &mut T) -> Result<(), E>;
}

/// Use the utility function [cell_manager] to create a [CellManager].
pub struct CellManager<C, T, FL, FU, FR, Marker = ()> {
    load: FL,
    unload: FU,
    reload: FR,
    phantom: std::marker::PhantomData<(C, T, Marker)>,
}

impl<C, T, FL, FU, FR> CellManage<C, T> for CellManager<C, T, FL, FU, FR>
where
    T: Sized,
    FL: FnMut(C) -> T,
    FU: FnMut(C, T),
    FR: FnMut(C, C, &mut T),
{
    /// Load the cell at `position`.
    fn load(&mut self, position: C) -> T {
        (self.load)(position)
    }

    /// Unload cell that was at `position`.
    fn unload(&mut self, position: C, value: T) {
        (self.unload)(position, value);
    }

    /// Reload cell that was at `old_position` and is being moved to `new_position`.
    fn reload(&mut self, old_position: C, new_position: C, value: &mut T) {
        (self.reload)(old_position, new_position, value);
    }
}

impl<C, T, E, FL, FU, FR> TryCellManage<C, T, E> for CellManager<C, T, FL, FU, FR, (E,)>
where
    T: Sized,
    FL: FnMut(C) -> Result<T, E>,
    FU: FnMut(C, T) -> Result<(), E>,
    FR: FnMut(C, C, &mut T) -> Result<(), E>,
{
    /// Load the cell at `position`.
    fn try_load(&mut self, position: C) -> Result<T, E> {
        (self.load)(position)
    }

    /// Unload cell that was at `position`.
    fn try_unload(&mut self, position: C, old_value: T) -> Result<(), E> {
        (self.unload)(position, old_value)
    }

    /// Reload cell that was at `old_position` and is being moved to `new_position`.
    fn try_reload(&mut self, old_position: C, new_position: C, value: &mut T) -> Result<(), E> {
        (self.reload)(old_position, new_position, value)
    }
}

/// Creates a [CellManager] instance that implements [CellManage] using the given `load`, `unload`, and `reload` functions.
pub fn cell_manager<C, T, FL, FU, FR>(
    load: FL,
    unload: FU,
    reload: FR,
) -> CellManager<C, T, FL, FU, FR>
where
    CellManager<C, T, FL, FU, FR>: CellManage<C, T>,
{
    CellManager {
        load,
        unload,
        reload,
        phantom: PhantomData,
    }
}

/// Creates a [CellManager] instance that implements [TryCellManage] using the given `load`, `unload`, and `reload` functions.
pub fn try_cell_manager<C, T, E, FL, FU, FR>(
    load: FL,
    unload: FU,
    reload: FR,
) -> CellManager<C, T, FL, FU, FR, (E,)>
where
    CellManager<C, T, FL, FU, FR, (E,)>: TryCellManage<C, T, E>,
{
    CellManager {
        load,
        unload,
        reload,
        phantom: PhantomData,
    }
}

#[cfg(test)]
mod tests {
    #![allow(unused)]
    use crate::bounds2d::Bounds2D;
    use crate::rollgrid2d::RollGrid2D;

    use super::*;

    #[test]
    pub fn roll_test() {
        const HEX_CHARS: [char; 16] = [
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
        ];
        let mut hex = HEX_CHARS.into_iter();
        let mut grid = RollGrid2D::new(4, 4, (0, 0), |pos: (i32, i32)| hex.next().unwrap());
        fn print_grid(grid: &RollGrid2D<char>) {
            for y in grid.y_min()..grid.y_max() {
                for x in grid.x_min()..grid.x_max() {
                    if let Some(c) = grid.get((x, y)) {
                        print!("{}", *c);
                    }
                }
                println!();
            }
        }
        print_grid(&grid);
        grid.translate((1, 1), |old_pos, new_pos, old_value| {});
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
                assert!(Bounds2D::from_bounds($a_min, $a_max)
                    .intersects(Bounds2D::from_bounds($b_min, $b_max)));
            };
            (($a_min:expr, $a_max:expr) -!> ($b_min:expr, $b_max:expr)) => {
                assert!(!Bounds2D::from_bounds($a_min, $a_max)
                    .intersects(Bounds2D::from_bounds($b_min, $b_max)));
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
        let mut grid = RollGrid2D::new(2, 2, (0, 0), |coord: (i32, i32)| coord);
        fn print_grid(grid: &RollGrid2D<(i32, i32)>) {
            println!("***");
            for y in grid.y_min()..grid.y_max() {
                for x in grid.x_min()..grid.x_max() {
                    if let Some(&(cx, cy)) = grid.get((x, y)) {
                        print!("({cx:3},{cy:3})");
                    }
                }
                println!();
            }
        }
        print_grid(&grid);
        grid.translate((1, 1), |old, new, old_value| {
            *old_value = old;
        });
        print_grid(&grid);
        return;
        grid.inflate_size(
            (1, 1),
            cell_manager(
                |pos: (i32, i32)| {
                    println!("Load: ({}, {})", pos.0, pos.1);
                    pos
                },
                |pos, value| {},
                |old_pos, new_pos, value| {},
            ),
        );
        println!("***");
        print_grid(&grid);
        if let Some(&(x, y)) = grid.get((-5, -16)) {
            println!("({x}, {y})");
        } else {
            println!("None");
        }
    }
}
