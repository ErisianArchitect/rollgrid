#![allow(unused)]

use crate::{CellManage, OFFSET_TOO_CLOSE_TO_MAX, OUT_OF_BOUNDS, SIZE_TOO_LARGE};
const VOLUME_IS_ZERO: &'static str = "Width/Height/Depth cannot be 0";

type Coord = (i32, i32, i32);

pub struct RollGrid3D<T> {
    cells: Vec<Option<T>>,
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
            // cells: Bounds3D::new(grid_offset, (grid_offset.0 + width as i32, grid_offset.1 + height as i32, grid_offset.2 + depth as i32))
            //     .iter()
            //     .map(C::from)
            //     .map(init)
            //     .collect(),
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
                // lx means low x   (-X) (-1, 0, 0)
                // hx means high x  (+X) ( 1, 0, 0)
                // mx means middle x (X) ( 0, 0, 0)
                // lx_ly_lz = (-1, -1, -1)
                // mx_ly_lz = ( 0, -1, -1)
                // hx_ly_lz = ( 1, -1, -1)
                // lx_ly_mz = (-1, -1,  0)
                // mx_ly_mz = ( 0, -1,  0)
                // hx_ly_mz = ( 1, -1,  0)
                // lx_ly_hz = (-1, -1,  1)
                // mx_ly_hz = ( 0, -1,  1)
                // hx_ly_hz = ( 1, -1,  1)
                // lx_my_lz = (-1,  0, -1)
                // mx_my_lz = ( 0,  0, -1)
                // hx_my_lz = ( 1,  0, -1)
                // lx_my_mz = (-1,  0,  0)
                // hx_my_mz = ( 1,  0,  0)
                // lx_my_hz = (-1,  0,  1)
                // mx_my_hz = ( 0,  0,  1)
                // hx_my_hz = ( 1,  0,  1)
                // lx_hy_lz = (-1,  1, -1)
                // mx_hy_lz = ( 0,  1, -1)
                // hx_hy_lz = ( 1,  1, -1)
                // lx_hy_mz = (-1,  1,  0)
                // mx_hy_mz = ( 0,  1,  0)
                // hx_hy_mz = ( 1,  1,  0)
                // lx_hy_hz = (-1,  1,  1)
                // mx_hy_hz = ( 0,  1,  1)
                // hx_hy_hz = ( 1,  1,  1)
                // These arcane looking identifiers are the names
                // of the unload sections. There might be 26, there might
                // be 0. Who knows?

