use rustc_serialize::{json, Encodable, Decodable};
use std::fmt;

#[derive(RustcDecodable, RustcEncodable, Debug, Eq, PartialEq)]
pub struct Ping {
    relay_count: u32,
    total_relays: u32,
    delay_ms: u32,
}

impl Ping {
    pub fn new() -> Ping {
        Ping {
            relay_count: 0,
            total_relays: 0,
            delay_ms: 0,
        }
    }

    pub fn with_relay(total_relays: u32, delay_ms: u32) -> Ping {
        Ping {
            relay_count: 0,
            total_relays: total_relays,
            delay_ms: delay_ms,
        }
    }

}
