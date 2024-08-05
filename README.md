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
let mut grid = RollGrid2D::new_with_init(4, 4, (0, 0), |pos: (i32, i32)| {
    Some(pos)
});
println!("Initial grid:");
print_grid(&grid);
let mut iterations = 0;
let mut changes = vec![];
grid.reposition((1, 2), |old, new, old_value| {
    iterations += 1;
    changes.push((old, new));
    Some(new)
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
    let mut chunk = chunk.expect("Chunk was None");
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
    Some(chunk)
});
```

New (as of August 5th, 2024):
    All the methods that take callbacks now have fallible versions, so now there's `try_new_with_init`, `try_resize`, `try_inflate_size`, `try_deflate_size`, `try_resize_and_reposition`, `try_translate`, and `try_reposition`.


This `reposition` method works for the 2d and 3d variants of the rollgrid.

You can modify this code to fit your purpose.

# Short Documentation
These functions/methods are found on both RollGrid2D and RollGrid3D.

If `T` is `Default`:
### `new_default`
Creates a new RollGridXD with the specified size and offset.  
This function will panic if the volume of the size is `0` or if it's greater than `i32::MAX`.

### `new`
Creates a new RollGridXD with specified size and offset, but initializes all cells with `None`.  
This function will panic if the volume of the size is `0` or if it's greater than `i32::MAX`.

### `try_new_with_init`
The fallible version of `new`.  
This function will panic if the volume of the size is `0` or if it's greater than `i32::MAX`.

### `get`
Returns the reference to the cell data wrapped in `Option::Some` if the cell data exists and the coordinate is in bounds. Otherwise returns `None`.

### `get_copy` (if `T: Copy`)
Rather than getting a reference to the cell's value, get's a copy of it.

### `get_clone` (if `T: Clone`)
Rather than getting a reference to the cell's value, get's a clone of it.

### `get_mut`
The mutable version of `get`.

### `get_or_insert`
Either gets the cell or inserts a value into it, returns a mutable reference to the cell.

### `get_or_insert_with`
Similar to `get_or_insert`, but instead you supply a callback. This is handy if it would be an expensive operation to use `get_or_insert`.

### `get_opt`
Get a reference to the cell's `Option` (all cells are stored in an `Option<T>`)

### `take`
Take ownership of a cell. This is similar to `Option::take`.

### `set`
Set the value in the cell and return the old value (as an `Option`).

### `set_opt`
Set the cell's internal `Option` value.

### `reposition`
Reposition the grid. This operation will allow you to load in new cells in the area that you move the grid to. The grid simply applies an offset to the grid (without moving anything in memory) and then calculates the cells that need to be updated, then it calls a callback that you supply to it that takes the arguments of (old position, new position, old value), and the return value is the new value.

### `try_reposition`
The fallible version of `reposition`.

### `translate`
Similar to `reposition`, but instead takes a relative offset as its argument and moves the grid by that amount.

### `try_translate`
The fallible version of `translate`.

### `resize_and_reposition`
Resize and reposition the grid.

### `try_resize_and_reposition`
The fallible version of `resize_and_reposition`.

### `resize`
Resize the grid while keeping the offset the same.

### `try_resize`
The fallible version of `resize`.

### `inflate_size`
If you know what an `inflate` operation is on a rectangle type, then you know what this does. It essentially increases the bounds of the grid by the specified amount. So if you have a 2x2 grid and you inflate the size by (1, 1), the resulting grid will have a size of 4x4 and the offset will be (1, 1) less than the previous offset.

### `try_inflate_size`
The fallible version of `inflate_size`.

### `deflate_size`
Similar to the `inflate_size` method, but instead of inflating the size it deflates it.

### `try_deflate_size`
The fallible version of `deflate_size`.

### `relative_offset`
Tells you where a coordinate is in relation to the grid offset. So for example, if your grid offset were (1, 2), and you asked it what the relative offset of (3, 4) was, the answer would be (2, 2).

### `offset`
The offset of the grid. This is the location of the first cell in the grid.

### `size`
Returns `(usize, usize, usize)` that tells you the dimensions of the grid.

### `width`
Tells you the size along the `X` axis.

### `height`
Tells you the size along the `Y` axis.

### `depth`
Tells you the size along the Z axis.

### `x_min`
Tells you the minimum bound on the `X` axis.

### `y_min`
Tells you the minimum bound on the `Y` axis.

### `z_min` (`RollGrid3D`)
Tells you the minimum bound on the `Z` axis.

### `x_max`
Tells you the maximum bound on the `X` axis.

### `y_max`
Tells you the maximum bound on the `Y` axis.

### `z_max` (`RollGrid3D`)
Tells you the maximum bound on the `Z` axis.

### `bounds`
Returns a bounds object (either `Bounds2D` or `Bounds3D`).

### `len`
Returns the number of cells present in the grid.

### `iter`
Iterate the cells in the grid. This gives you tuples with `(position, &Option<T>)`.

### `iter_mut`
The mutable iterator. This unfortunately uses `unsafe` code. I think it's possible to do it without `unsafe`, but I haven't gotten around to that yet.