                // lx_ly_lz = (-1, -1, -1)
                if old_bounds.x_min() < new_bounds.x_min()
                && old_bounds.y_min() < new_bounds.y_min()
                && old_bounds.z_min() < new_bounds.z_min() {
                    let x_min = old_bounds.x_min();
                    let y_min = old_bounds.y_min();
                    let z_min = old_bounds.z_min();
                    let x_max = new_bounds.x_min();
                    let y_max = new_bounds.y_min();
                    let z_max = new_bounds.z_min();
                    let bounds = Bounds3D::new(
                        (x_min, y_min, z_min),
                        (x_max, y_max, z_max)
                    );
                    bounds.iter().for_each(|pos| {
                        let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        manage(CellManage::Unload(C::from(pos), old_value));
                    });
                }
                // mx_ly_lz = ( 0, -1, -1)
                if old_bounds.y_min() < new_bounds.y_min()
                && old_bounds.z_min() < new_bounds.z_min() {
                    let x_min = old_bounds.x_min().max(new_bounds.x_min());
                    let y_min = old_bounds.y_min();
                    let z_min = old_bounds.z_min();
                    let x_max = old_bounds.x_max().min(new_bounds.x_max());
                    let y_max = new_bounds.y_min();
                    let z_max = new_bounds.z_min();
                    let bounds = Bounds3D::new(
                        (x_min, y_min, z_min),
                        (x_max, y_max, z_max)
                    );
                    bounds.iter().for_each(|pos| {
                        let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        manage(CellManage::Unload(C::from(pos), old_value));
                    });
                }
                // hx_ly_lz = ( 1, -1, -1)
                if old_bounds.x_max() > new_bounds.x_max()
                && old_bounds.y_min() < new_bounds.y_min()
                && old_bounds.z_min() < new_bounds.z_min() {
                    let x_min = new_bounds.x_max();
                    let y_min = old_bounds.y_min();
                    let z_min = old_bounds.z_min();
                    let x_max = old_bounds.x_max();
                    let y_max = new_bounds.y_min();
                    let z_max = new_bounds.z_min();
                    let bounds = Bounds3D::new(
                        (x_min, y_min, z_min),
                        (x_max, y_max, z_max)
                    );
                    bounds.iter().for_each(|pos| {
                        let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        manage(CellManage::Unload(C::from(pos), old_value));
                    });
                }
                // lx_ly_mz = (-1, -1,  0)
                if old_bounds.x_min() < new_bounds.x_min()
                && old_bounds.y_min() < new_bounds.y_min() {
                    let x_min = old_bounds.x_min();
                    let y_min = old_bounds.y_min();
                    let z_min = old_bounds.z_min().max(new_bounds.z_min());
                    let x_max = new_bounds.x_min();
                    let y_max = new_bounds.y_min();
                    let z_max = old_bounds.z_max().min(new_bounds.z_max());
                    let bounds = Bounds3D::new(
                        (x_min, y_min, z_min),
                        (x_max, y_max, z_max)
                    );
                    bounds.iter().for_each(|pos| {
                        let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        manage(CellManage::Unload(C::from(pos), old_value));
                    });
                }
                // mx_ly_mz = ( 0, -1,  0)
                if old_bounds.y_min() < new_bounds.y_min() {
                    let x_min = old_bounds.x_min().max(new_bounds.x_min());
                    let y_min = old_bounds.y_min();
                    let z_min = old_bounds.z_min().max(new_bounds.z_min());
                    let x_max = old_bounds.x_max().min(new_bounds.x_max());
                    let y_max = new_bounds.y_min();
                    let z_max = old_bounds.z_max().min(new_bounds.z_max());
                    let bounds = Bounds3D::new(
                        (x_min, y_min, z_min),
                        (x_max, y_max, z_max)
                    );
                    bounds.iter().for_each(|pos| {
                        let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        manage(CellManage::Unload(C::from(pos), old_value));
                    });
                }
                // hx_ly_mz = ( 1, -1,  0)
                if old_bounds.x_max() > new_bounds.x_max()
                && old_bounds.y_min() < new_bounds.y_min() {
                    let x_min = new_bounds.x_max();
                    let y_min = old_bounds.y_min();
                    let z_min = old_bounds.z_min().max(new_bounds.z_min());
                    let x_max = old_bounds.x_max();
                    let y_max = new_bounds.y_min();
                    let z_max = old_bounds.z_max().min(new_bounds.z_max());
                    let bounds = Bounds3D::new(
                        (x_min, y_min, z_min),
                        (x_max, y_max, z_max)
                    );
                    bounds.iter().for_each(|pos| {
                        let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        manage(CellManage::Unload(C::from(pos), old_value));
                    });
                }
                // lx_ly_hz = (-1, -1,  1)
                if old_bounds.x_min() < new_bounds.x_min()
                && old_bounds.y_min() < new_bounds.y_min()
                && old_bounds.z_max() > new_bounds.z_max() {
                    let x_min = old_bounds.x_min();
                    let y_min = old_bounds.y_min();
                    let z_min = new_bounds.z_max();
                    let x_max = new_bounds.x_min();
                    let y_max = new_bounds.y_min();
                    let z_max = old_bounds.z_max();
                    let bounds = Bounds3D::new(
                        (x_min, y_min, z_min),
                        (x_max, y_max, z_max)
                    );
                    bounds.iter().for_each(|pos| {
                        let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        manage(CellManage::Unload(C::from(pos), old_value));
                    });
                }
                // mx_ly_hz = ( 0, -1,  1)
                if old_bounds.y_min() < new_bounds.y_min()
                && old_bounds.z_max() > new_bounds.z_max() {
                    let x_min = old_bounds.x_min().max(new_bounds.x_min());
                    let y_min = old_bounds.y_min();
                    let z_min = new_bounds.z_max();
                    let x_max = old_bounds.x_max().min(new_bounds.x_max());
                    let y_max = new_bounds.y_min();
                    let z_max = old_bounds.z_max();
                    let bounds = Bounds3D::new(
                        (x_min, y_min, z_min),
                        (x_max, y_max, z_max)
                    );
                    bounds.iter().for_each(|pos| {
                        let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        manage(CellManage::Unload(C::from(pos), old_value));
                    });
                }
                // hx_ly_hz = ( 1, -1,  1)
                if old_bounds.x_max() > new_bounds.x_max()
                && old_bounds.y_min() < new_bounds.y_min()
                && old_bounds.z_max() > new_bounds.z_max() {
                    let x_min = new_bounds.x_max();
                    let y_min = old_bounds.y_min();
                    let z_min = new_bounds.z_max();
                    let x_max = old_bounds.x_max();
                    let y_max = new_bounds.y_min();
                    let z_max = old_bounds.z_max();
                    let bounds = Bounds3D::new(
                        (x_min, y_min, z_min),
                        (x_max, y_max, z_max)
                    );
                    bounds.iter().for_each(|pos| {
                        let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        manage(CellManage::Unload(C::from(pos), old_value));
                    });
                }
                // lx_my_lz = (-1,  0, -1)
                if old_bounds.x_min() < new_bounds.x_min()
                && old_bounds.z_min() < new_bounds.z_min() {
                    let x_min = old_bounds.x_min();
                    let y_min = old_bounds.y_min().max(new_bounds.y_min());
                    let z_min = old_bounds.z_min();
                    let x_max = new_bounds.x_min();
                    let y_max = old_bounds.y_max().min(new_bounds.y_max());
                    let z_max = new_bounds.z_min();
                    let bounds = Bounds3D::new(
                        (x_min, y_min, z_min),
                        (x_max, y_max, z_max)
                    );
                    bounds.iter().for_each(|pos| {
                        let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        manage(CellManage::Unload(C::from(pos), old_value));
                    });
                }
                // mx_my_lz = ( 0,  0, -1)
                if old_bounds.z_min() < new_bounds.z_min() {
                    let x_min = old_bounds.x_min().max(new_bounds.x_min());
                    let y_min = old_bounds.y_min().max(new_bounds.y_min());
                    let z_min = old_bounds.z_min();
                    let x_max = old_bounds.x_max().min(new_bounds.x_max());
                    let y_max = old_bounds.y_max().min(new_bounds.y_max());
                    let z_max = new_bounds.z_min();
                    let bounds = Bounds3D::new(
                        (x_min, y_min, z_min),
                        (x_max, y_max, z_max)
                    );
                    bounds.iter().for_each(|pos| {
                        let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        manage(CellManage::Unload(C::from(pos), old_value));
                    });
                }
                // hx_my_lz = ( 1,  0, -1)
                if old_bounds.x_max() > new_bounds.x_max()
                && old_bounds.z_min() < new_bounds.z_min() {
                    let x_min = new_bounds.x_max();
                    let y_min = old_bounds.y_min().max(new_bounds.y_min());
                    let z_min = old_bounds.z_min();
                    let x_max = old_bounds.x_max();
                    let y_max = old_bounds.y_max().min(new_bounds.y_max());
                    let z_max = new_bounds.z_min();
                    let bounds = Bounds3D::new(
                        (x_min, y_min, z_min),
                        (x_max, y_max, z_max)
                    );
                    bounds.iter().for_each(|pos| {
                        let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        manage(CellManage::Unload(C::from(pos), old_value));
                    });
                }
                // lx_my_mz = (-1,  0,  0)
                if old_bounds.x_min() < new_bounds.x_min() {
                    let x_min = old_bounds.x_min();
                    let y_min = old_bounds.y_min().max(new_bounds.y_min());
                    let z_min = old_bounds.z_min().max(new_bounds.z_min());
                    let x_max = new_bounds.x_min();
                    let y_max = old_bounds.y_max().min(new_bounds.y_max());
                    let z_max = old_bounds.z_max().min(new_bounds.z_max());
                    let bounds = Bounds3D::new(
                        (x_min, y_min, z_min),
                        (x_max, y_max, z_max)
                    );
                    bounds.iter().for_each(|pos| {
                        let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        manage(CellManage::Unload(C::from(pos), old_value));
                    });
                }
                // hx_my_mz = ( 1,  0,  0)
                if old_bounds.x_max() > new_bounds.x_max() {
                    let x_min = new_bounds.x_max();
                    let y_min = old_bounds.y_min().max(new_bounds.y_min());
                    let z_min = old_bounds.z_min().max(new_bounds.z_min());
                    let x_max = old_bounds.x_max();
                    let y_max = old_bounds.y_max().min(new_bounds.y_max());
                    let z_max = old_bounds.z_max().min(new_bounds.z_max());
                    let bounds = Bounds3D::new(
                        (x_min, y_min, z_min),
                        (x_max, y_max, z_max)
                    );
                    bounds.iter().for_each(|pos| {
                        let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        manage(CellManage::Unload(C::from(pos), old_value));
                    });
                }
                // lx_my_hz = (-1,  0,  1)
                if old_bounds.x_min() < new_bounds.x_min()
                && old_bounds.z_max() > new_bounds.z_max() {
                    let x_min = old_bounds.x_min();
                    let y_min = old_bounds.y_min().max(new_bounds.y_min());
                    let z_min = new_bounds.z_max();
                    let x_max = new_bounds.x_min();
                    let y_max = old_bounds.y_max().min(new_bounds.y_max());
                    let z_max = old_bounds.z_max();
                    let bounds = Bounds3D::new(
                        (x_min, y_min, z_min),
                        (x_max, y_max, z_max)
                    );
                    bounds.iter().for_each(|pos| {
                        let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        manage(CellManage::Unload(C::from(pos), old_value));
                    });
                }
                // mx_my_hz = ( 0,  0,  1)
                if old_bounds.z_max() > new_bounds.z_max() {
                    let x_min = old_bounds.x_min().max(new_bounds.x_min());
                    let y_min = old_bounds.y_min().max(new_bounds.y_min());
                    let z_min = new_bounds.z_max();
                    let x_max = old_bounds.x_max().min(new_bounds.x_max());
                    let y_max = old_bounds.y_max().min(new_bounds.y_max());
                    let z_max = old_bounds.z_max();
                    let bounds = Bounds3D::new(
                        (x_min, y_min, z_min),
                        (x_max, y_max, z_max)
                    );
                    bounds.iter().for_each(|pos| {
                        let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        manage(CellManage::Unload(C::from(pos), old_value));
                    });
                }
                // hx_my_hz = ( 1,  0,  1)
                if old_bounds.x_max() > new_bounds.x_max()
                && old_bounds.z_max() > new_bounds.z_max() {
                    let x_min = new_bounds.x_max();
                    let y_min = old_bounds.y_min().max(new_bounds.y_min());
                    let z_min = new_bounds.z_max();
                    let x_max = old_bounds.x_max();
                    let y_max = old_bounds.y_max().min(new_bounds.y_max());
                    let z_max = old_bounds.z_max();
                    let bounds = Bounds3D::new(
                        (x_min, y_min, z_min),
                        (x_max, y_max, z_max)
                    );
                    bounds.iter().for_each(|pos| {
                        let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        manage(CellManage::Unload(C::from(pos), old_value));
                    });
                }
                // lx_hy_lz = (-1,  1, -1)
                if old_bounds.x_min() < new_bounds.x_min()
                && old_bounds.y_max() > new_bounds.y_max()
                && old_bounds.z_min() < new_bounds.z_min() {
                    let x_min = old_bounds.x_min();
                    let y_min = new_bounds.y_max();
                    let z_min = old_bounds.z_min();
                    let x_max = new_bounds.x_min();
                    let y_max = old_bounds.y_max();
                    let z_max = new_bounds.z_min();
                    let bounds = Bounds3D::new(
                        (x_min, y_min, z_min),
                        (x_max, y_max, z_max)
                    );
                    bounds.iter().for_each(|pos| {
                        let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        manage(CellManage::Unload(C::from(pos), old_value));
                    });
                }
                // mx_hy_lz = ( 0,  1, -1)
                if old_bounds.y_max() > new_bounds.y_max()
                && old_bounds.z_min() < new_bounds.z_min() {
                    let x_min = old_bounds.x_min().max(new_bounds.x_min());
                    let y_min = new_bounds.y_max();
                    let z_min = old_bounds.z_min();
                    let x_max = old_bounds.x_max().min(new_bounds.x_max());
                    let y_max = old_bounds.y_max();
                    let z_max = new_bounds.z_min();
                    let bounds = Bounds3D::new(
                        (x_min, y_min, z_min),
                        (x_max, y_max, z_max)
                    );
                    bounds.iter().for_each(|pos| {
                        let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        manage(CellManage::Unload(C::from(pos), old_value));
                    });
                }
                // hx_hy_lz = ( 1,  1, -1)
                if old_bounds.x_max() > new_bounds.x_max()
                && old_bounds.y_max() > new_bounds.y_max()
                && old_bounds.z_min() < new_bounds.z_min() {
                    let x_min = new_bounds.x_max();
                    let y_min = new_bounds.y_max();
                    let z_min = old_bounds.z_min();
                    let x_max = old_bounds.x_max();
                    let y_max = old_bounds.y_max();
                    let z_max = new_bounds.z_min();
                    let bounds = Bounds3D::new(
                        (x_min, y_min, z_min),
                        (x_max, y_max, z_max)
                    );
                    bounds.iter().for_each(|pos| {
                        let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        manage(CellManage::Unload(C::from(pos), old_value));
                    });
                }
                // lx_hy_mz = (-1,  1,  0)
                if old_bounds.x_min() < new_bounds.x_min()
                && old_bounds.y_max() > new_bounds.y_max() {
                    let x_min = old_bounds.x_min();
                    let y_min = new_bounds.y_max();
                    let z_min = old_bounds.z_min().max(new_bounds.z_min());
                    let x_max = new_bounds.x_min();
                    let y_max = old_bounds.y_max();
                    let z_max = old_bounds.z_max().min(new_bounds.z_max());
                    let bounds = Bounds3D::new(
                        (x_min, y_min, z_min),
                        (x_max, y_max, z_max)
                    );
                    bounds.iter().for_each(|pos| {
                        let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        manage(CellManage::Unload(C::from(pos), old_value));
                    });
                }
                // mx_hy_mz = ( 0,  1,  0)
                if old_bounds.y_max() > new_bounds.y_max() {
                    let x_min = old_bounds.x_min().max(new_bounds.x_min());
                    let y_min = new_bounds.y_max();
                    let z_min = old_bounds.z_min().max(new_bounds.z_min());
                    let x_max = old_bounds.x_max().min(new_bounds.x_max());
                    let y_max = old_bounds.y_max();
                    let z_max = old_bounds.z_max().min(new_bounds.z_max());
                    let bounds = Bounds3D::new(
                        (x_min, y_min, z_min),
                        (x_max, y_max, z_max)
                    );
                    bounds.iter().for_each(|pos| {
                        let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        manage(CellManage::Unload(C::from(pos), old_value));
                    });
                }
                // hx_hy_mz = ( 1,  1,  0)
                if old_bounds.x_max() > new_bounds.x_max()
                && old_bounds.y_max() > new_bounds.y_max() {
                    let x_min = new_bounds.x_max();
                    let y_min = new_bounds.y_max();
                    let z_min = old_bounds.z_min().max(new_bounds.z_min());
                    let x_max = old_bounds.x_max();
                    let y_max = old_bounds.y_max();
                    let z_max = old_bounds.z_max().min(new_bounds.z_max());
                    let bounds = Bounds3D::new(
                        (x_min, y_min, z_min),
                        (x_max, y_max, z_max)
                    );
                    bounds.iter().for_each(|pos| {
                        let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        manage(CellManage::Unload(C::from(pos), old_value));
                    });
                }
                // lx_hy_hz = (-1,  1,  1)
                if old_bounds.x_min() < new_bounds.x_min()
                && old_bounds.y_max() > new_bounds.y_max()
                && old_bounds.z_max() > new_bounds.z_max() {
                    let x_min = old_bounds.x_min();
                    let y_min = new_bounds.y_max();
                    let z_min = new_bounds.z_max();
                    let x_max = new_bounds.x_min();
                    let y_max = old_bounds.y_max();
                    let z_max = old_bounds.z_max();
                    let bounds = Bounds3D::new(
                        (x_min, y_min, z_min),
                        (x_max, y_max, z_max)
                    );
                    bounds.iter().for_each(|pos| {
                        let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        manage(CellManage::Unload(C::from(pos), old_value));
                    });
                }
                // mx_hy_hz = ( 0,  1,  1)
                if old_bounds.y_max() > new_bounds.y_max()
                && old_bounds.z_max() > new_bounds.z_max() {
                    let x_min = old_bounds.x_min().max(new_bounds.x_min());
                    let y_min = new_bounds.y_max();
                    let z_min = new_bounds.z_max();
                    let x_max = old_bounds.x_max().min(new_bounds.x_max());
                    let y_max = old_bounds.y_max();
                    let z_max = old_bounds.z_max();
                    let bounds = Bounds3D::new(
                        (x_min, y_min, z_min),
                        (x_max, y_max, z_max)
                    );
                    bounds.iter().for_each(|pos| {
                        let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        manage(CellManage::Unload(C::from(pos), old_value));
                    });
                }
                // hx_hy_hz = ( 1,  1,  1)
                if old_bounds.x_max() > new_bounds.x_max()
                && old_bounds.y_max() > new_bounds.y_max()
                && old_bounds.z_max() > new_bounds.z_max() {
                    let x_min = new_bounds.x_max();
                    let y_min = new_bounds.y_max();
                    let z_min = new_bounds.z_max();
                    let x_max = old_bounds.x_max();
                    let y_max = old_bounds.y_max();
                    let z_max = old_bounds.z_max();
                    let bounds = Bounds3D::new(
                        (x_min, y_min, z_min),
                        (x_max, y_max, z_max)
                    );
                    bounds.iter().for_each(|pos| {
                        let index = self.offset_index(pos).expect(OUT_OF_BOUNDS);
                        let old_value = self.cells[index].take();
                        manage(CellManage::Unload(C::from(pos), old_value));
                    });
                }
            } else { // !old_bounds.intersects(new_bounds)

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

}

struct TempGrid3D<T> {
    pub cells: Vec<Option<T>>,
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
    pub fn new(min: (i32, i32, i32), max: (i32, i32, i32)) -> Self {
        Self {
            min,
            max
        }
    }

    pub fn from_bounds(a: (i32, i32, i32), b: (i32, i32, i32)) -> Self {
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

    pub fn volume(&self) -> i32 {
        self.width() * self.height() * self.depth()
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

    pub fn contains(self, point: (i32, i32, i32)) -> bool {
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

#[test]
fn bounds_test() {
    let bounds = Bounds3D::new((0, 0, 0), (3, 3, 3));
    bounds.iter().for_each(|(x, y, z)| {
        println!("({x},{y},{z})");
    });
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