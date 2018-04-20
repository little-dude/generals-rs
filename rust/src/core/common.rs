pub type PlayerId = usize;

/// Represent a player during a game.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Player {
    /// An integer that uniquely identifies each player during a game
    pub id: PlayerId,

    /// Number of tiles the player currently owns
    #[serde(skip_serializing_if = "has_no_tile")]
    pub owned_tiles: usize,

    /// Turn at which the player was defeated, if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defeated_at: Option<usize>,
}

/// Small helper used by serde to avoid serializing the owned_tile field if the player does not own
/// any tile. We try to keep the jsons as small as possible for network efficiency.
fn has_no_tile(owned_tiles: &usize) -> bool {
    *owned_tiles == 0
}

impl Player {
    /// Return a new undefeated player, with no owned tile.
    pub fn new(id: PlayerId) -> Self {
        Player {
            id,
            owned_tiles: 0,
            defeated_at: None,
        }
    }

    /// Return whether the player has been defeated already
    pub fn defeated(&self) -> bool {
        self.defeated_at.is_some()
    }

    /// Return whether the player can move. A player can move if it owns at least one tile, and if
    /// it has not been defeated.
    pub fn can_move(&self) -> bool {
        !self.defeated() && self.owned_tiles > 0
    }
}

/// Represent an action a player can perform.
#[derive(Copy, Clone, Debug, Deserialize)]
pub enum Action {
    /// Resign
    Resign,
    /// Cancel all the moves already queued for the player
    CancelMoves,
    /// Make a move from a tile to another
    Move(Move),
}

/// Represent a move from one tile to another. During a move, units are transfered from one tile to
/// another adjacent tile.
#[derive(Copy, Clone, Debug, Deserialize)]
pub struct Move {
    /// Player that is making the move.
    #[serde(skip)]
    pub player: PlayerId,
    /// Index of the tile from which troops are being moved.
    pub from: usize,
    /// Direction to which the troops are being moved.
    pub direction: Direction,
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
pub enum Direction {
    Right,
    Left,
    Up,
    Down,
}

#[derive(Copy, Clone, Debug, PartialEq)]
/// Outcome of a move
pub enum MoveOutcome {
    /// Outcome when a move resulted in a general being captured. The player ID is the ID of the
    /// defeated player.
    GeneralCaptured(PlayerId),
    /// Outcome when a move resulted in an open tile or a fortress tile being captured. If the tile
    /// was belonging to a different player than the one making the move, the player's ID is
    /// specified.
    TileCaptured(Option<PlayerId>),
    /// Outcome when a move did not result in a tile being captured.
    StatuQuo,
}

/// Represent the different types of open (ie non-wall) tiles
#[derive(Copy, Clone, PartialEq, Debug, Serialize)]
pub enum TileKind {
    /// A tile that contains a general
    General,
    /// A tile that contains a fortress
    Fortress,
    /// A regular tile
    Normal,
}

/// Represent an open tile. Open tiles are tiles that are not walls, ie tiles that players can
/// conquer.
#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct OpenTile {
    /// The ID of the player that currenlty owns the tile (a player own a tile if he/she has units
    /// occupying the tile).
    #[serde(skip_serializing_if = "Option::is_none")]
    owner: Option<PlayerId>,

    /// Number of units occupying the tile
    #[serde(skip_serializing_if = "has_no_unit")]
    units: u16,

    /// The type of tile (normal, fortress or general)
    #[serde(skip_serializing_if = "is_normal")]
    kind: TileKind,

    /// List of players that can see the tile. To be able to see an open tile, a player must own a
    /// tile that touches it.
    #[serde(skip)]
    visible_by: Vec<PlayerId>,

    /// Flag that indicates that this tile's state changed. It is set when the number of units on
    /// the tile changes or when the owner changes.
    #[serde(skip)]
    dirty: bool,
}

/// Small helper used by serde to avoid serializing the `kind` field if the tile if of type
/// `TileKind::Normal`. We try to keep the jsons as small as possible for network efficiency.
fn is_normal(kind: &TileKind) -> bool {
    *kind == TileKind::Normal
}

/// Small helper used by serde to avoid serializing the `units` field if the tile does not have any
/// units. We try to keep the jsons as small as possible for network efficiency.
fn has_no_unit(units: &u16) -> bool {
    *units == 0
}

impl OpenTile {
    /// Return a new open tile or the given type, with no owner, and no unit.
    pub fn new(kind: TileKind) -> Self {
        OpenTile {
            owner: None,
            units: 0,
            dirty: false,
            visible_by: Vec::new(),
            kind,
        }
    }
}

