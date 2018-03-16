use std::collections::HashMap;
use std::cell::{Ref, RefCell, RefMut};
use grid::Coordinates;

#[derive(Clone, Debug, PartialEq)]
pub enum TileKind {
    General,
    Fortress,
    Normal,
}

type PlayerId = usize;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    Right,
    Left,
    Up,
    Down,
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct Capture(PlayerId, PlayerId);

#[derive(Copy, Clone, Debug)]
pub struct Move {
    player: PlayerId,
    from: Coordinates,
    direction: Direction,
}

#[derive(Clone, Debug)]
pub struct OpenTile {
    owner: Option<PlayerId>,
    units: u16,
    kind: TileKind,
}

impl OpenTile {
    pub fn new(kind: TileKind) -> Self {
        OpenTile {
            owner: None,
            units: 0,
            kind,
        }
    }
}

pub type Tile = Option<RefCell<OpenTile>>;

#[derive(Clone, Debug)]
pub struct Grid(pub Vec<Vec<Tile>>);

impl Grid {
    pub fn new(height: usize, width: usize) -> Self {
        let grid = Grid(vec![vec![None; width]; height]);
        // TODO: populate the grid:
        //  - add a general for each player
        //  - make the grid percolate
        //  - add random cities
        grid
    }

    /// Check whether a given move starts and finishes within the grid bounds.
    fn move_is_within_grid(&self, mv: Move) -> bool {
        let Move {
            from, direction, ..
        } = mv;
        !(
            // It's invalid to move up from the top row
            (from.y == 0 && direction == Direction::Up)
            // It's invalid to move down from the bottom row
            || (from.y >= self.0.len() - 1 && direction == Direction::Down)
            // It's invalid to move left from the left-most column
            || (from.x == 0 && direction == Direction::Left)
            // It's invalid to move right from the right-most column
            || (from.x >= self.0[0].len() - 1 && direction == Direction::Right))
    }

    /// Give a move, return the tile corresponding to the destination of the move.
    fn get_destination_tile(&self, mv: Move) -> Option<&RefCell<OpenTile>> {
        let Move {
            from, direction, ..
        } = mv;
        let coordinates = match direction {
            Direction::Right => Coordinates::new(from.x + 1, from.y),
            Direction::Left => Coordinates::new(from.x - 1, from.y),
            Direction::Up => Coordinates::new(from.x, from.y - 1),
            Direction::Down => Coordinates::new(from.x, from.y + 1),
        };
        self.get_tile(coordinates)
    }

    /// Return the tile at the given coordinates.
    fn get_tile(&self, coords: Coordinates) -> Option<&RefCell<OpenTile>> {
        if coords.y < self.0.len() {
            let row = &self.0[coords.y];
            if coords.x < row.len() {
                return row[coords.x].as_ref();
            }
        }
        None
    }

    /// Update the tiles involved in a move.
    fn compute_move(&mut self, mv: Move) -> Option<Capture> {
        if !self.move_is_within_grid(mv) {
            return None;
        }

        if let Some(src_tile) = self.get_tile(mv.from) {
            let src_tile = src_tile.borrow_mut();

            if src_tile.owner != Some(mv.player) || src_tile.units < 2 {
                // If the tile does not belong to the player making the move, do nothing since the
                // move is illegal.
                //
                // If the source tile has less than 2 units, do nothing since we cannot leave the
                // tile empty or with a negative number of units.
                return None;
            }

            if let Some(dst_tile) = self.get_destination_tile(mv) {
                let mut src_tile = src_tile;
                let mut dst_tile = dst_tile.borrow_mut();

                if dst_tile.owner == Some(mv.player) {
                    // The owner is the same for both tiles, just transfer the units
                    dst_tile.units += src_tile.units - 1;
                } else {
                    // The owner is different. Let's fight!
                    if dst_tile.units >= src_tile.units - 1 {
                        // The other player has more units. Just Decrement the number of units.
                        dst_tile.units -= src_tile.units - 1;
                    } else {
                        // We have more units, we conquer the tile.

                        if dst_tile.kind == TileKind::General {
                            // We're capturing a general, that requires special handling.
                            dst_tile.units = src_tile.units - 1 - dst_tile.units;
                            src_tile.units = 1;
                            dst_tile.kind = TileKind::Fortress;
                            assert!(dst_tile.owner.is_some()); // normally a general tile is owned by a player
                            return Some(Capture(mv.player, dst_tile.owner.unwrap()));
                        }

                        dst_tile.units = src_tile.units - 1 - dst_tile.units;
                        dst_tile.owner = src_tile.owner;
                    }
                }
                // In any case, we always only leave 1 unit in the source tile
                // TODO: would be nice to support splitting the source tile units before moving.
                src_tile.units = 1;
            }
        }
        None
    }

