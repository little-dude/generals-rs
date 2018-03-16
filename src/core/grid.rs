use std::slice::Iter;

#[derive(Debug)]
pub struct Grid<T> {
    tiles: Vec<T>,
    width: usize,
    height: usize,
}

impl<T> Grid<T> {
    pub fn new<F>(factory: F, width: usize, height: usize) -> Self
    where
        F: Fn(usize) -> T,
    {
        let nb_tiles = width * height;
        Grid {
            tiles: (0..nb_tiles).into_iter().map(factory).collect(),
            width,
            height,
        }
    }

    pub fn manhattan_distance(&self, i1: usize, i2: usize) -> usize {
        let (c1, l1) = self.coordinates(i1);
        let (c2, l2) = self.coordinates(i2);
        let x = if c1 <= c2 { c2 - c1 } else { c1 - c2 };
        let y = if l1 <= l2 { l2 - l1 } else { l1 - l2 };
        x + y
    }

    pub fn tiles(&self) -> &[T] {
        self.tiles.as_ref()
    }

    pub fn get(&self, index: usize) -> &T {
        &self.tiles()[index]
    }

    pub fn tiles_mut(&mut self) -> &mut [T] {
        self.tiles.as_mut()
    }

    pub fn get_mut(&mut self, index: usize) -> &mut T {
        &mut self.tiles_mut()[index]
    }

    fn index(&self, column: usize, line: usize) -> usize {
        column + line * self.width
    }

    pub fn is_valid_index(&self, i: usize) -> bool {
        i >= 0 && i < self.width * self.height
    }

    fn coordinates(&self, i: usize) -> (usize, usize) {
        (self.column(i), self.line(i))
    }

    fn column(&self, i: usize) -> usize {
        i % self.width
    }

    fn line(&self, i: usize) -> usize {
        i / self.width
    }

    pub fn up_left(&self, index: usize) -> Option<usize> {
        if !self.is_valid_index(index) {
            return None;
        }
        let (column, line) = self.coordinates(index);
        if column == 0 || line == 0 {
            return None;
        }
        Some(self.index(column - 1, line - 1))
    }

    pub fn up(&self, index: usize) -> Option<usize> {
        if !self.is_valid_index(index) {
            return None;
        }
        let (column, line) = self.coordinates(index);
        if line == 0 {
            return None;
        }
        Some(self.index(column, line - 1))
    }

    pub fn up_right(&self, index: usize) -> Option<usize> {
        if !self.is_valid_index(index) {
            return None;
        }
        let (column, line) = self.coordinates(index);
        if column == self.width() - 1 || line == 0 {
            return None;
        }
        Some(self.index(column + 1, line - 1))
    }

    pub fn left(&self, index: usize) -> Option<usize> {
        if !self.is_valid_index(index) {
            return None;
        }
        let (column, line) = self.coordinates(index);
        if column == 0 {
            return None;
        }
        Some(self.index(column - 1, line))
    }

    pub fn right(&self, index: usize) -> Option<usize> {
        if !self.is_valid_index(index) {
            return None;
        }
        let (column, line) = self.coordinates(index);
        if column == self.width() - 1 {
            return None;
        }
        Some(self.index(column + 1, line))
    }

    pub fn down_left(&self, index: usize) -> Option<usize> {
        if !self.is_valid_index(index) {
            return None;
        }
        let (column, line) = self.coordinates(index);
        if column == 0 || line == self.height() - 1 {
            return None;
        }
        Some(self.index(column - 1, line + 1))
    }

    pub fn down(&self, index: usize) -> Option<usize> {
        if !self.is_valid_index(index) {
            return None;
        }
        let (column, line) = self.coordinates(index);
        if line == self.height() - 1 {
            return None;
        }
        Some(self.index(column, line + 1))
    }

