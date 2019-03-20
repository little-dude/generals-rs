use std::collections::HashSet;

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
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Action {
    /// Resign
    Resign,
    /// Cancel all the moves already queued for the player
    #[serde(rename = "cancel_moves")]
    CancelMoves,
    /// Make a move from a tile to another
    Move(Move),
}

/// Represent a move from one tile to another. During a move, units are transfered from one tile to
/// another adjacent tile.
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Move {
    /// Player that is making the move.
    #[serde(skip)]
    pub player: PlayerId,
    /// Index of the tile from which troops are being moved.
    pub from: usize,
    /// Direction to which the troops are being moved.
    pub direction: Direction,
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
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
    /// Outcome when a move resulted in an open tile or a city tile being captured. If the tile
    /// was belonging to a different player than the one making the move, the player's ID is
    /// specified.
    TileCaptured(Option<PlayerId>),
    /// Outcome when a move did not result in a tile being captured.
    StatuQuo,
}

/// Represent the different types of open (ie non-mountain) tiles
#[derive(Copy, Clone, PartialEq, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TileKind {
    /// A tile that contains a general
    General,
    /// A tile that contains a city
    City,
    /// A regular tile
    Open,
    /// A tile that contains a mountain
    Mountain,
}

/// Represent an open tile. Open tiles are tiles that are not mountains, ie tiles that players can
/// conquer.
#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct Tile {
    /// The ID of the player that currenlty owns the tile (a player own a tile if he/she has units
    /// occupying the tile).
    #[serde(skip_serializing_if = "Option::is_none")]
    owner: Option<PlayerId>,

    /// Number of units occupying the tile
    #[serde(skip_serializing_if = "has_no_unit")]
    units: u16,

    /// The type of tile (open, city or general)
    #[serde(skip_serializing_if = "is_open")]
    kind: TileKind,

    /// List of players that can see the tile. To be able to see an open tile, a player must own a
    /// tile that touches it.
    #[serde(skip)]
    visible_by: HashSet<PlayerId>,

    /// Players that had visibility on this tile when it changed.
    #[serde(skip)]
    dirty_for: HashSet<PlayerId>,
}

/// Small helper used by serde to avoid serializing the `kind` field if the tile if of type
/// `TileKind::Open`. We try to keep the jsons as small as possible for network efficiency.
fn is_open(kind: &TileKind) -> bool {
    *kind == TileKind::Open
}

/// Small helper used by serde to avoid serializing the `units` field if the tile does not have any
/// units. We try to keep the jsons as small as possible for network efficiency.
fn has_no_unit(units: &u16) -> bool {
    *units == 0
}

impl Tile {
    /// Return a new open tile or the given type, with no owner, and no unit.
    pub fn new() -> Self {
        Tile {
            owner: None,
            units: 0,
            dirty_for: HashSet::new(),
            visible_by: HashSet::new(),
            kind: TileKind::Mountain,
        }
    }

    /// Return whether the tile is marked as visible by the given player.
    pub fn is_visible_by(&self, player: PlayerId) -> bool {
        self.visible_by.contains(&player)
    }

    /// Mark the tile as invisible for the given player
    pub fn hide_from(&mut self, player: PlayerId) {
        let was_visible = self.visible_by.remove(&player);
        if was_visible {
            self.dirty_for.insert(player);
        }
    }

    /// Mark the tile as visible for the given player, updating the source and destination tiles
    /// state if necessary (number of units, owner, etc.).
    pub fn reveal_to(&mut self, player: PlayerId) {
        self.visible_by.insert(player);
        self.dirty_for.insert(player);
    }

    /// Perform a move from a source tile to a destination tile.
    pub fn attack(&mut self, dst: &mut Tile) -> Result<MoveOutcome, InvalidMove> {
        if self.is_mountain() {
            return Err(InvalidMove::FromInvalidTile);
        }
        if dst.is_mountain() {
            return Err(InvalidMove::ToInvalidTile);
        }
        if self.units() < 2 {
            return Err(InvalidMove::NotEnoughUnits);
        }
        let attacker = self.owner.ok_or(InvalidMove::SourceTileNotOwned)?;

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
                        //  Turn the general into a regular city
                        dst.kind = TileKind::City;
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
        self.set_dirty();
        dst.set_dirty();
        Ok(outcome)
    }

    /// Return the owner of the tile, if any
    pub fn owner(&self) -> Option<PlayerId> {
        self.owner
    }

