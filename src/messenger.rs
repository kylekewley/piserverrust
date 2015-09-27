/**
 * This defines the struct used for sending and recieving messages from a byte stream
 *
 * Messages are sent out using the Message_capnp struct preceeded by 4 bytes defining the length of
 * the message not including the four bytes. These bytes are sent in big endian byte order.
 */
use rustc_serialize::json;

use std::io::{self, Write, Read, Result, Error, ErrorKind, BufReader, BufRead, BufWriter};
use std::sync::mpsc::{TryRecvError, channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::vec::Vec;
use std::thread;
use std::net::{TcpStream, Shutdown};

use parser::Parser;

use message::Message;
const PREFIX_SIZE: usize = 4;


#[allow(dead_code)]
pub struct Messenger {
    ostream: Arc<Mutex<TcpStream>>,
    istream: TcpStream,
    oqueue: Arc<Mutex<Vec<Message>>>,
    parser: Arc<Parser>,
}

impl Messenger {
    pub fn with_connection(client: TcpStream, parser: Arc<Parser>) -> Result<Messenger> {
        let ostream = Arc::new(Mutex::new(try!(client.try_clone())));
        let istream = try!(client.try_clone());
        let oqueue: Arc<Mutex<Vec<Message>>> = Arc::new(Mutex::new(Vec::new()));

        Ok(Messenger {
            ostream: ostream,
            istream: istream,
            oqueue: oqueue,
            parser: parser
        })
    }

    pub fn get_oqueue(&self) -> Arc<Mutex<Vec<Message>>> {
        return self.oqueue.clone();
    }

    /**
     * This function is called on a new thread by the main thread for the
     * individual client. It will block until a message is recieved or there is an io error and
     * return the result.
     */
    pub fn recv_message(istream: &mut Read) -> Result<String> {
        let size = try!({
            Messenger::read_message_size(istream)
        });

        // Create a take with the total number of bytes needed
        let mut stream = istream.take(size as u64);

        let mut tmp = Vec::new();
        let read_len = try!(stream.read_to_end(&mut tmp)) as u64;

        println!("Vec length: {}", tmp.len());
        let recv_str = String::from_utf8(tmp).unwrap();

        println!("Message read with length: {} {}", recv_str.len(), recv_str);

        if read_len < size {
            return Err(Error::new(ErrorKind::InvalidInput, "Not enough bytes in the stream"));
        }

        Ok(recv_str)
    }

    /// Read the first PREFIX_SIZE bytes from the stream interpreted as big endian
    /// Return the message size, or an error if there is an io error.
    pub fn read_message_size(stream: &mut Read) -> Result<u64> {
        println!("Reading message size");
        let mut stream = stream.take(PREFIX_SIZE as u64);

        // Shift each byte into a u64 up to PREFIX_SIZE bytes
        let mut length: u64 = 0;

        let mut bytes: Vec<u8> = Vec::new();
        println!("Attempting take read");
        let byte_count = stream.read_to_end(&mut bytes).unwrap();
        println!("Read to end of take");

        // bytes.reverse();

        for byte in bytes {
            length = length<<8;
            length |= byte as u64;
        }

        println!("Message length read: {}", length);


        if byte_count != PREFIX_SIZE {
            println!("Error reading message length");
            return Err(Error::new(ErrorKind::InvalidInput, "Couldn't read message length from stream"));
        }

        return Ok(length)
    }

    /// Write the size of the message into the output stream
    fn write_message_size(stream: &mut Write, size: u32) -> Result<()> {
        let mut buffer = [0u8; PREFIX_SIZE];

        let mut size = size;
        let mask = 0x000000FFu32;

        for i in (0..PREFIX_SIZE).rev() {
            // Add each byte to the buffer
            buffer[i] = (size & mask) as u8;
            size = size >> 8;
        }

        stream.write_all(& buffer)
    }

    pub fn send_message<T: Write>(ostream: &mut T, message: &Message) -> Result<()> {
        let message_str = json::encode(&message).unwrap();
        let message_size = message_str.len();
        Messenger::write_message_size(ostream, message_size as u32).unwrap();
        ostream.write_all(message_str.as_bytes()).unwrap();
        ostream.flush()
    }

    pub fn add_to_send_queue(&mut self, message: Message) {
        let mut queue = self.oqueue.lock().unwrap();
        queue.push(message);
    }

    /// This call loops forever, creating a new thread to handle reading from
    /// the stream. The blocking thread will handle messages as they come in, 
    /// and write new messages when they are added to the queue.
    pub fn handle_client_stream(&mut self) -> Result<()> {
        let (tx, rx) = channel();

        let mut istream = self.istream.try_clone().unwrap();

        // Spawn a new thread to listen to incoming messages
        let handle = thread::spawn(move|| {
            loop {
                let m = Messenger::recv_message(&mut istream).unwrap();
                match tx.send(m) {
                    Ok(()) => {},
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }
        });

        let mut errorMessage = String::new();

        loop {
            {
                // Send the first item in the queue
                let message = {
                    let mut queue = self.oqueue.lock().unwrap();
                    queue.pop()
                };

                match message {
                    Some(mut m) => {
                        let ostream = self.ostream.lock();
                        match ostream {
                            Ok(mut guard) => {
                                Messenger::send_message(&mut *guard, &mut m).unwrap();
                            },
                            Err(_) => { 
                                errorMessage.push_str("ostream mutex poisoned\n");
                                break;
                            }
                        }
                    },
                    None => { /* Nothing in the queue */}
                }
            }

            // Try to recieve messages
            match rx.try_recv() {
                Ok(r) => {
                    let message: Message = json::decode(&r).unwrap();
                    let response = self.parser.parse_message(&message);
                    if response.is_some() {
                        println!("Sending response");
                        self.add_to_send_queue(response.unwrap());
                    }
                },
                Err(TryRecvError::Empty) => {},
                Err(e) => {
                    println!("Error reading message: {}", e);
                    errorMessage.push_str("Error receiving message\n");
                    break;
                }
            }
        }

        self.istream.shutdown(Shutdown::Both);
        handle.join();
        if errorMessage.len() != 0 {
            return Err(Error::new(ErrorKind::Other, errorMessage));
        }

        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use message::Message;
    use rustc_serialize::{json, Encodable, Decodable};
    use std::io::{Cursor, Read, Write};

    use bufstream::BufStream;
    use core_messages::Ping;

    #[test]
    fn test_write_stream() {
        let cursor = Cursor::new(Vec::new());
        let mut stream = BufStream::new(cursor);

        let message = "Hello World";
        let message_size = message.len();

        let write_size = stream.write(message.as_bytes()).unwrap();
        stream.flush().unwrap();

        assert_eq!(message_size, write_size);

        stream.get_mut().set_position(0);

        let mut s = String::new();
        let read_size = stream.read_to_string(&mut s).unwrap();

        assert_eq!(read_size, message_size);
        assert_eq!(message, s);
    }

    /*
    #[test]
    fn test_excess_size() {
        let cursor = Cursor::new(Vec::new());
        let mut stream = BufStream::new(cursor);

        let message = "Hello World";
        let incorrect_size = message.len()+1;

        Messenger::write_message_size(&mut stream, incorrect_size as u32);
        let write_size = stream.write(message.as_bytes()).unwrap();
        stream.flush().unwrap();

        stream.get_mut().set_position(0);

        // Read the message out of the stream
        let serialized_string = Messenger::recv_message(&mut stream).unwrap();

        println!("Decoded String: {}", serialized_string);
    }
    */

    fn test_send_message(message: &Message) -> BufStream<Cursor<Vec<u8>>> {
        let cursor = Cursor::new(Vec::new());
        let mut stream = BufStream::new(cursor);

        Messenger::send_message(&mut stream, &message).unwrap();

        // return the cursor postion back to 0 for reading
        stream.get_mut().set_position(0);

        stream
    }


    fn generate_ping_message() -> Message {
        let ping = Ping::new();
        Message::with_message(&ping, 1)
    }


    #[test]
    fn send_message() {
        let ping = generate_ping_message();

        // Send the message into the stream
        let mut stream = test_send_message(&ping);

        // Read the message out of the stream
        let serialized_string = Messenger::recv_message(&mut stream).unwrap();

        let recv_ping: Message = json::decode(&serialized_string).unwrap();

        assert_eq!(ping, recv_ping);
    }
}
