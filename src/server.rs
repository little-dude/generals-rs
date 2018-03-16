use std::mem;

struct PendingGame {
    players: Vec<Player>,
    threshold: usize,
}

impl PendingGame {
    fn new(threshold: usize) -> Self {
        PendingGame {
            players: Vec::new(),
            threshold,
        }
    }

    fn ready(&self) -> bool {
        self.threshold == self.players.len()
    }
}

struct Server {
    handle: Handle,
    running_games: Vec<Games>,
    pending_game: PendingGame,
    incoming_players: mpsc::UnboundedReceiver<Player>,
}

impl Future for Server {
    type Item = ();
    type Error = ();
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.incoming_players.poll().expect("polling failed") {
            Async::Ready(Some(player)) => {
                self.pending_game.add_player(player);
                if self.pending_game.ready() {
                    let new_pending_game = Game::new(2);
                    let game = mem::replace(self.pending_game, new_pending_game);
                    self.start_game(game);
                }
            }
            Async::Ready(None) => panic!("channel closed."),
            Async::NotReady => return Async::NotReady,
        }
    }
}

enum PlayerState {
    Playing,
    Waiting,
    Idle,
}

enum Action {
    Move(Move),
    Cancel,
    GiveUp,
}

struct Player {
    name: String,
    actions: mpsc::UnboundedSender<Action>,
    state: PlayerState,
}
