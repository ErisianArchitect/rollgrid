use std::marker::PhantomData;

pub(crate) mod cells;
pub mod rollgrid2d;
pub mod rollgrid3d;

const SIZE_TOO_LARGE: &'static str = "Size is too large";
const OFFSET_TOO_CLOSE_TO_MAX: &'static str = "Offset is too close to maximum bound";
const OUT_OF_BOUNDS: &'static str = "Out of bounds";

const AREA_IS_ZERO_2D: &'static str = "Width/Height cannot be 0";
const VOLUME_IS_ZERO: &'static str = "Width/Height/Depth cannot be 0";

// TODO: manual_allocation: Update the Unload functionality.
/// Used in the `manage` callback for loading and unloading cells during resize/reposition operations.
// pub enum CellManage<'a, T> {
//     /// For when a cell is loaded.
//     /// The callback should return the new value for the loaded cell.
//     /// `Load(position, cell)`
//     Load((i32, i32), &'a mut T),
//     /// For when a cell is unloaded.
//     /// The callback should return `None`.
//     Unload((i32, i32), T),
//     /// For when a cell is reloaded (changes position).
//     /// `Reload(old_position, new_position, cell)`
//     Reload((i32, i32), (i32, i32), &'a mut T),
// }

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

pub struct CellManager<C, T, FL, FU, FR, Marker=()> {
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
FR: FnMut(C, C, &mut T) {
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

pub fn cell_manager<C, T, FL, FU, FR>(load: FL, unload: FU, reload: FR) -> CellManager<C, T, FL, FU, FR>
where CellManager<C, T, FL, FU, FR>: CellManage<C, T> {
    CellManager { load, unload, reload, phantom: PhantomData }
}

pub fn try_cell_manager<C, T, E, FL, FU, FR>(load: FL, unload: FU, reload: FR) -> CellManager<C, T, FL, FU, FR, (E,)>
where CellManager<C, T, FL, FU, FR, (E,)>: TryCellManage<C, T, E> {
    CellManager { load, unload, reload, phantom: PhantomData }
}

#[cfg(test)]
mod tests {
    #![allow(unused)]
    use rollgrid2d::{RollGrid2D, Bounds2D};

    use super::*;

    #[test]
    pub fn roll_test() {
        const HEX_CHARS: [char; 16] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F'];
        let mut hex = HEX_CHARS.into_iter();
        let mut grid = RollGrid2D::new(4, 4, (0, 0), |pos: (i32, i32)| {
            hex.next()
        });
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
        let mut grid = RollGrid2D::new(2, 2, (0, 0), |coord: (i32, i32)| {
            Some(coord)
        });
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
            Some(old)
        });
        print_grid(&grid);
        return;
        grid.inflate_size::<(i32, i32), _>(1, |action| {
            match action {
                CellManage::Load(pos) => {
                    println!("Load: ({},{})", pos.0, pos.1);
                    Some(pos)
                }
                CellManage::Unload(pos, old) => {
                    println!("Unload: ({},{})", pos.0, pos.1);
                    None
                }
            }
        });
        // grid.resize_and_reposition(3, 3, (4, 4), |action| {
        //     match action {
        //         CellManage::Load(pos) => {
        //             println!("Load: ({},{})", pos.0, pos.1);
        //             Some(pos)
        //         }
        //         CellManage::Unload(pos, old) => {
        //             println!("Unload: ({},{})", pos.0, pos.1);
        //             None
        //         }
        //     }
        // });
        // print_grid(&grid);
        // grid.translate((-1, -1), |old_pos, new_pos, old_value| {
        //     let (old_x, old_y) = old_pos;
        //     let (new_x, new_y) = new_pos;
        //     println!("({old_x},{old_y}) -> ({new_x},{new_y})");
        //     Some(new_pos)
        // });
        println!("***");
        print_grid(&grid);
        if let Some(&(x, y)) = grid.get((-5, -16)) {
            println!("({x}, {y})");
        } else {
            println!("None");
        }
    }
}
