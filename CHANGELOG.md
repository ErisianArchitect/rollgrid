### 3.1.0
##### Bug Fixes
- Fixed incorrect comparison operator in internal function that would cause a panic in debug mode under certain circumstances.
##### Additions
- Implemented `Serialize` and `Deserialize` for `FixedArray`, `Grid2D`, `Grid3D`, `RollGrid2D`, and `RollGrid3D`.
##### Changes
- Renamed `grid_offset` to `offset`.
### 3.0.0
##### Bug Fixes
- Integer overflows in as many places as I could find.
- Major bug that caused the grids to become invalidated during `resize_and_reposition` operation.
##### Additions
- Added `Grid2D` and `Grid3D`.
- Added `math` module, for some useful math functionality that this crate uses.
- Added `new_zst` constructor to `RollGrid2D<()>` and `RollGrid3D<()>`.
- Added `new_1d` and `try_new_1d` functions to `FixedArray`.
- Added `into_raw` and `from_raw` to `FixedArray`.
- Added `subgrid`, `subgrid_mut`, `copy_subgrid`, and `clone_subgrid` to `RollGrid2D` and `RollGrid3D`.
- Implemented `Send` for `RollGrid2D<T>` where `T: Send`.
- Implemented `Send` for `RollGrid3D<T>` where `T: Send`.
- Implemented `Sync` for `RollGrid2D<T>` where `T: Sync`.
- Implemented `Sync` for `RollGrid3D<T>` where `T: Sync`.
- Implemented `Clone` for `FixedArray<T>` where `T: Clone`.
- Implemented `Clone` for `RollGrid2D<T>` where `T: Clone`.
- Implemented `Clone` for `RollGrid3D<T>` where `T: Clone`.
- Implemented `FromIterator` for `FixedArray`.
- Implemented `From<Vec<T>>` for `FixedArray<T>`.
- Implemented `From<Box<[T]>>` for `FixedArray<T>`.
- Implemented `Index<(i32, i32)>` for `RollGrid2D<T>`.
- Implemented `IndexMut<(i32, i32)>` for `RollGrid2D<T>`.
- Implemented `Index<(i32, i32, i32)>` for `RollGrid3D<T>`.
- Implemented `IndexMut<(i32, i32, i32)>` for `RollGrid3D<T>`.
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
- `relative_offset` now returns `(i64, ...)` instead of `(i32, ...)`.
- Operations that took dimensions as individual arguments now take them as tuples.
- `read` in `RollGrid2D` and `RollGrid3D` now returns `T` instead of `Option<T>` and panics if the coordinate is out of bounds.
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