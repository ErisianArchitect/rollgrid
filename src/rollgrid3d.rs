
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