extern crate actix_web;
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

use std::env;
use std::thread;

use server::Server;

use actix_web::{fs::StaticFiles, middleware, server as actix_server, App};

fn main() {
    env_logger::init();
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string())
        .parse()
        .unwrap();
    thread::spawn(move || Server::run(&addr));
    actix_server::new(|| {
        App::new()
            .middleware(middleware::Logger::default())
            .handler(
                "/",
                StaticFiles::new("./static")
                    .unwrap()
                    .index_file("index.html"),
            )
            .handler("/static", StaticFiles::new("./static").unwrap())
            .finish()
    })
    .bind("127.0.0.1:8000")
    .expect("Can not bind to port 8000")
    .run();
}
