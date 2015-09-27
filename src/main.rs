extern crate piserverrust;

use std::sync::{Arc, Mutex};

use self::piserverrust::server::server;
use self::piserverrust::parser::Parser;

fn main() {
    let parser = Arc::new(Parser::new());

    let mut s = server::with_port(10142u16, parser);
    s.run_forever();
}