impl OpenTile {
    /// Return whether the tile is marked as visible by the given player.
    fn is_visible_by(&self, player: PlayerId) -> bool {
        for p in &self.visible_by {
            if *p == player {
                return true;
            }
        }
        false
    }

    /// Mark the tile as invisible for the given player
    fn hide_from(&mut self, player: PlayerId) {
        for i in 0..self.visible_by.len() {
            if self.visible_by[i] != player {
                continue;
            }
            self.visible_by.remove(i);
            self.dirty = true;
            return;
        }
    }

    /// Mark the tile as visible for the given player, updating the source and destination tiles
    /// state if necessary (number of units, owner, etc.).
    fn reveal_to(&mut self, player: PlayerId) {
        if !self.is_visible_by(player) {
            self.visible_by.push(player);
            self.dirty = true;
        }
    }

    /// Perform a move from a source tile to a destination tile.
    fn attack(&mut self, dst: &mut OpenTile) -> Result<MoveOutcome, InvalidMove> {
        let attacker = self.owner.ok_or(InvalidMove::UnclaimedSourceTile)?;
        if self.units < 2 {
            return Err(InvalidMove::NotEnoughUnits);
        }

        let outcome = match dst.owner {
            // The destination tile belongs to someone else
            Some(defender) if defender != attacker => {
                // The defender has more units.
                if dst.units >= self.units - 1 {
                    dst.units -= self.units - 1;
                    MoveOutcome::StatuQuo
                }
                // The attacker has more units. Capture the tile.
                else {
                    dst.units = self.units - 1 - dst.units;
                    dst.owner = self.owner;
                    // We're capturing a general
                    if dst.kind == TileKind::General {
                        //  Turn the general into a regular fortress
                        dst.kind = TileKind::Fortress;
                        MoveOutcome::GeneralCaptured(defender)
                    }
                    // We're capturing a regular tile
                    else {
                        MoveOutcome::TileCaptured(Some(defender))
                    }
                }
            }
            // The owner is the same for both tiles, just transfer the unit
            Some(_defender) => {
                dst.units += self.units - 1;
                MoveOutcome::StatuQuo
            }
            // The destination tile is not owned by anyone.
            None => {
                // The destination has more units, we can't capture it
                if dst.units >= self.units - 1 {
                    dst.units -= self.units - 1;
                    MoveOutcome::StatuQuo
                } else {
                    dst.units = self.units - 1 - dst.units;
                    dst.owner = self.owner;
                    MoveOutcome::TileCaptured(None)
                }
            }
        };
        // In any case, we always only leave 1 unit in the source tile
        // TODO: would be nice to support splitting the source tile units before moving.
        self.units = 1;
        self.dirty = true;
        dst.dirty = true;
        Ok(outcome)
    }
}

/// Represent a tile.
#[derive(Clone, PartialEq, Debug, Serialize)]
// A wall is represented by `None`, and any tile that is not a wall is represented as
// `Some(OpenTile)`
pub struct Tile(Option<OpenTile>);

impl Default for Tile {
    fn default() -> Self {
        Tile(None)
    }
}

impl Tile {
    /// Return a new tile. By default, new tiles are walls.
    pub fn new() -> Self {
        Default::default()
    }

    /// Return the owner of the tile, if any
    pub fn owner(&self) -> Option<PlayerId> {
        self.0.as_ref().and_then(|t| t.owner)
    }

    #[cfg(test)]
    /// Return the number of units occupying the tile
    pub fn units(&self) -> u16 {
        self.0.as_ref().map(|t| t.units).unwrap_or(0)
    }

    /// Return whether the tile is open. A tile is open if it's not a fortress, a general or a
    /// wall.
    pub fn is_open(&self) -> bool {
        self.0
            .as_ref()
            .map(|tile| match tile.kind {
                TileKind::Normal => true,
                TileKind::Fortress | TileKind::General => false,
            })
            .unwrap_or(false)
    }

    /// Return whether the tile is a general.
    pub fn is_general(&self) -> bool {
        self.0
            .as_ref()
            .map(|tile| match tile.kind {
                TileKind::General => true,
                TileKind::Fortress | TileKind::Normal => false,
            })
            .unwrap_or(false)
    }

    /// Return whether the tile is a fortress.
    pub fn is_fortress(&self) -> bool {
        self.0
            .as_ref()
            .map(|tile| match tile.kind {
                TileKind::Fortress => true,
                TileKind::General | TileKind::Normal => false,
            })
            .unwrap_or(false)
    }

    /// Return whether the tile is a wall
    pub fn is_wall(&self) -> bool {
        self.0.is_none()
    }

