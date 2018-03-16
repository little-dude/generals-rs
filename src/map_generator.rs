use std::cmp::min;
use std::cell::RefCell;
use grid::Coordinates;
use rand::seq::{sample_indices, sample_iter};
use rand::{thread_rng, ThreadRng};
use fera_unionfind::UnionFindRange;
use game;

#[cfg(test)]
use std::fmt;

// In this module, we use the manhattan distance to measure the distance between two tiles.  This
// function returns the manhattan distance between to tiles. It's not used but let's keep it around
// just for a while just in case.
//
// fn manhattan_distance(c1: Coordinates, c2: Coordinates) -> usize {
//     let x = if c1.x <= c2.x {
//         c2.x - c1.x
//     } else {
//         c1.x - c2.x
//     };
//     let y = if c1.y <= c2.y {
//         c2.y - c1.y
//     } else {
//         c1.y - c2.y
//     };
//     x + y
// }

const MIN_DISTANCE: usize = 10;
const MIN_GRID_SIZE: usize = 17;
const GRID_SIZE_MAX_DELTA: usize = 6;

struct Generator {
    /// Width of the map
    width: usize,
    /// Height of the map
    height: usize,
    /// A random number generator
    rng: ThreadRng,
    /// Number of generals to spawn on the map
    nb_generals: usize,
    /// Coordinates of the generals on the map
    generals: Vec<usize>,
    /// The generated map
    map: Vec<Tile>,
}

impl Generator {
    fn coord_to_index(&self, x: usize, y: usize) -> usize {
        x + y * self.width
    }

    fn index_to_coord(&self, i: usize) -> (usize, usize) {
        (i % self.width, i / self.width)
    }

    fn init(nb_generals: usize) -> Self {
        let mut rng = thread_rng();

        // Generate random dimensions based on the number of generals. Here is how we compute the
        // map dimensions:
        //
        // +------------+--------------+
        // | nb players | sided length |
        // +------------+--------------+
        // |    2       |   20 +/- 3   |
        // |    3       |   21 +/- 3   |
        // |    4       |   22 +/- 3   |
        // |    5       |   23 +/- 3   |
        // |    6       |   24 +/- 3   |
        // |    7       |   25 +/- 3   |
        // |    8       |   26 +/- 3   |
        // +------------+--------------+
        let dimensions = sample_indices(&mut rng, GRID_SIZE_MAX_DELTA + 1, 2);
        let width = MIN_GRID_SIZE + nb_generals + dimensions[0];
        let height = MIN_GRID_SIZE + nb_generals + dimensions[1];

        // create a map with only walls
        let nb_tiles = width * height;
        let map = (0..nb_tiles).into_iter().map(Tile::new).collect();

        Generator {
            nb_generals,
            generals: Vec::with_capacity(nb_generals),
            height,
            width,
            rng,
            map,
        }
    }

    /// Randomly spawn the generals on the map. The distance between two generals is at least
    /// `MIN_DISTANCE`.
    fn spawn_generals(&mut self) {
        let mut available_tiles: Vec<usize> = (0..self.width * self.height).into_iter().collect();
        for _ in 0..self.nb_generals {
            if available_tiles.is_empty() {
                // FIXME: we could return an error but I'd rather try to see whether this can
                // happen or not. But maths are hard...
                panic!("Grid is too small, cannot spawn all the generals");
            }
            let index = *sample_iter(&mut self.rng, &available_tiles, 1).unwrap()[0];
            self.generals.push(index);
            self.map[index].kind = TileKind::General;
            self.remove_reserved_tiles(index, &mut available_tiles);
        }
    }

    /// To ensure that the distance between two generals is at least `MIN_DISTANCE`, we keep a list
    /// of tiles where new generals can be spawned. This function updates this list of available
    /// tiles after we spawned a general.
    fn remove_reserved_tiles(&self, new_general_idx: usize, available_tiles: &mut Vec<usize>) {
        // Since we use the manhattan distance, when spawning a general (G) the following tiles (X)
        // are removed from the available tiles (in this diagram, MIN_DISTANCE == 3):
        //
        //   --------X------------> start_row
        //         X X X
        //       X X X X X
        //     X X X G X X X
        //       X X X X X
        //         X X X
        //   --------X------------> end_row
        let (x, y) = self.index_to_coord(new_general_idx);

        let start_row = if y < MIN_DISTANCE {
            MIN_DISTANCE - y
        } else {
            0
        };
        let end_row = min(self.height - 1, y + MIN_DISTANCE);

        let offset = 0;
        for row in start_row..=end_row {
            let start = if x < MIN_DISTANCE - offset {
                MIN_DISTANCE - offset - x
            } else {
                0
            };
            let end = min(self.width - 1, x + offset);
            for tile_idx in self.coord_to_index(start, row)..=self.coord_to_index(end, row) {
                // The tile might have already been removed, so we don't check the result of the
                // binary search here.
                let _ = available_tiles
                    .binary_search(&tile_idx)
                    .map(|idx| available_tiles.remove(idx));
            }
        }
    }

