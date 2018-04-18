use super::common::{Move, Player, PlayerId, Tile};
use super::map::Map;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Game {
    pub(crate) map: Map,
    pub(crate) players: HashMap<PlayerId, Player>,
    pub(crate) turn: usize,
}

impl Game {
    /// Create a new gmae for the given players. The map that is generated for the game gets bigger
    /// as the number of players increases. A game start at turn 0, with each player owning exactly
    /// one tile, their general.
    pub fn new(mut players: Vec<PlayerId>) -> Self {
        let mut game = Game {
            map: Map::generate(players.len()),
            players: HashMap::with_capacity(players.len()),
            turn: 0,
        };
        for player in players.drain(..) {
            let _ = game.players.insert(player, Player::new(player));
        }
        game
    }

    /// Mark the given player as defeated. When a player is defeated he cannot perform any action
    /// anymore. Note that this method does not take care of the tiles owned by the player
    /// resigning so there are two cases:
    ///
    /// - This method was called because the player performed the action of resigning, in which
    ///   case it still owns some tiles, that will keep beeing reinforced
    /// - The player's general has been captured, in which case all his tiles have already been
    ///   transfered to the player that captured him
    pub fn resign(&mut self, id: PlayerId) {
        let player = self.players.get_mut(&id).expect("Unknown player");
        if player.defeated() {
            error!("Got resignation from player defeated at turn {}", self.turn);
        } else {
            player.defeated_at = Some(self.turn);
        }
    }

    /// Process the given move, and update the game state. If the move is invalid (between tiles
    /// that are not adjacent, or from a tile that does not belong to the player making the move,
    /// for example), it is simply ignored. Not error is returned. Tiles that are updated by the
    /// move are marked as dirty.
    pub fn perform_move(&mut self, mv: Move) {
        if let Some(player) = self.players.get(&mv.player) {
            if !player.can_move() {
                warn!("Ignoring move from player {}", mv.player);
            }
            // We don't care about the result of the move.
            let _ = self.map.perform_move(mv);
        } else {
            warn!("Ignoring move from unkown player {}", mv.player);
        }
    }

    /// Increment the number of units on tiles that are owned by players.
    /// Regular tiles are reinforced once every 25 turns, but generals and fortresses are
    /// reinforced at every turn.
    pub fn reinforce(&mut self) {
        if self.turn % 25 == 0 {
            self.map.reinforce(true);
        } else {
            self.map.reinforce(false);
        }
    }

    /// Get all the tiles that are marked as dirty, unmark them, and return them along with other
    /// of properties that reflect the changes that occured in the game since the last update.
    pub fn get_update(&mut self) -> Update {
        let updated_tiles = {
            let Game {
                ref mut players,
                ref map,
                ..
            } = self;
            let mut updated_tiles = Vec::with_capacity(map.len());
            for (i, mut tile) in map.enumerate_mut() {
                if let Some(owner) = tile.owner() {
                    if let Some(player) = players.get_mut(&owner) {
                        player.owned_tiles += 1;
                    } else {
                        panic!("Tile owned by an unknown player");
                    }
                }
                if tile.is_dirty() {
                    updated_tiles.push((i, tile.clone()));
                    tile.set_clean();
                }
            }
            updated_tiles
        };

        for mut player in self.players.values_mut() {
            player.owned_tiles = 0;
        }
        for player in self.players.values_mut() {
            if player.owned_tiles == 0 && !player.defeated() {
                player.defeated_at = Some(self.turn);
            }
        }

        self.turn += 1;

        Patch {
            turn: self.turn,
            players: self.players.clone(),
            updated_tiles,
        }.into()
    }

    /// Return a full representation of the current game state.
    pub fn get_snapshot(&self) -> Snapshot {
        Snapshot {
            turn: self.turn,
            width: self.map.width(),
            height: self.map.height(),
            players: self.players.clone(),
            tiles: self.map.iter().map(|t| t.clone()).collect(),
        }
    }
}

/// Represent a set of changes in a game. It contains the tiles that have been modified as well as
/// information about the players and the current turn number.
#[derive(Serialize, Clone)]
pub struct Patch {
    turn: usize,
    players: HashMap<PlayerId, Player>,
    updated_tiles: Vec<(usize, Tile)>,
}

impl Patch {
    pub fn filtered(&self, player: PlayerId) -> Patch {
        Patch {
            turn: self.turn,
            players: self.players.clone(),
            updated_tiles: self.updated_tiles
                .iter()
                .filter(|(_, t)| t.is_visible_by(player))
                .map(|(i, t)| (*i, t.clone()))
                .collect(),
        }
    }
}

/// A full representation of a game, that can be serialized.
#[derive(Serialize, Clone)]
pub struct Snapshot {
    turn: usize,
    width: usize,
    height: usize,
    players: HashMap<PlayerId, Player>,
    tiles: Vec<Tile>,
}

impl Snapshot {
    pub fn filtered(&self, player: PlayerId) -> Snapshot {
        let tiles = self.tiles
            .iter()
            .map(|t| {
                let mut t = t.clone();
                if !t.is_visible_by(player) {
                    t.set_units(0);
                    if t.is_general() {
                        t.make_open();
                    }
                }
                t
            })
            .collect();

        Snapshot {
            turn: self.turn,
            width: self.width,
            height: self.height,
            players: self.players.clone(),
            tiles,
        }
    }
}

/// Represent an update that can be sent to clients to inform them of the current game state.
#[derive(Serialize, Clone)]
#[serde(untagged)]
pub enum Update {
    /// A representation of a set of changes that occured in the game
    Patch(Patch),
    /// A full representation of the game
    Snapshot(Snapshot),
}

impl From<Patch> for Update {
    fn from(patch: Patch) -> Self {
        Update::Patch(patch)
    }
}

impl From<Snapshot> for Update {
    fn from(snapshot: Snapshot) -> Self {
        Update::Snapshot(snapshot)
    }
}

impl Update {
    pub fn filtered(&self, player: PlayerId) -> Self {
        match *self {
            Update::Patch(ref patch) => patch.filtered(player).into(),
            Update::Snapshot(ref snapshot) => snapshot.filtered(player).into(),
        }
    }
}
