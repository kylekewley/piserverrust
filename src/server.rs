extern crate piservercore;

use std::thread;
use std::sync::{Arc, Mutex};
use std::io::{self, Error, ErrorKind};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

use self::piservercore::messenger::messenger;

pub struct server {
    port: u16,
    clients: Arc<Mutex<Vec<Arc<messenger>>>>
}

impl server {
    pub fn with_port(port: u16) -> server {
        server {
            port: port,
            clients: Arc::new(Mutex::new(Vec::new()))
        }
    }

    fn handle_client(clients: &mut Arc<Mutex<Vec<Arc<messenger>>>>, stream: TcpStream) {
        let mut client = Arc::new(messenger::with_connection(stream).unwrap());

        let c = client.clone();
        match clients.lock() {
            Ok(mut guard) => {
                guard.push(c);
            },
            Err(e) => {}
        }
        client.handle_client_stream();
    }

    fn run_forever(&mut self) -> Result<(), io::Error> {
        // Create a tcplistener on the set port and listen for messages
        let listener = try!(TcpListener::bind(("127.0.0.1", self.port)));

        // accept connections and process them, spawning a new thread for each one
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let mut clients = self.clients.clone();
                    thread::spawn(move|| {
                        // connection succeeded.
                        server::handle_client(&mut clients, stream);
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
