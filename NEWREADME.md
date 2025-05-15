# Rollgrid
A crate for the 2D and 3D Rollgrid data structures.

## Rollgrid Data Structure
The Rollgrid data structure is a repositionable grid that changes to lookup function to account for repositioning, and provides an O(n) time complexity solution for updating cells that might change during reposition/resize. Rollgrid is represented as a fixed-size array in memory, which is more optimal than a hashmap, which is what is used in the typical implementation of this kind of repositionable grid.