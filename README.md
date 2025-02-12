# WARNING!!!

> This project is currently very broken. I'm working on fixing it. Please be patient.

Implementation of a rolling grid. A type that stores values in a 2D (or 3D) grid and can translate the grid to a new location. This is ideal for pseudo-infinite worlds where you want a buffer to store a region of chunks that can be moved around. Moving the region (and setting new values) is O(n) where n is the number of cells that would change position during the move operation. The move operation will call a callback with the old position, the new position, and the old value and the callback is expected to return the new value. This functionality allows you to move the grid and save any cells being unloaded and load the new cell at the same time.
The move operation can be thought of as a 2D (or 3D) ring buffer.
If it were 1D, a move operation might look like this:
```
Offset: 1
   old: [0, 1, 2, 3, 4]
   new: [1, 2, 3, 4, 0]
```
As you can see, the 0 was moved to the end of the buffer. 
Since the 0 was moved to the end of the buffer, it needs to be reloaded, so the move operation will call reload:
```
reload(0, 5, Some(0))
```

Let's say you had this grid:
```
0123
4567
89AB
CDEF
```
If you were to translate it by the offset `(1, 1)`, the result would be this grid (without modifying the values during repositioning):
```
5674
9AB8
DEFC
1230
```

Now, that's not the benefit of this library. When repositioning the grid, it doesn't move any elements at all. Instead, it keeps track of movement offsets and allows the reposition operation to be really fast because it's not actually moving anything around in memory. It calculates the offset and wrap offset (the offset where (0, 0) begins, everything before that is wrapped to the end). Then the algorithm calculates which cells should change and calls the supplied callback for each of those cells.

Here are a couple examples in practice

```rust
let mut grid = RollGrid2D::new(4, 4, (0, 0), |pos: (i32, i32)| {
    pos
});
println!("Initial grid:");
print_grid(&grid);
let mut iterations = 0;
let mut changes = vec![];
grid.reposition((1, 2), |old, new, cell_mut| {
    iterations += 1;
    changes.push((old, new));
    *cell_mut = new;
});
println!("Changes:");
for (old, new) in changes {
    println!("{old:?} moved to {new:?}");
}
println!("Grid repositioned to (1, 2) with {iterations} iterations:");
print_grid(&grid);
println!("Cell at (4, 5): {:?}", grid.get_copy((4, 5)).unwrap());
println!("Cell at (0, 0): {:?}", grid.get_copy((0, 0)));
```
Output:
```
Initial grid:
[
    [( 0,  0), ( 1,  0), ( 2,  0), ( 3,  0)]
    [( 0,  1), ( 1,  1), ( 2,  1), ( 3,  1)]
    [( 0,  2), ( 1,  2), ( 2,  2), ( 3,  2)]
    [( 0,  3), ( 1,  3), ( 2,  3), ( 3,  3)]
]
Changes:
(0, 2) moved to (4, 2)
(0, 3) moved to (4, 3)
(1, 0) moved to (1, 4)
(2, 0) moved to (2, 4)
(3, 0) moved to (3, 4)
(1, 1) moved to (1, 5)
(2, 1) moved to (2, 5)
(3, 1) moved to (3, 5)
(0, 0) moved to (4, 4)
(0, 1) moved to (4, 5)
Grid repositioned to (1, 2) with 10 iterations:
[
    [( 1,  2), ( 2,  2), ( 3,  2), ( 4,  2)]
    [( 1,  3), ( 2,  3), ( 3,  3), ( 4,  3)]
    [( 1,  4), ( 2,  4), ( 3,  4), ( 4,  4)]
    [( 1,  5), ( 2,  5), ( 3,  5), ( 4,  5)]
]
Cell at (4, 5): (4, 5)
Cell at (0, 0): None
```

One more example, a little more advanced:

