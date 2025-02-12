use crate::fixedarray::FixedArray;

// TODO: There's a lot of work to be done here.
//       Ideally, `Grid3D` can be used to represent a slice of
//       a `RollGrid3D`.

pub struct Grid3D<T: Sized> {
    cells: FixedArray<T>,
    size: (u32, u32, u32),
}
