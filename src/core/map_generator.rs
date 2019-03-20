//! This module contains code to generate random grids.
//!
//! Grids have variable sizes, based on the number of players. Generals are spawned randomly, but
//! with a minimum manhattan distance between each other.
//!
//! Finally, the topologies are random, but there is a least one open path between the generals.
use std::cell::RefCell;

use fera_unionfind::UnionFindRange;
use rand::{rngs::ThreadRng, thread_rng, Rng};

use super::common::Tile;
use super::grid::Grid;

const MIN_DISTANCE: usize = 10;
const MIN_GRID_SIZE: usize = 17;
const GRID_SIZE_MAX_DELTA: usize = 6;

/// A temporary datastructure used to generate a random grid.
#[derive(Debug)]
pub struct GridBuilder {
    grid: Grid<RefCell<Tile>>,
    rng: ThreadRng,
    generals: Vec<usize>,
    nb_generals: usize,
}

impl GridBuilder {
    /// Return a new builder. The grid dimensions are random but are related to the number of
    /// generals: more generals mean bigger grid.
    pub fn new(nb_generals: usize) -> Self {
        let mut rng = thread_rng();
        let width = MIN_GRID_SIZE + nb_generals + rng.gen_range(0, GRID_SIZE_MAX_DELTA + 1);
        let height = MIN_GRID_SIZE + nb_generals + rng.gen_range(0, GRID_SIZE_MAX_DELTA + 1);

        GridBuilder {
            generals: Vec::new(),
            grid: Grid::new(|_| RefCell::new(Tile::new()), width, height),
            rng,
            nb_generals,
        }
    }

    /// Return whether a given cell on the grid is open (ie is not a mountain or a city).
    fn is_open(&self, index: usize) -> bool {
        let tile = self.grid.get(index).borrow();
        tile.is_open() || tile.is_general()
    }

    /// Create a new grid with only closed tiles and the generals. Then, keep opening tiles until
    /// all the generals are connected. Finally, the grid.
    pub fn build(mut self) -> (Vec<usize>, Grid<RefCell<Tile>>) {
        let nb_tiles = self.grid.len();
        let mut uf = UnionFindRange::with_keys_in_range(..nb_tiles);

        debug!("generating grid");
        'outer: loop {
            // Pick a random tile
            let index = self.rng.gen_range(0, nb_tiles);
            // If that tile is already open, ignore it and pick another one
            {
                let tile = self.grid.get(index).borrow();
                if tile.is_open() || tile.is_general() {
                    continue;
                }
            }
            // Open the tile, and connect it to its neighbors that are already open
            self.grid.get(index).borrow_mut().make_open();
            for i in self.grid.direct_neighbors(index) {
                if self.is_open(i) && !uf.in_same_set(index, i) {
                    uf.union(index, i);
                }
            }

            // If not all the generals have been spawned, and if the tile is far enough from all
            // the other generals, make it a general.
            if self.generals.len() < self.nb_generals {
                for general in &self.generals {
                    if self.grid.manhattan_distance(index, *general) < MIN_DISTANCE {
                        continue 'outer;
                    }
                }
                info!("making {} a general", index);
                self.grid.get(index).borrow_mut().make_general();
                self.generals.push(index);
                continue;
            }

            // Check that all the generals are connected. If so, we're done.
            //
            // To check whether all the generals are connected, we check that they are all
            // connected to the first one.
            let first_general = self.generals[0];
            for general in self.generals.iter().skip(1) {
                if !uf.in_same_set(first_general, *general) {
                    continue 'outer;
                }
            }

            debug!("successfully generated a grid");
            // For debugging print the generated grid
            for (idx, tile) in self.grid.tiles().iter().enumerate() {
                debug!("index {}: {:?}", idx, tile);
            }
            return (self.generals, self.grid);
        }
    }
}