    /// Return the number of units occupying the tile
    pub fn units(&self) -> u16 {
        self.units
    }

    /// Return whether the tile is open. A tile is open if it's not a city, a general or a
    /// mountain.
    pub fn is_open(&self) -> bool {
        self.kind == TileKind::Open
    }

    /// Return whether the tile is a general.
    pub fn is_general(&self) -> bool {
        self.kind == TileKind::General
    }

    /// Return whether the tile is a city.
    pub fn is_city(&self) -> bool {
        self.kind == TileKind::City
    }

    /// Return whether the tile is a mountain
    pub fn is_mountain(&self) -> bool {
        self.kind == TileKind::Mountain
    }

    /// Turn the tile into an open tile
    pub fn make_open(&mut self) {
        self.kind = TileKind::Open;
        self.set_dirty();
    }

    pub fn set_dirty(&mut self) {
        for player_id in self.visible_by.iter() {
            self.dirty_for.insert(*player_id);
        }
    }
    /// Turn the tile into a general
    pub fn make_general(&mut self) {
        self.kind = TileKind::General;
        self.set_dirty();
    }

    // // FIXME: unused for now, but that's because we don't have city yet
    // /// Turn the tile into a fortess.
    // pub fn make_city(&mut self) {
    //     self.kind = TileKind::City;
    //     self.set_dirty();
    // }

    /// Turn the tile into a mountain.
    pub fn make_mountain(&mut self) {
        self.kind = TileKind::Mountain;
        self.set_dirty();
    }

    /// Set the number of units occupying the tile
    pub fn set_units(&mut self, units: u16) {
        if self.is_mountain() {
            return;
        }
        self.units = units;
        self.set_dirty();
    }

    /// Increment the number of units occupying the tile
    pub fn incr_units(&mut self, units: u16) {
        if self.is_mountain() {
            return;
        }
        self.units += units;
        self.set_dirty();
    }

    /// Set the owner of the tile. To remove the existing owner, set the owner to `None`.
    pub fn set_owner(&mut self, player: Option<PlayerId>) {
        if self.is_mountain() {
            return;
        }
        // Mark the tile as dirty for the players that have visibility on the tile
        self.set_dirty();
        // Mark the tile as dirty for the previous owner. As owner, it should have visibility on
        // the tile, so should have been added `dirty_for` already, but let's be safe, it's pretty
        // cheap.
        if let Some(owner) = self.owner {
            self.dirty_for.insert(owner);
        }
        self.owner = player;
        if let Some(owner) = self.owner {
            self.reveal_to(owner);
        }
    }

    /// Return whether the tile's state has changed. A tile state changes when its type, its owner,
    /// or the number of units occupying it changes.
    pub fn is_dirty(&self) -> bool {
        !self.dirty_for.is_empty()
    }

    pub fn is_dirty_for(&self, player_id: PlayerId) -> bool {
        self.dirty_for.contains(&player_id)
    }

    /// Mark the tile a clean. This should be called to acknoledge that the tile has been processed
    /// when after is was marked as dirty.
    pub fn set_clean(&mut self) {
        let _ = self.dirty_for.drain();
    }
}

/// Represent an error that occurs when an invalid move is processed.
#[derive(Debug, PartialEq, Eq)]
pub enum InvalidMove {
    /// The source tile does not have enough units to perform the move. To be able to move from one
    /// tile, the tile must have at least two units.
    NotEnoughUnits,

    /// The destination tile is invalid (it can be a mountain or an out-of-grid tile. This occurs
    /// for instance if the source tile is on the top row, and the move is upward.
    ToInvalidTile,

    /// The source tile is either a mountain or out of the grid.
    FromInvalidTile,

    /// The source tile does not belong to the player making the move. A move can only be performed
    /// by a player.
    SourceTileNotOwned,
}

use std::error::Error;
use std::fmt;

impl Error for InvalidMove {
    fn description(&self) -> &str {
        match *self {
            InvalidMove::NotEnoughUnits => "not enough unit on the source tile",
            InvalidMove::ToInvalidTile => {
                "the destination tile is either a mountain or not on the map"
            }
            InvalidMove::FromInvalidTile => {
                "the source tile is either a mountain or not on the map"
            }
            InvalidMove::SourceTileNotOwned => {
                "the source tile does not belong to the player making the move"
            }
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
