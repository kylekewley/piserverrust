use std::thread;
use std::sync::{Arc, Mutex};
use std::io::{self, Error, ErrorKind};
use std::net::{Shutdown, TcpListener, TcpStream, ToSocketAddrs};
use std::cell::RefCell;
use std::ops::Deref;

use messenger::Messenger;
use message::Message;
use parser::ParserManager;

pub struct Server {
    port: u16,
    clients: Arc<Mutex<Vec<(u32, Arc<Mutex<Vec<Message>>>)>>>,
    parser: Arc<ParserManager<'static>>,
    current_id: u32
}

impl Server {
    pub fn with_port(port: u16, parser: Arc<ParserManager<'static>>) -> Server {
        Server {
            port: port,
            clients: Arc::new(Mutex::new(Vec::new())),
            parser: parser,
            current_id: 0
        }
    }

    pub fn broadcast_message(&mut self, message: Message) {
        {
            let clients = self.clients.lock().unwrap();
            let clients = clients.deref();

            for client in clients {
                let ref queue = client.1;
                let mut queue = queue.lock().unwrap();
                queue.push(message.clone());
            }
        }
    }

    fn handle_client(clients: &mut Arc<Mutex<Vec<(u32, Arc<Mutex<Vec<Message>>>)>>>, client_id: u32, parser: Arc<ParserManager>, stream: TcpStream) {
        let mut client = Messenger::with_connection(stream, parser).unwrap();

        let oqueue = client.get_oqueue();

        clients.lock().unwrap().push((client_id, oqueue));

        let client_result = client.handle_client_stream();

        println!("Client disconnected");

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
            Err(e) => {
                println!("Error: {}", e);
            }
        }

        client_result.unwrap();
    }

    pub fn run_forever(&mut self) -> Result<(), io::Error> {
        // Create a tcplistener on the set port and listen for messages
        let listener = try!(TcpListener::bind(("0.0.0.0", self.port)));

        println!("Listening on port {}", self.port);

        // accept connections and process them, spawning a new thread for each one
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("New Connection");
                    let mut clients = self.clients.clone();
                    let parser = self.parser.clone();

                    let client_id = self.current_id;
                    self.current_id += 1;

                    thread::spawn(move|| {
                        // connection succeeded.
                        Server::handle_client(&mut clients, client_id, parser, stream);
                        println!("Exiting thread");
                    });
                }
                Err(e) => {
                    /* connection failed */ 
                    println!("Error: {}", e);
                }
            }
        }

        return Err(Error::new(ErrorKind::Other, "Unknown error. Server broke from loop"));
    }

    fn restart_server(&mut self) {
    }
}
