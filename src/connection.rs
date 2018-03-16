use std::collections::VecDeque;
use std::error::Error;
use std::fmt;

use futures::stream::Stream;
use futures::sync::mpsc::{channel, Receiver, Sender};
use futures::{Async, Sink};
use futures::{Future, Poll};
use tokio_io::{AsyncRead, AsyncWrite};

use tokio_tungstenite::WebSocketStream;
use tungstenite::Error as WebSocketError;
use tungstenite::Message;

use core::{Action, Move, Update};

use serde_json;

pub struct Connection<S> {
    ws: WebSocketStream<S>,
    actions: Sender<Action>,
    updates: Receiver<Update>,
}

impl<S> Future for Connection<S>
where
    S: AsyncRead + AsyncWrite,
{
    type Item = ();
    type Error = ConnectionError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        // Implement some sort of backpressure: `Sink::poll_complete()` returns `Async::Ready(())`
        // if all the outgoing messages have been sent, and `Async::NotReady` if that's not the
        // case. When we get `Async::NotReady`, we have the guarantee that the task is scheduled to
        // wake up when there is more progress possible (per the documentation[0]). Since we have
        // such a guarantee, we can return `Async::NotReady` without polling the websocket for
        // incoming messages.
        //
        // [0]https://docs.rs/futures/0.1/futures/sink/trait.Sink.html#return-value-1
        if let Async::NotReady = self.ws.poll_complete()? {
            warn!("Websocket is busy processing outgoing messages. Postponing processing of incoming messages.");
            return Ok(Async::NotReady);
        }

        self.process_updates()?;
        self.process_new_messages()
    }
}

impl<S> Connection<S>
where
    S: AsyncRead + AsyncWrite,
{
    pub fn new(ws: WebSocketStream<S>) -> (Self, ConnectionProxy) {
        let (action_tx, action_rx) = channel(10);
        let (update_tx, update_rx) = channel(10);
        let connection = Connection {
            ws,
            actions: action_tx,
            updates: update_rx,
        };
        let proxy = ConnectionProxy {
            actions: action_rx,
            updates: update_tx,
            pending_moves: VecDeque::new(),
            resigned: false,
        };
        (connection, proxy)
    }
    /// Start processing messages from the client
    fn process_new_messages(&mut self) -> Poll<(), ConnectionError> {
        loop {
            match self.ws.poll()? {
                Async::Ready(Some(msg)) => self.handle_message(msg),
                Async::Ready(None) => {
                    return Err(ConnectionError::Internal("Websocket disconnected".into()))
                }
                Async::NotReady => return Ok(Async::NotReady),
            }
        }
    }

    /// Send the pending updates to the client
    fn process_updates(&mut self) -> Poll<(), ConnectionError> {
        loop {
            match self.updates
                .poll()
                .map_err(|()| ConnectionError::Internal("Failed to poll update channel".into()))?
            {
                Async::Ready(Some(update)) => {
                    // FIXME: handle errors
                    let msg = Message::Text(serde_json::to_string(&update).unwrap());
                    self.ws.start_send(msg)?;
                }
                Async::Ready(None) => {
                    return Err(ConnectionError::Internal("Updates channel closed".into()))
                }
                Async::NotReady => return Ok(Async::NotReady),
            }
        }
    }

    fn handle_message(&mut self, msg: Message) {
        if let Message::Text(string) = msg {
            match serde_json::from_str(&string) {
                Ok(mut action) => {
                    // If the channel is full already, discard the message
                    if self.actions.start_send(action).is_err() {
                        error!("Discarding action from client");
                    }
                }
                Err(e) => {
                    error!("Could not deserialize message: {} (err: {})", string, e);
                }
            }
        } else {
            error!("Unsupported message {:?}", msg);
        }
    }
}

pub struct ConnectionProxy {
    pub actions: Receiver<Action>,
    pub updates: Sender<Update>,
    pub pending_moves: VecDeque<Move>,
    pub resigned: bool,
}

impl ConnectionProxy {
    pub fn poll_actions(&mut self) {
        loop {
            let ConnectionProxy {
                ref mut actions,
                ref mut pending_moves,
                ..
            } = *self;

            match actions.poll() {
                Ok(Async::Ready(Some(Action::CancelMoves))) => pending_moves.truncate(0),
                Ok(Async::Ready(Some(Action::Resign))) => break,
                Ok(Async::Ready(Some(Action::Move(mv)))) => pending_moves.push_back(mv),
                Ok(Async::NotReady) => return,
                Ok(Async::Ready(None)) => {
                    warn!("remote end of actions channel closed");
                    // We treat this as if we received Action::Resigned because we won't be able to
                    // get the player's next moves anyway.
                    break;
                }
                Err(()) => {
                    error!("failed to get actions from connection");
                    // We treat this as if we received Action::Resigned because we won't be able to
                    // get the player's next moves anyway.
                    break;
                }
            }
        }
        // If we're after the loop, the player resigned.
        self.resign();
    }

    pub fn resign(&mut self) {
        self.resigned = true;
        self.pending_moves.truncate(0);
    }

    pub fn get_move(&mut self) -> Option<Move> {
        self.pending_moves.pop_front()
    }

    pub fn has_resigned(&self) -> bool {
        self.resigned
    }
}

#[derive(Debug)]
pub enum ConnectionError {
    WebSocket(WebSocketError),
    Internal(String),
}

impl From<WebSocketError> for ConnectionError {
    fn from(err: WebSocketError) -> Self {
        ConnectionError::WebSocket(err)
    }
}

impl Error for ConnectionError {
    fn description(&self) -> &str {
        match *self {
            ConnectionError::WebSocket(ref e) => e.description(),
            ConnectionError::Internal(ref s) => s,
        }
    }

    fn cause(&self) -> Option<&Error> {
        if let ConnectionError::WebSocket(ref e) = *self {
            Some(e)
        } else {
            None
        }
    }
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConnectionError::WebSocket(e) => {
                write!(f, "Connection error (websocket): {}", e.description())
            }
            ConnectionError::Internal(s) => write!(f, "Connection error (internal): {}", s),
        }
    }
}
