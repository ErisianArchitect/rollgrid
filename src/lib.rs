pub mod rollgrid2d;
pub mod rollgrid3d;

const SIZE_TOO_LARGE: &'static str = "Size is too large";
const OFFSET_TOO_CLOSE_TO_MAX: &'static str = "Offset is too close to maximum bound";
const OUT_OF_BOUNDS: &'static str = "Out of bounds";

// #[inline(always)]
// fn iproduct_arg_rev<T>(input: (T, T)) -> (T, T) {
//     (input.1, input.0)
// }

/// Used in the `manage` callback for loading and unloading cells during resize/reposition operations.
pub enum CellManage<C, T> {
    /// For when a cell is loaded.
    /// The callback should return the new value for the loaded cell.
    Load(C),
    /// For when a cell is unloaded.
    /// The callback should return `None`.
    Unload(C, Option<T>)
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
        let mut grid = RollGrid2D::new_with_init(2, 2, (0, 0), |coord: (i32, i32)| {
            Some(coord)
        });
        fn print_grid(grid: &RollGrid2D<(i32, i32)>) {
            println!("***");
            for y in grid.top()..grid.bottom() {
                for x in grid.left()..grid.right() {
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
