extern crate piservercore;
extern crate capnp;

use std::thread;
use std::sync::{Arc, Mutex};
use std::io::{self, Error, ErrorKind};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::cell::RefCell;

use capnp::message::{MessageReader, ReaderOptions, MallocMessageBuilder, MessageBuilder};

use self::piservercore::messenger::Messenger;
use self::piservercore::parser::Parser;

pub struct server {
    port: u16,
    clients: Arc<Mutex<Vec<(u32, Arc<Mutex<Vec<MallocMessageBuilder>>>)>>>,
    parser: Arc<Parser>,
    current_id: u32
}

impl server {
    pub fn with_port(port: u16, parser: Arc<Parser>) -> server {
        server {
            port: port,
            clients: Arc::new(Mutex::new(Vec::new())),
            parser: parser,
            current_id: 0
        }
    }

    fn handle_client(clients: &mut Arc<Mutex<Vec<(u32, Arc<Mutex<Vec<MallocMessageBuilder>>>)>>>, client_id: u32, mut parser: Arc<Parser>, stream: TcpStream) {
        let mut client = Messenger::with_connection(stream, parser).unwrap();

        let mut oqueue = client.get_oqueue();

        clients.lock().unwrap().push((client_id, oqueue));

        client.handle_client_stream();


        // Client disconnected. Remove from clients list
        match clients.lock() {
            Ok(mut guard) => {
                for i in 0..guard.len() {
                    let (id, _) = guard[i];

                    if id == client_id {
                        guard.swap_remove(i);
                        break;
                    }
                }
            },
            Err(e) => {}
        }
    }

    pub fn run_forever(&mut self) -> Result<(), io::Error> {
        // Create a tcplistener on the set port and listen for messages
        let listener = try!(TcpListener::bind(("127.0.0.1", self.port)));

        // accept connections and process them, spawning a new thread for each one
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let mut clients = self.clients.clone();
                    let mut parser = self.parser.clone();

                    let client_id = self.current_id;
                    self.current_id += 1;

                    thread::spawn(move|| {
                        // connection succeeded.
                        server::handle_client(&mut clients, client_id, parser, stream);
                    });
                }
                Err(e) => { /* connection failed */ }
            }
        }

        return Err(Error::new(ErrorKind::Other, "Unknown error. Server broke from loop"));
    }

    fn restart_server(&mut self) {
    }
}