    /// Turn the tile into an open tile (ie a tile that is not a wall, a general or a fortress)
    pub fn make_open(&mut self) {
        if self.0.is_none() {
            self.0 = Some(OpenTile::new(TileKind::Normal));
        } else {
            self.0.as_mut().unwrap().kind = TileKind::Normal;
        }
        self.0.as_mut().unwrap().dirty = true;
    }

    /// Turn the tile into a general
    pub fn make_general(&mut self) {
        if self.0.is_none() {
            self.0 = Some(OpenTile::new(TileKind::General));
        } else {
            self.0.as_mut().unwrap().kind = TileKind::General;
        }
        self.0.as_mut().unwrap().dirty = true;
    }

    /// Turn the tile into a fortess.
    pub fn make_fortress(&mut self) {
        if self.0.is_none() {
            self.0 = Some(OpenTile::new(TileKind::Fortress));
        } else {
            self.0.as_mut().unwrap().kind = TileKind::Fortress;
        }
        self.0.as_mut().unwrap().dirty = true;
    }

    /// Set the number of units occupying the tile
    pub fn set_units(&mut self, units: u16) {
        if let Some(ref mut tile) = self.0 {
            tile.units = units;
            tile.dirty = true;
        }
    }

    /// Increment the number of units occupying the tile
    pub fn incr_units(&mut self, units: u16) {
        if let Some(ref mut tile) = self.0 {
            tile.units += units;
            tile.dirty = true;
        }
    }

    /// Set the owner of the tile. To remove the existing owner, set the owner to `None`.
    pub fn set_owner(&mut self, player: Option<PlayerId>) {
        if let Some(ref mut tile) = self.0 {
            tile.owner = player;
            if let Some(player) = player {
                tile.reveal_to(player);
            }
            tile.dirty = true;
        }
    }

    /// Return whether the tile's state has changed. A tile state changes when its type, its owner,
    /// or the number of units occupying it changes.
    pub fn is_dirty(&self) -> bool {
        if let Some(ref tile) = self.0 {
            tile.dirty
        } else {
            false
        }
    }

    /// Mark the tile a clean. This should be called to acknoledge that the tile has been processed
    /// when after is was marked as dirty.
    pub fn set_clean(&mut self) {
        if let Some(ref mut tile) = self.0 {
            tile.dirty = false;
        }
    }

    /// Check whether the tile is marked as visible by the given player.
    pub fn is_visible_by(&self, player: PlayerId) -> bool {
        if let Some(tile) = self.0.as_ref() {
            tile.is_visible_by(player)
        } else {
            false
        }
    }

    /// Mark the tile as invisible for the given player.
    pub fn hide_from(&mut self, player: PlayerId) {
        if let Some(tile) = self.0.as_mut() {
            tile.hide_from(player);
        }
    }

    /// Mark the tile as visible for the given player.
    pub fn reveal_to(&mut self, player: PlayerId) {
        if let Some(tile) = self.0.as_mut() {
            tile.reveal_to(player);
        }
    }

    /// Move units from this tile to the given tile, updating the number of units of both tile, and
    /// the owners of the target tile if necessary.
    pub fn attack(&mut self, tile: &mut Tile) -> Result<MoveOutcome, InvalidMove> {
        if let Some(src) = self.0.as_mut() {
            if let Some(dst) = tile.0.as_mut() {
                src.attack(dst)
            } else {
                Err(InvalidMove::ToInvalidTile)
            }
        } else {
            Err(InvalidMove::FromInvalidTile)
        }
    }
}

/// Represent an error that occurs when an invalid move is processed.
#[derive(Debug, PartialEq, Eq)]
pub enum InvalidMove {
    /// The source tile does not have enough units to perform the move. To be able to move from one
    /// tile, the tile must have at least two units.
    NotEnoughUnits,

    /// The destination tile is invalid (it can be a wall or an out-of-grid tile. This occurs for
    /// instance if the source tile is on the top row, and the move is upward.
    ToInvalidTile,

    /// The source tile is either a wall or out of the grid.
    FromInvalidTile,

    /// The source tile does not belong to anyone. A move can only be performed by a player, so it
    /// does not make sense to perform a move form a tile that does not belong to any player.
    UnclaimedSourceTile,
}

use std::error::Error;
use std::fmt;

impl Error for InvalidMove {
    fn description(&self) -> &str {
        match *self {
            InvalidMove::NotEnoughUnits => "not enough unit on the source tile",
            InvalidMove::ToInvalidTile => "the destination tile is either a wall or not on the map",
            InvalidMove::FromInvalidTile => "the source tile is either a wall or not on the map",
            InvalidMove::UnclaimedSourceTile => "the source tile does not belong to any player",
        }
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

impl fmt::Display for InvalidMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid move: {}", self.description())
    }
}