```rust
chunks.reposition((chunk_x, chunk_z), |old_pos, (x, z), chunk| {
    self.unload_chunk(&mut chunk);
    chunk.block_offset = Coord::new(x * 16, WORLD_BOTTOM, z * 16);
    if let Some(region) = regions.get_mut((x >> 5, z >> 5)) {
        let result = region.read((x & 31, z & 31), |reader| {
            chunk.read_from(reader, self)
        });
        match result {
            Err(Error::ChunkNotFound) => (/* Do nothing, that just means it's an empty chunk */),
            Err(err) => {
                panic!("Error: {err}");
            }
            _ => (),
        }
        chunk.edit_time = region.get_timestamp((x & 31, z & 31));
    }
    chunk.block_offset.x = x * 16;
    chunk.block_offset.z = z * 16;
});
```

This `reposition` method works for the 2d and 3d variants of the rollgrid.
You can modify this code to fit your purpose.

# Changelog

### 3.0.0-alpha1
##### Big Fixes

##### Additions
- Added `Grid2D` and `Grid3D`.
- Added `math` module, for some useful math functionality that this crate uses.
- Added `new_zst` constructor to `RollGrid2D<()>` and `RollGrid3D<()>`.
- Added `new_1d` and `try_new_1d` functions to `FixedArray`.
- Added `into_raw` and `from_raw` to `FixedArray`.
- Implemented `Send` for `RollGrid2D<T>` where `T: Send`.
- Implemented `Send` for `RollGrid3D<T>` where `T: Send`.
- Implemented `Sync` for `RollGrid2D<T>` where `T: Sync`.
- Implemented `Sync` for `RollGrid3D<T>` where `T: Sync`.
- Implemented `FromIterator` for `FixedArray`.
- Implemented `From<Vec<T>>` for `FixedArray<T>`.
- Implemented `From<Box<[T]>>` for `FixedArray<T>`.
- Implemented `Clone` for `FixedArray<T>` where `T: Clone`.
- Implemented `Clone` for `RollGrid2D<T>` where `T: Clone`.
- Implemented `Clone` for `RollGrid3D<T>` where `T: Clone`.
- Implemented `AsRef<FixedArray<T>>` for `FixedArray<T>`.
- Implemented `AsMut<FixedArray<T>>` for `FixedArray<T>`.
- Implemented `AsRef<[T]>` for `FixedArray<T>`.
- Implemented `AsMut<[T]>` for `FixedArray<T>`.
- Implemented `Borrow<[T]>` for `FixedArray<T>`.
- Implemented `BorrowMut<[T]>` for `FixedArray<T>`.
##### Changes
- Modified `RollGrid2D` and `RollGrid3D` to use `u32` for dimensions rather than `usize`. This makes it more clear what types of values should be used.
- Reset `capacity` in `FixedArray::internal_dealloc` instead of `FixedArray::dealloc`.
- Modified `FixedArray::into_raw` to be a static function rather than a method.
### 2.0.0

- Changed the internal representation of the cells in `RollGrid2D` and `RollGrid3D` from `Box<[Option<T>]>` to `rollgrid::cells::FixedArray<T>`. `FixedArray` is an internal type that was created to fulfill the needs of this crate.
- Removed generic coordinate parameters. Coordinate arguments must be explicitly `(i32, i32)` for `RollGrid2D` and `(i32, i32, i32)` for `RollGrid3D`.
- For `reposition` functions, the `reload` callback now takes `&mut T` rather than `Option<T>` and returns `()` instead of `Option<T>`.
- For resize functions, the `manage` parameter is now a generic of the trait `CellManage<C, T>`, which separates the functionality of `load`, `reload`, and `unload`. There is also `TryCellManage<C, T, E>` for the fallible resize functions.
- Removed `get_opt`, `get_opt_mut`, and `set_opt`.
- Removed `get_or_insert` and `get_or_insert_with`.
- Removed `take`.
- Changed the `Item` for `RollGridXDIterator` and `RollGridXDMutIterator`. Now returns `&T`/`&mut T` rather than `Option<&T>`/`Option<&mut T>`.
- Replaced the `new` constructor for `RollGrid2D` and `RollGrid3D` with the `new_with_init` constructor. `new_with_init` is now called `new` and the original `new` no longer exists. Likewise changed the name of `try_new_with_init` to `try_new`.
- In `RollGrid2D`, changed the inflate/deflate to use both width and height rather than inflating/deflating both simultaneously. This is to ensure parity with `RollGrid3D`.