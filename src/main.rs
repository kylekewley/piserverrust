extern crate piserverrust;

use std::sync::{Arc};

use self::piserverrust::server::Server;
use self::piserverrust::parser::{ParserManager};

fn main() {
    loop {
        let parser = Arc::new(ParserManager::new());
        let mut s = Server::with_port(10142u16, parser);
        let result = s.run_forever();
        println!("Crash: {}", result.err().unwrap());
    }
}
