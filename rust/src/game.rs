use std::collections::HashMap;
use std::iter::FromIterator;
use std::time::{Duration, Instant};

use futures::stream::Stream;
use futures::{Async, AsyncSink, Sink};
use futures::{Future, Poll};
use tokio_timer::{self, Interval};

use connection::ConnectionProxy;
use core::{Game, PlayerId};

pub struct ActiveGame {
    game: Game,
    connections: HashMap<PlayerId, ConnectionProxy>,
    ticks: Interval,
}

impl ActiveGame {
    fn process_players_actions(&mut self) {
        debug!("processing players actions");

        for (player, mut connection) in self.connections
            .iter_mut()
            // Do not take any more action from players that have resigned
            .filter(|(_, c)| !c.has_resigned())
        {
            connection.poll_actions();
            if connection.has_resigned() {
                self.game.resign(*player);
            } else if let Some(mv) = connection.get_move() {
                self.game.perform_move(mv);
            }
        }
    }

    pub fn start_send_updates(&mut self) {
        debug!("sending updates to connection proxies");

        let ActiveGame {
            ref mut game,
            ref mut connections,
            ..
        } = *self;

        let update = game.get_update();

        for (player, mut connection) in connections.iter_mut().filter(|(_, c)| !c.has_resigned()) {
            match connection.updates.start_send(update.filtered(*player)) {
                Ok(AsyncSink::Ready) => continue,
                Ok(AsyncSink::NotReady(_)) => {
                    // If we can NotReady, the start_send attempt failed due to the sink being full.
                    // That means the Connection has been unable to process our previous updates. Since
                    // clients will end up in an inconsistent state if they miss an update, we have not
                    // other choice but consider this player lost.
                    warn!("could not send update (the sink is full). Considering that player {} resigned", player);
                }
                Err(_) => {
                    // If we get an error, the Connection closed the other of the channel, so the
                    // result is the same than above, we have to consider that this player resigned.
                    warn!("could not send update (receiver dropped). Considering that player {} resigned", player);
                }
            }
            // If we're here, it means we were unable to send the update, and we have to force the
            // player to resign.
            connection.resign();
            game.resign(*player);
        }
    }

    fn poll_complete_updates(&mut self) {
        debug!("flushing update channels");
        let ActiveGame {
            ref mut game,
            ref mut connections,
            ..
        } = *self;
        for (player, mut connection) in connections.iter_mut().filter(|(_, c)| !c.has_resigned()) {
            // We don't care whether it this poll return NotReady, all that matter is that we don't
            // get an error.  If we get NotReady, we know the future will be waken up when more
            // progress is possible.
            if let Err(e) = connection.updates.poll_complete() {
                // Errors are usually not recoverable, so we can't send updates to the
                // player anymore. We have to consider it resigned.
                error!("Uknown error while sending update: {}", e);
                connection.resign();
                game.resign(*player);
            }
        }
    }
}

impl Future for ActiveGame {
    type Item = (); // TODO: return the game outcome (winner, nb of turns, etc.)
    type Error = tokio_timer::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        // If we had pending updates, make sure to poll the sender to ensure progress.
        self.poll_complete_updates();
        loop {
            match self.ticks.poll()? {
                Async::Ready(Some(_instant)) => {
                    debug!("tick: updating the game");
                    self.process_players_actions();
                    self.game.incr_turn();
                    self.start_send_updates();
                    // To prevent the updates from being buffered, we call poll_complete on each
                    // sender
                    self.poll_complete_updates();
                }
                Async::Ready(None) => panic!("Unexpected end of ticks stream"),
                Async::NotReady => return Ok(Async::NotReady),
            }
        }
    }
}

pub struct PendingGame {
    pub connections: Vec<ConnectionProxy>,
    pub size: u8,
}

impl PendingGame {
    pub fn new() -> Self {
        PendingGame {
            connections: Vec::new(),
            size: 2,
        }
    }
    pub fn add(&mut self, connection: ConnectionProxy) {
        self.connections.push(connection);
    }
    pub fn is_ready(&self) -> bool {
        self.connections.len() == self.size as usize
    }
}

impl<'a> From<&'a mut PendingGame> for ActiveGame {
    fn from(pending_game: &mut PendingGame) -> Self {
        let connections: HashMap<PlayerId, ConnectionProxy> =
            FromIterator::from_iter(pending_game.connections.drain(..).enumerate());
        let game = Game::new(connections.keys().cloned().collect());
        ActiveGame {
            connections,
            game,
            ticks: Interval::new(Instant::now(), Duration::from_secs(1)),
        }
    }
}