    /// Create a random map in which there's a path between all the generals
    fn generate_map(&mut self) {
        let nb_tiles = self.width * self.height;
        // initialize the union-find type we use to check whether all the generals are connected
        let mut uf = UnionFindRange::with_keys_in_range(..nb_tiles);
        // keep one general around, so that we can check if the others are connected to it
        let general_idx = self.generals[0];

        // turn walls into an open tiles until all the generals are connected
        'outer: for index in sample_indices(&mut self.rng, nb_tiles, nb_tiles) {
            // open a random tile
            self.map[index].open();

            // get the neighbors that are open, and connect them to the tile we just opened
            let open_neighbors = self.get_neighbors_indices(index)
                .into_iter()
                .filter_map(|opt_idx| opt_idx.map(|i| self.map[i]))
                .filter(|tile| tile.is_open());
            for neighbor in open_neighbors {
                if !uf.in_same_set(index, neighbor.index) {
                    uf.union(index, neighbor.index);
                }
            }

            // if all the generals are connected, we're done
            for general in self.generals.iter().skip(1) {
                if !uf.in_same_set(general_idx, *general) {
                    continue 'outer;
                }
            }
            break;
        }
        // TODO: fortresses and stuff
    }

    fn build_grid(&self) -> game::Grid {
        let mut grid: Vec<Vec<game::Tile>> = Vec::with_capacity(self.height);
        let mut idx = 0;
        for _ in 0..self.height {
            let mut row: Vec<game::Tile> = Vec::with_capacity(self.width);
            for _ in 0..self.width {
                row.push((&self.map[idx]).into());
                idx += 1;
            }
            grid.push(row);
        }
        game::Grid(grid)
    }

    fn generate(nb_generals: usize) -> (Vec<Coordinates>, game::Grid) {
        let mut generator = Self::init(nb_generals);
        generator.spawn_generals();
        generator.generate_map();
        let grid = generator.build_grid();
        let generals = generator
            .generals
            .iter()
            .map(|idx| Coordinates::from(generator.index_to_coord(*idx)))
            .collect();
        (generals, grid)
    }

    fn get_neighbors_indices(&self, index: usize) -> Vec<Option<usize>> {
        let (x, y) = self.index_to_coord(index);

        let up = if y == 0 {
            None
        } else {
            Some(self.coord_to_index(x, y - 1))
        };

        let right = if x == self.width - 1 {
            None
        } else {
            Some(self.coord_to_index(x + 1, y))
        };

        let down = if y == self.height - 1 {
            None
        } else {
            Some(self.coord_to_index(x, y + 1))
        };

        let left = if x == 0 {
            None
        } else {
            Some(self.coord_to_index(x - 1, y))
        };

        vec![up, right, down, left]
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum TileKind {
    General,
    Fortress(usize),
    Wall,
    Open,
}

#[derive(Copy, Clone, PartialEq, Debug)]
struct Tile {
    index: usize,
    kind: TileKind,
}

impl Tile {
    fn new(index: usize) -> Self {
        Tile {
            index,
            kind: TileKind::Wall,
        }
    }

    fn is_open(&self) -> bool {
        match self.kind {
            TileKind::General | TileKind::Open => true,
            _ => false,
        }
    }

    fn open(&mut self) {
        if self.kind == TileKind::General {
            return;
        }
        self.kind = TileKind::Open;
    }
}

#[cfg(test)]
impl fmt::Display for TileKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            TileKind::General => "G",
            TileKind::Wall => "█",
            TileKind::Open => " ",
            TileKind::Fortress(_) => F,
        };
        write!(f, "{}", s)
    }
}

impl<'a> Into<game::Tile> for &'a Tile {
    fn into(self) -> game::Tile {
        match self.kind {
            TileKind::Wall => None,
            TileKind::General => Some(RefCell::new(game::OpenTile::new(game::TileKind::General))),
            TileKind::Open => Some(RefCell::new(game::OpenTile::new(game::TileKind::Normal))),
            TileKind::Fortress(_) => {
                Some(RefCell::new(game::OpenTile::new(game::TileKind::Fortress)))
            }
        }
    }
}
