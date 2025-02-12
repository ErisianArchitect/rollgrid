use crate::fixedarray::FixedArray;

// TODO: There's a lot of work to be done here.
//       Ideally, `Grid3D` can be used to represent a slice of
//       a `RollGrid3D`. I'd like to be able to use this struct
//       to store a subgrid of references or mutable references.

pub struct Grid3D<T: Sized> {
    cells: FixedArray<T>,
    size: (u32, u32, u32),
}
