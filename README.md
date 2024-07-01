Implementation of a rolling grid. A type that stores values in a 2D grid and can translate the grid to a new location. This is ideal for pseudo-infinite worlds where you want a buffer to store a region of chunks that can be moved around. Moving the region (and setting new values) is O(n) where n is the number of cells that would change position during the move operation. The move operation will call a callback with the old position, the new position, and the old value and the callback is expected to return the new value. This functionality allows you to move the grid and save any cells being unloaded and load the new cell at the same time.
The move operation can be thought of as a 2D ring buffer.
If it were 1D, a move operation might look like this:
```
Offset: 1
   old: [0, 1, 2, 3, 4]
   new: [4, 0, 1, 2, 3]
```
Since the 4 was moved to the front of the buffer, it needs to be reloaded, so the move operation will call reload:
```
reload(4, -1, Some(4))
```

Sorry for the poor explanation, I'll work on the readme eventually.