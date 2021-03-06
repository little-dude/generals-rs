#[cfg(test)]
use std::cell::Ref;
use std::cell::{RefCell, RefMut};

use super::common::{Direction, InvalidMove, Move, MoveOutcome, PlayerId, Tile};
use super::grid::Grid;
use super::map_generator::GridBuilder;

/// A grid representing the game map. It provides interior mutability for the tiles, which means
/// multiple tiles can be borrowed mutable at the same time, without having to borrow mutably the
/// map itself.
#[derive(Debug)]
pub struct Map(Grid<RefCell<Tile>>);

impl Map {
    /// Return a random new map with the specified number of generals.
    pub fn generate(nb_generals: usize) -> (Vec<usize>, Self) {
        let grid_builder = GridBuilder::new(nb_generals);
        let (generals, grid) = grid_builder.build();
        (generals, Map(grid))
    }

    /// The number of tiles on the map
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// The number of tiles in a row
    pub fn width(&self) -> usize {
        self.0.width()
    }

    /// The number of tiles in a column
    pub fn height(&self) -> usize {
        self.0.height()
    }

    #[cfg(test)]
    pub fn from_grid(inner: Grid<RefCell<Tile>>) -> Self {
        Map(inner)
    }

    /// Update the tiles involved in a move.
    ///
    /// - make sure the move is valid
    /// - perform the move, updating the number of units of both the source and destination tile,
    ///   as well as the owner of the destination tile, if it changes as a consequence of the move
    /// - update the visibility of the tiles surrounding the destination tile
    ///
    /// If a general is captured, this method also gives all the tiles that belonged to the
    /// defeated general to the attacker, and updates the visibility of the attacker.
    pub fn perform_move(&mut self, mv: Move) -> Result<(), InvalidMove> {
        // If the source tile is not in the grid, the move is invalid
        if !self.0.is_valid_index(mv.from) {
            return Err(InvalidMove::FromInvalidTile);
        }

        let dst_idx = match mv.direction {
            Direction::Right => self.0.right(mv.from),
            Direction::Left => self.0.left(mv.from),
            Direction::Up => self.0.up(mv.from),
            Direction::Down => self.0.down(mv.from),
        }
        .ok_or(InvalidMove::ToInvalidTile)?;

        let outcome = {
            let mut src = self.get_mut(mv.from);
            match src.owner() {
                Some(player) => {
                    if player != mv.player {
                        warn!(
                            "source tile is owned by {:?}, but move comes from {}.",
                            player, mv.player
                        );
                        return Err(InvalidMove::SourceTileNotOwned);
                    }
                    let mut dst = self.get_mut(dst_idx);
                    src.attack(&mut dst)?
                }
                None => {
                    warn!("source tile is not owned by any player");
                    return Err(InvalidMove::SourceTileNotOwned);
                }
            }
        };

        match outcome {
            // If a general was captured, give all the tiles owned by the defeated general to
            // the attacker, and make all the tiles visible by the defeated general visible by
            // the attacker.
            MoveOutcome::GeneralCaptured(defeated_player) => {
                for mut t in self.iter_mut().filter(|t| !t.is_mountain()) {
                    if t.owner() == Some(defeated_player) {
                        t.set_owner(Some(mv.player));
                    }
                    if t.is_visible_by(defeated_player) {
                        t.hide_from(defeated_player);
                        t.reveal_to(mv.player);
                    }
                }
            }
            // If a regular tile was captured, we just need to extend the player's horizon and
            // reveal a few new tiles.
            MoveOutcome::TileCaptured(defeated_player) => {
                if let Some(defeated_player) = defeated_player {
                    self.shrink_horizon(defeated_player, dst_idx);
                }
                self.enlarge_horizon(mv.player, dst_idx);
            }
            // If no tile was captured, the player's visibility does not change, so there's
            // nothing to do.
            _ => {}
        }
        Ok(())
    }

    /// Return an iterator over all the tiles. The tiles are mutable.
    fn iter_mut(&mut self) -> impl Iterator<Item = RefMut<Tile>> {
        self.0.iter().map(RefCell::borrow_mut)
    }

    /// Return an iterator over all the tiles with their indices. The tiles are mutable.
    pub fn enumerate_mut(&self) -> impl Iterator<Item = (usize, RefMut<Tile>)> {
        self.0.iter().enumerate().map(|(i, t)| (i, t.borrow_mut()))
    }

    /// Return a mutable reference to the tile at the given index.
    pub fn get_mut(&self, index: usize) -> RefMut<Tile> {
        self.0.get(index).borrow_mut()
    }

    #[cfg(test)]
    /// Return a reference to the tile at the given index.
    pub(crate) fn get(&self, index: usize) -> Ref<Tile> {
        self.0.get(index).borrow()
    }

    /// Make sure the given player can see all the tiles surrounding the given index. This should be
    /// called after the player just conquered the tile.
    pub fn enlarge_horizon(&self, player: PlayerId, idx: usize) {
        for mut tile in self
            .0
            .extended_neighbors(idx)
            .map(|i| self.get_mut(i))
            .filter(|t| !t.is_mountain())
        {
            tile.reveal_to(player);
        }
    }

    /// Reduce the visibility of the tiles that surround the tile at the given index, for the given
    /// player. This should be called after the player just lost the tile.
    fn shrink_horizon(&self, player: PlayerId, idx: usize) {
        for (index, mut neighbor) in self
            .0
            .extended_neighbors(idx)
            .map(|i| (i, self.get_mut(i)))
            .filter(|(_, t)| !t.is_mountain() && t.is_visible_by(player))
        {
            if !self.owns_extended_neighbor(player, index) {
                neighbor.hide_from(player);
            }
        }
    }

    /// Return whether the given player is the own of any of the tile that surround the given tile.
    /// This is used to know whether that player can view the given tile or if it's in the fog or
    /// war.
    fn owns_extended_neighbor(&self, player: PlayerId, idx: usize) -> bool {
        for tile in self
            .0
            .extended_neighbors(idx)
            .map(|i| self.0.get(i).borrow())
        {
            if tile.owner() == Some(player) {
                return true;
            }
        }
        false
    }

    /// Increment the number of units of the tiles that are owned by players. If the
    /// `reinforce_all_tiles` is `false`, then only the generals and cityes are reinforced,
    /// otherwise, all the tiles are reinforced.
    pub fn reinforce(&mut self, reinforce_all_tiles: bool) {
        for mut tile in self.iter_mut().filter(|t| !t.is_mountain()) {
            if
            // reinforce open tiles only when there's a global reinforcement round
            (tile.owner().is_some() && reinforce_all_tiles)
                    // reinforce generals every round
                    || tile.is_general()
                    // reinfoce city every round if they are occupied
                    || (tile.is_city() && tile.owner().is_some())
            {
                trace!("reinforcing tile {:?}", tile);
                tile.incr_units(1);
            }
            trace!("not reinforcing tile {:?}", tile);
        }
    }
}
