use std::marker::PhantomData;

pub(crate) mod cells;
pub mod rollgrid2d;
pub mod rollgrid3d;

// TODO: Move Bounds2D and Bounds3D into their own modules.

mod constants {
    pub const SIZE_TOO_LARGE: &'static str = "Size is too large";
    pub const OFFSET_TOO_CLOSE_TO_MAX: &'static str = "Offset is too close to maximum bound";
    pub const OUT_OF_BOUNDS: &'static str = "Out of bounds";
    pub const AREA_IS_ZERO_2D: &'static str = "Width/Height cannot be 0";
    pub const VOLUME_IS_ZERO: &'static str = "Width/Height/Depth cannot be 0";
    pub const INFLATE_PAST_I32_MAX: &'static str = "Cannot inflate more than i32::MAX";
    pub const INFLATE_OVERFLOW: &'static str = "Inflate operation results in integer overflow";
    pub const DEFLATE_PAST_I32_MAX: &'static str = "Cannot deflate more than i32::MAX";
    pub const DEFLATE_OVERFLOW: &'static str = "Deflate operation results in integer overflow";
}

// TODO: Write documentation for this stuff.
// TODO: Create an implementation for CellManage for FnMut(Manage) (where Manage is an enum with Load, Unload, and Reload).
pub trait CellManage<C, T> {
    fn load(&mut self, position: C) -> T;
    fn unload(&mut self, position: C, old_value: T);
    fn reload(&mut self, old_position: C, new_position: C, value: &mut T);
}

pub trait TryCellManage<C, T, E> {
    fn try_load(&mut self, position: C) -> Result<T, E>;
    fn try_unload(&mut self, position: C, old_value: T) -> Result<(), E>;
    fn try_reload(&mut self, old_position: C, new_position: C, value: &mut T) -> Result<(), E>;
}

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
    fn load(&mut self, position: C) -> T {
        (self.load)(position)
    }

    fn unload(&mut self, position: C, value: T) {
        (self.unload)(position, value);
    }

    fn reload(&mut self, old_position: C, new_position: C, value: &mut T) {
        (self.reload)(old_position, new_position, value);
    }
}

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
    use rollgrid2d::{Bounds2D, RollGrid2D};

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
            1,
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