    fn capture_general(&mut self, capture: &Capture) {
        for mut tile in self.into_iter()
            .filter(|tile| tile.owner == Some(capture.1))
        {
            tile.owner = Some(capture.0)
        }
    }

    fn compute_moves(&mut self, moves: Vec<Move>, defeated_players: &mut Vec<PlayerId>) {
        for mv in moves {
            if defeated_players.binary_search(&mv.player).is_ok() {
                // This player has been defeated. Ignore his move.
                continue;
            }
            if let Some(capture) = self.compute_move(mv) {
                self.capture_general(&capture);
                // Lookup where we should insert the defeated player to maintain a sorted
                // `defeated_players` vector. `Vec::binary_search()` returns the index where the
                // missing item should be inserted to maintain sorted order.
                let insert_at = defeated_players.binary_search(&capture.1).unwrap_err();
                defeated_players.insert(insert_at, capture.1);
            }
        }
    }

    fn reinforce(&mut self, reinforce_all_tiles: bool) {
        for mut tile in self {
            match tile.kind {
                TileKind::Normal => if reinforce_all_tiles {
                    tile.units += 1;
                },
                TileKind::General => tile.units += 1,
                TileKind::Fortress => if tile.owner.is_some() {
                    tile.units += 1;
                },
            }
        }
    }
}

struct GridIterator<'a> {
    grid: &'a Grid,
    coordinates: Coordinates,
}

impl<'a> GridIterator<'a> {
    fn increment_coordinates(&mut self) {
        if self.coordinates.x == self.grid.0[self.coordinates.y].len() {
            // we finished the current row, start a new one
            self.coordinates.y += 1;
            self.coordinates.x = 0;
        } else {
            // we didn't finish the current row, just increment the horizontal coordinate
            self.coordinates.x += 1;
        }
    }
}

impl<'a> Iterator for GridIterator<'a> {
    type Item = &'a RefCell<OpenTile>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(tile) = self.grid.get_tile(self.coordinates) {
                self.increment_coordinates();
                return Some(tile);
            } else if self.coordinates.y >= self.grid.0.len() {
                return None;
            } else {
                self.increment_coordinates();
            }
        }
    }
}

pub struct Iter<'a>(GridIterator<'a>);

impl<'a> Iterator for Iter<'a> {
    type Item = Ref<'a, OpenTile>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|tile| tile.borrow())
    }
}

pub struct IterMut<'a>(GridIterator<'a>);

impl<'a> Iterator for IterMut<'a> {
    type Item = RefMut<'a, OpenTile>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|tile| tile.borrow_mut())
    }
}

impl<'a> IntoIterator for &'a mut Grid {
    type Item = RefMut<'a, OpenTile>;
    type IntoIter = IterMut<'a>;
    fn into_iter(self) -> Self::IntoIter {
        IterMut(GridIterator {
            grid: self,
            coordinates: Coordinates::new(0, 0),
        })
    }
}

impl<'a> IntoIterator for &'a Grid {
    type Item = Ref<'a, OpenTile>;
    type IntoIter = Iter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        Iter(GridIterator {
            grid: self,
            coordinates: Coordinates::new(0, 0),
        })
    }
}

#[derive(Clone, Debug)]
struct Game {
    grid: Grid,
    players: HashMap<PlayerId, Player>,
    defeated_players: Vec<PlayerId>,
    turn: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Player {
    name: String,
}

impl Game {
    fn new(players: Vec<String>) -> Self {
        // FIXME: create a grid with a valid size based on the number of players.
        // Also, how should the grid be generated? With Grid::init(Vec<Generals>) ?
        // Or by the Game type?
        let mut game = Game {
            grid: Grid::new(20, 20),
            players: HashMap::new(),
            turn: 0,
            defeated_players: Vec::new(),
        };
        for (idx, name) in players.into_iter().enumerate() {
            let _ = game.players.insert(idx, Player { name });
        }
        game
    }

    fn give_up(&mut self, player: PlayerId) {
        let _ = self.defeated_players
            .binary_search(&player)
            .map_err(|idx| self.defeated_players.insert(idx, player));
    }

    fn update(&mut self, moves: Vec<Move>) {
        self.turn += 1;
        self.grid.compute_moves(moves, &mut self.defeated_players);
        if self.turn % 25 == 0 {
            self.grid.reinforce(true);
        } else {
            self.grid.reinforce(false);
        }
    }
}
