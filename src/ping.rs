use rustc_serialize::{json, Encodable, Decodable};

#[derive(RustcDecodable, RustcEncodable)]
pub struct Ping {
    relay_count: u32,
}
