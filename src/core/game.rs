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
        info!("starting a new game for player {:?}", players);

        let (generals, map) = Map::generate(players.len());
        assert_eq!(generals.len(), players.len());

        for (general, player) in generals.into_iter().zip(players.iter().cloned()) {
            info!("spawning player {} on {}", general, player);
            let mut tile = map.get_mut(general);
            tile.set_owner(Some(player));
            map.enlarge_horizon(player, general);
        }

        let mut game = Game {
            map,
            players: HashMap::with_capacity(players.len()),
            turn: 0,
        };
        for player_id in players.drain(..) {
            let mut player = Player::new(player_id);
            // Players start with a general
            player.owned_tiles = 1;
            let _ = game.players.insert(player_id, player);
        }
        info!("game is ready to start");
        game
    }

    /// Return the current turn number
    pub fn turn(&self) -> usize {
        self.turn
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
        info!("player {} is resigning", id);
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
        info!("processing move {:?}", mv);
        if let Some(player) = self.players.get(&mv.player) {
            if !player.can_move() {
                warn!("player {} cannot move, ignoring the move", mv.player);
                return;
            }
            if let Err(e) = self.map.perform_move(mv) {
                warn!("failed to process move {:?}: {}", mv, e);
            }
        } else {
            warn!("unknown player {}, ignoring the move", mv.player);
        }
    }

    /// Increment the number of units on tiles that are owned by players.
    /// Regular tiles are reinforced once every 25 turns, but generals and fortresses are
    /// reinforced at every turn.
    pub fn reinforce(&mut self) {
        if self.turn % 50 == 0 {
            info!("reinforcing all the tiles");
            self.map.reinforce(true);
        } else if self.turn % 2 == 0 {
            info!("reinforcing generals and fortresses");
            self.map.reinforce(false);
        }
    }

    /// Increment the number of turns and reinforce the tiles that needs to be reinforced.
    pub fn incr_turn(&mut self) {
        self.turn += 1;
        info!("incrementing turn: {}", self.turn);
        self.reinforce();
    }

    /// Get all the tiles that are marked as dirty, unmark them, and return them along with other
    /// of properties that reflect the changes that occured in the game since the last update.
    pub fn get_update(&mut self) -> Update {
        info!("building an update of the game");

        // Reset the number of tiles owned by each player, because we'll recount them while
        // building the update.
        for mut player in self.players.values_mut() {
            player.owned_tiles = 0;
        }

        // Get all the dirty tiles
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
                        panic!("Tile {:?} owned by an unknown player {}", tile, owner);
                    }
                }
                if self.is_first_turn() || tile.is_dirty() {
                    updated_tiles.push((i, tile.clone()));
                    tile.set_clean();
                }
            }
            updated_tiles
        };

        for player in self.players.values_mut() {
            if player.owned_tiles == 0 && !player.defeated() {
                info!(
                    "player {} does not own any tile: marking it as defeated",
                    player.id
                );
                player.defeated_at = Some(self.turn);
            }
        }

        Update {
            turn: self.turn,
            players: self.players.clone(),
            width: self.map.width(),
            height: self.map.height(),
            is_initial_update: self.is_first_turn(),
            tiles: updated_tiles,
        }
    }

    fn is_first_turn(&self) -> bool {
        self.turn() == 0
    }
}

#[derive(Serialize, Clone)]
pub struct Update {
    turn: usize,
    width: usize,
    height: usize,
    players: HashMap<PlayerId, Player>,
    tiles: Vec<(usize, Tile)>,
    #[serde(skip)]
    is_initial_update: bool,
}

impl Update {
    pub fn filtered(&self, player: PlayerId) -> Self {
        info!("filtering update for player {}", player);
        Update {
            turn: self.turn,
            width: self.width,
            height: self.height,
            players: self.players.clone(),
            is_initial_update: self.is_initial_update,
            tiles: self.tiles
                .iter()
                .filter(|(_, t)| t.is_dirty_for(&player) || self.is_initial_update)
                .map(|(i, t)| {
                    let mut t = t.clone();
                    if !t.is_visible_by(player) {
                        t.set_units(0);
                        if t.is_general() {
                            t.make_open();
                            t.set_owner(None);
                        }

                        if t.is_fortress() {
                            t.make_wall();
                        }
                    }
                    (*i, t)
                })
                .collect(),
        }
    }
}
