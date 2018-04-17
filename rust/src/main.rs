// #![feature(plugin)]
// #![plugin(rocket_codegen)]

extern crate fera_unionfind;
extern crate futures;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_timer;
extern crate tokio_tungstenite;
extern crate tungstenite;

mod connection;
mod core;
mod game;
mod server;

use server::Server;
use std::env;

fn main() {
    env_logger::init();
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string())
        .parse()
        .unwrap();
    Server::run(&addr);
}
