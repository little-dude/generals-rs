pub type PlayerId = usize;

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Player {
    pub id: PlayerId,
    pub owned_tiles: usize,
    #[serde(skip)]
    pub defeated_at: Option<usize>,
}

impl Player {
    pub fn new(id: PlayerId) -> Self {
        Player {
            id,
            owned_tiles: 0,
            defeated_at: None,
        }
    }

    pub fn defeated(&self) -> bool {
        self.defeated_at.is_some()
    }

    pub fn can_move(&self) -> bool {
        !self.defeated() && self.owned_tiles > 0
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub enum Action {
    Resign,
    CancelMoves,
    Move(Move),
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct Move {
    #[serde(skip)]
    pub player: PlayerId,
    pub from: usize,
    pub direction: Direction,
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
pub enum Direction {
    Right,
    Left,
    Up,
    Down,
}

pub enum MoveOutcome {
    GeneralCaptured(PlayerId),
    TileCaptured(Option<PlayerId>),
    StatuQuo,
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize)]
pub enum TileKind {
    General,
    Fortress,
    Normal,
}

fn is_normal(kind: &TileKind) -> bool {
    *kind == TileKind::Normal
}

fn has_no_unit(units: &u16) -> bool {
    *units == 0
}

#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct OpenTile {
    #[serde(skip_serializing_if = "Option::is_none")]
    owner: Option<PlayerId>,

    #[serde(skip_serializing_if = "has_no_unit")]
    units: u16,

    #[serde(skip_serializing_if = "is_normal")]
    kind: TileKind,

    #[serde(skip)]
    visible_by: Vec<PlayerId>,

    #[serde(skip)]
    dirty: bool,
}

impl OpenTile {
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
    fn is_visible_by(&self, player: PlayerId) -> bool {
        for p in &self.visible_by {
            if *p == player {
                return true;
            }
        }
        false
    }

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

    fn reveal_to(&mut self, player: PlayerId) {
        if !self.is_visible_by(player) {
            self.visible_by.push(player);
            self.dirty = true;
        }
    }

    /// Perform an attack from a source tile to a destination tile. This method assumes both tiles
    /// are valid tiles (ie that they are not walls), and that the source tile has an owner and at
    /// least 2 units. Such validation should be performed before calling this method.
    ///
    /// # Panics
    ///
    /// This method panics if `self` does not have an owner or has 0 unit.
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
            // The destination tile is not owned by anyone. Capture it.
            None => {
                dst.units += self.units - 1;
                dst.owner = self.owner;
                MoveOutcome::TileCaptured(None)
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

// serde serializes Option as `null` if None, and just the value if `Some`
#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct Tile(Option<OpenTile>);

impl Default for Tile {
    fn default() -> Self {
        Tile(None)
    }
}

impl Tile {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn owner(&self) -> Option<PlayerId> {
        self.0.as_ref().and_then(|t| t.owner)
    }

    pub fn units(&self) -> Option<u16> {
        self.0.as_ref().map(|t| t.units)
    }

    pub fn is_open(&self) -> bool {
        self.0
            .as_ref()
            .map(|tile| match tile.kind {
                TileKind::Normal => true,
                TileKind::Fortress | TileKind::General => false,
            })
            .unwrap_or(false)
    }

    pub fn is_general(&self) -> bool {
        self.0
            .as_ref()
            .map(|tile| match tile.kind {
                TileKind::General => true,
                TileKind::Fortress | TileKind::Normal => false,
            })
            .unwrap_or(false)
    }

    pub fn is_fortress(&self) -> bool {
        self.0
            .as_ref()
            .map(|tile| match tile.kind {
                TileKind::Fortress => true,
                TileKind::General | TileKind::Normal => false,
            })
            .unwrap_or(false)
    }

    pub fn is_wall(&self) -> bool {
        self.0.is_none()
    }

    pub fn make_open(&mut self) {
        if self.0.is_none() {
            self.0 = Some(OpenTile::new(TileKind::Normal));
        } else {
            self.0.as_mut().unwrap().kind = TileKind::Normal;
        }
    }

    pub fn make_general(&mut self) {
        if self.0.is_none() {
            self.0 = Some(OpenTile::new(TileKind::General));
        } else {
            self.0.as_mut().unwrap().kind = TileKind::General;
        }
    }

    pub fn make_fortress(&mut self) {
        if self.0.is_none() {
            self.0 = Some(OpenTile::new(TileKind::Fortress));
        } else {
            self.0.as_mut().unwrap().kind = TileKind::Fortress;
        }
    }

    pub fn set_units(&mut self, units: u16) {
        if let Some(ref mut tile) = self.0 {
            tile.units = units;
            tile.dirty = true;
        }
    }

    pub fn incr_units(&mut self, units: u16) {
        if let Some(ref mut tile) = self.0 {
            tile.units += units;
            tile.dirty = true;
        }
    }

    pub fn decr_units(&mut self, units: u16) {
        if let Some(ref mut tile) = self.0 {
            if units < tile.units {
                tile.units -= units;
            } else {
                tile.units = 0;
            }
            tile.dirty = true;
        }
    }

    pub fn set_owner(&mut self, player: Option<PlayerId>) {
        if let Some(ref mut tile) = self.0 {
            tile.owner = player;
            tile.dirty = true;
        }
    }

    pub fn is_dirty(&self) -> bool {
        if let Some(ref tile) = self.0 {
            tile.dirty
        } else {
            false
        }
    }

    pub fn set_clean(&mut self) {
        if let Some(ref mut tile) = self.0 {
            tile.dirty = false;
        }
    }

    pub fn is_visible_by(&self, player: PlayerId) -> bool {
        if let Some(tile) = self.0.as_ref() {
            tile.is_visible_by(player)
        } else {
            false
        }
    }

    pub fn hide_from(&mut self, player: PlayerId) {
        if let Some(tile) = self.0.as_mut() {
            tile.hide_from(player);
        }
    }

    pub fn reveal_to(&mut self, player: PlayerId) {
        if let Some(tile) = self.0.as_mut() {
            tile.reveal_to(player);
        }
    }

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

#[derive(Debug)]
pub enum InvalidMove {
    NotEnoughUnits,
    ToInvalidTile,
    FromInvalidTile,
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
