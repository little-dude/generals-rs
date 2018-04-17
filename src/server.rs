use std::io::{self, ErrorKind};
use std::mem;
use std::net::SocketAddr;

use futures::stream::Stream;
use futures::sync::mpsc::{channel, Receiver, Sender};
use futures::{Async, AsyncSink, Future, Poll, Sink};
use tokio_core::net::TcpListener;
use tokio_core::reactor::{Core, Handle};
use tokio_tungstenite::accept_async;

use connection::{Connection, ConnectionProxy};
use game::{ActiveGame, PendingGame};

pub struct Server {
    pending_game: PendingGame,
    connections: Receiver<ConnectionProxy>,
    handle: Handle,
}

impl Server {
    pub fn run(addr: &SocketAddr) {
        let mut core = Core::new().unwrap();
        let handle = core.handle();
        let (connection_tx, connection_rx) = channel(100);
        let server = Server {
            pending_game: PendingGame::new(),
            connections: connection_rx,
            handle: handle.clone(),
        };
        handle.spawn(server.map_err(|e| error!("{}", e)));

        let socket = TcpListener::bind(&addr, &handle).unwrap();
        println!("Listening on: {}", addr);

        let srv = socket.incoming().for_each(|(stream, addr)| {
            debug!("new tcp connection {}", addr);
            let connection_tx = connection_tx.clone();
            let handle = handle.clone();
            accept_async(stream).and_then(move |ws| {
                debug!("{}: websocket open", addr);
                let (connection, connection_proxy) = Connection::new(ws);
                handle.spawn(connection.map_err(|_| ()));
                SendFuture::new(connection_tx).send(connection_proxy).or_else(|()| {
                    error!("failed to send connection proxy to the server");
                    Ok(())
                })
            })
            // The future as type Result<_, tungstenite::Error>, but socket.incoming() returns a
            // future of type Result<_, io::Error> so we have to convert back.
            .map_err(|e| io::Error::new(ErrorKind::Other, format!("{}", e)))
        });
        core.run(srv.map_err(|_| ())).unwrap();
    }
}

impl Future for Server {
    type Item = ();
    type Error = String;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let Server {
            ref mut connections,
            ref mut pending_game,
            ref handle,
        } = *self;
        loop {
            match connections
                .poll()
                .map_err(|()| "Failed to get new connections")?
            {
                Async::Ready(Some(connection)) => {
                    info!("adding client to pending game");
                    pending_game.add(connection);
                    if !pending_game.is_ready() {
                        continue;
                    }
                    info!("pending game is ready, starting the game");
                    let mut ready = mem::replace(pending_game, PendingGame::new());
                    let mut new_game = ActiveGame::from(&mut ready);
                    // FIXME: I'm not 100% we can do that here, before spawning thegame
                    new_game.start_send_updates();
                    handle.spawn(new_game.map_err(|e| error!("{}", e)));
                }
                Async::Ready(None) => return Err("Connection proxy sender dropped".into()),
                Async::NotReady => return Ok(Async::NotReady),
            }
        }
    }
}

// XXX: should be add a timeout to the sender?
struct SendFuture<T> {
    tx: Sender<T>,
    item: Option<T>,
}

impl<T> SendFuture<T> {
    fn new(sender: Sender<T>) -> Self {
        SendFuture {
            tx: sender,
            item: None,
        }
    }
    fn send(mut self, item: T) -> Self {
        self.item = Some(item);
        self
    }
}

impl<T> Future for SendFuture<T> {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Some(item) = self.item.take() {
            match self.tx.start_send(item).map_err(|_| ())? {
                AsyncSink::NotReady(item) => {
                    self.item = Some(item);
                    // XXX: should we have a counter and error out after a few tries?
                    Ok(Async::NotReady)
                }
                // FIXME: does poll_complete flush for all the producers or just for this one? If
                // it's for all the producers, maybe we should not call it too often, and instead
                // call it regularly after a certain amount of time?
                AsyncSink::Ready => self.tx.poll_complete().map_err(|_| ()),
            }
        } else {
            self.tx.poll_complete().map_err(|_| ())
        }
    }
}
