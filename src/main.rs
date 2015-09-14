extern crate piserverrust;
extern crate piservercore;

use std::sync::{Arc, Mutex};

use piserverrust::server::server;
use self::piservercore::parser::Parser;

fn main() {
    let parser = Arc::new(Parser::new());

    let mut s = server::with_port(10150u16, parser);
    s.run_forever();
}
