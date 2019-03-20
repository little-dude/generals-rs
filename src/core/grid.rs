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
            tiles: (0..nb_tiles).map(factory).collect(),
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

    fn index(&self, column: usize, line: usize) -> usize {
        column + line * self.width
    }

    pub fn is_valid_index(&self, i: usize) -> bool {
        i < self.width * self.height
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
