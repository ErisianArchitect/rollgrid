Implementation of a rolling grid. A type that stores values in a 2D grid and can translate the grid to a new location. This is ideal for pseudo-infinite worlds where you want a buffer to store a region of chunks that can be moved around. Moving the region (and setting new values) is O(n) where n is the number of cells that would change position during the move operation. The move operation will call a callback with the old position, the new position, and the old value and the callback is expected to return the new value. This functionality allows you to move the grid and save any cells being unloaded and load the new cell at the same time.
The move operation can be thought of as a 2D ring buffer.
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
If you were to translate it by the offset `(1, 1)`, the result would be this grid:
```
5674
9AB8
DEFC
1230
```

Now, that's not the benefit of this library. When repositioning the grid, it doesn't move any elements at all. Instead, it keeps track of movement offsets and allows the reposition operation to be really fast because it's not actually moving anything around in memory. It calculates the offset and wrap offset (the offset where (0, 0) begins, everything before that is wrapped to the end). Then the algorithm calculates which cells should change and calls the supplied callback for each of those cells.

Here's an example in practice

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
(todo: Add a try_reposition function)

This `reposition` method works for the 2d and 3d variants of the rollgrid.

You can modify this code to fit your purpose.