use super::common::{Move, Player, PlayerId, Tile};
use super::map::Map;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Game {
    map: Map,
    players: HashMap<PlayerId, Player>,
    turn: usize,
}

impl Game {
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

    pub fn resign(&mut self, id: PlayerId) {
        let player = self.players.get_mut(&id).expect("Unknown player");
        if player.defeated() {
            error!("Got resignation from player defeated at turn {}", self.turn);
        } else {
            player.defeated_at = Some(self.turn);
        }
    }

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

    pub fn reinforce(&mut self) {
        if self.turn % 25 == 0 {
            self.map.reinforce(true);
        } else {
            self.map.reinforce(false);
        }
    }

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
}

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

#[derive(Serialize, Clone)]
pub struct Snapshot {
    turn: usize,
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
            players: self.players.clone(),
            tiles,
        }
    }
}

#[derive(Serialize, Clone)]
#[serde(untagged)]
pub enum Update {
    Patch(Patch),
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