    pub fn down_right(&self, index: usize) -> Option<usize> {
        if !self.is_valid_index(index) {
            return None;
        }
        let (column, line) = self.coordinates(index);
        if column == self.width() - 1 || line == self.height() - 1 {
            return None;
        }
        Some(self.index(column + 1, line + 1))
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn len(&self) -> usize {
        self.width * self.height
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn direct_neighbors(&self, index: usize) -> DirectNeighborsIter {
        let neighbors = [
            self.up(index),
            self.left(index),
            self.right(index),
            self.down(index),
        ];
        DirectNeighborsIter::new(neighbors)
    }

    pub fn extended_neighbors(&self, index: usize) -> ExtendedNeighborsIter {
        let neighbors = [
            self.up_left(index),
            self.up(index),
            self.up_right(index),
            self.left(index),
            self.right(index),
            self.down_left(index),
            self.down(index),
            self.down_right(index),
        ];
        ExtendedNeighborsIter::new(neighbors)
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.tiles().iter()
    }
}

pub struct DirectNeighborsIter {
    count: usize,
    neighbors: [Option<usize>; 4],
}

impl DirectNeighborsIter {
    fn new(neighbors: [Option<usize>; 4]) -> Self {
        DirectNeighborsIter {
            count: 0,
            neighbors,
        }
    }
}

impl Iterator for DirectNeighborsIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.count < 4 {
            if let Some(neighbor) = self.neighbors[self.count] {
                self.count += 1;
                return Some(neighbor);
            } else {
                self.count += 1;
            }
        }
        None
    }
}

pub struct ExtendedNeighborsIter {
    count: usize,
    neighbors: [Option<usize>; 8],
}

impl ExtendedNeighborsIter {
    fn new(neighbors: [Option<usize>; 8]) -> Self {
        ExtendedNeighborsIter {
            count: 0,
            neighbors,
        }
    }
}

impl Iterator for ExtendedNeighborsIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.count < 8 {
            if let Some(neighbor) = self.neighbors[self.count] {
                self.count += 1;
                return Some(neighbor);
            } else {
                self.count += 1;
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    // 0  1  2
    // 3  4  5
    // 6  7  8
    // 9  10 11
    use super::*;
    #[test]
    fn test_grid() {
        let grid = Grid::<u8>::new(|idx| idx as u8, 3, 4);
        assert_eq!(grid.width(), 3);
        assert_eq!(grid.height(), 4);
        assert_eq!(grid.len(), 12);
        assert!(!grid.is_empty());
        assert_eq!(grid.tiles(), &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    }

    #[test]
    fn test_indices() {
        let grid = Grid::<u8>::new(|idx| idx as u8, 3, 4);
        for i in 0..=11 {
            println!("checking that {} is a valid index", i);
            assert!(grid.is_valid_index(i));
        }
        assert!(!grid.is_valid_index(12));
    }

    #[test]
    fn test_directions() {
        let grid = Grid::<u8>::new(|idx| idx as u8, 3, 4);
        assert!(grid.up(0).is_none());
        assert_eq!(grid.up(11), Some(8));

        assert!(grid.down(10).is_none());
        assert_eq!(grid.down(1), Some(4));

        assert!(grid.left(3).is_none());
        assert_eq!(grid.left(7), Some(6));

        assert!(grid.right(2).is_none());
        assert_eq!(grid.right(7), Some(8));

        assert_eq!(grid.right(0), Some(1));
        assert_eq!(grid.down(0), Some(3));
    }

    // 0  1  2
    // 3  4  5
    // 6  7  8
    // 9  10 11
    //
    // We go up/left/right/diwn for neighbors
    #[test]
    fn test_direct_neighbors() {
        let grid = Grid::<u8>::new(|idx| idx as u8, 3, 4);

        let neighbors: Vec<usize> = grid.direct_neighbors(0).collect();
        assert_eq!(neighbors.as_slice(), &[1, 3]);

        let neighbors: Vec<usize> = grid.direct_neighbors(7).collect();
        assert_eq!(neighbors.as_slice(), &[4, 6, 8, 10]);
    }

    // 0  1  2
    // 3  4  5
    // 6  7  8
    // 9  10 11
    //
    // We go up-left/up/up-right/left/right/down-left/down/down-right for neighbors
    #[test]
    fn test_extended_neighbors() {
        let grid = Grid::<u8>::new(|idx| idx as u8, 3, 4);

        let neighbors: Vec<usize> = grid.extended_neighbors(0).collect();
        assert_eq!(neighbors.as_slice(), &[1, 3, 4]);

        let neighbors: Vec<usize> = grid.extended_neighbors(7).collect();
        assert_eq!(neighbors.as_slice(), &[3, 4, 5, 6, 8, 9, 10, 11]);
    }
}
