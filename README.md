generals-rs
===========

This is a toy project to learn Javascript, and play with web technologies in
general. It is a clone of [generals.io](http://generals.io).

The backend is written in Rust. It uses [Rocket](https://rocket.rs/) to serve
static content. However most of the app logic is based on WebSockets. Since
that part needs to be fully asynchronous, it is built around
[Tokio](https://tokio.rs/) and leverages
[Tungstenite](https://github.com/snapview/tokio-tungstenite) for the WebSocket
protocol.

The frontend is written in pure JavaScript (ES8) with no dependency. This
probably makes it sub-efficient and less portable, but the goal was to stay
away from all the popular frameworks, to see what JavaScrit is like.

State of the project
====================

The basic game logic is here, but it's not really usable yet and needs some
polishing on both client a server sides. Once it's done there are a couple
different things I may work on, ordered by how likely I am to actually work on
it:


- Make a python client, in order to build a bot, and train it with
  reinforcement learning. I'd like to see if we can be #1 on the official game
  (http://bot.generals.io/), thanks to machine learning.
- Record games and replay them. This could be useful to train a bot.
- Make a frontend in go with a simple 2D game framework to learn go.

Running
=======

To run the server, you'll need `rust` and it's package manager `cargo`. See the
official documentation for installation instructions. Then, it's as simple as:

```
# Run the server in debug mode
RUST_LOG=generals_rs=debug cargo run
# Run the server in release mode
RUST_LOG=generals_rs=debug cargo run --release
```

The server will listen on http://localhost:8000. Open two different tabs to
start a game.


Testing
=======

To test the server, run `cargo test`. To run the client tests, open
`static/tests/index.html` in the browser.
