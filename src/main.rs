extern crate piserverrust;

use piserverrust::server::server;

fn main() {
    let s = server::with_port(1024u16);
    println!("Hello, world!");
}
