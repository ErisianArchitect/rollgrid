use crate::fixedarray::FixedArray;



pub struct Grid3D<T: Sized> {
    cells: FixedArray<T>,
    size: (u32, u32, u32),
}