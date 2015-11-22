/**
 * This defines the class that will be used to encode/decode general messages
 */
use rustc_serialize::{json, Encodable, Decodable};
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

#[derive(RustcDecodable, RustcEncodable, Debug, Eq, PartialEq)]
pub struct Message {
    ack: bool,      // Determines whether this message requires the reciever to acknowledge
    m_id: u32,      // Message ID. Just a unique integer
    p_id: u32,      // ID of the parser that will parse this message
    message: String, // The actual message payload
}

const REPLY_MESSAGE_PARSER_ID: u32 = 0;

static GLOBAL_MESSAGE_ID: AtomicUsize = ATOMIC_USIZE_INIT;


impl Clone for Message {
    fn clone(&self) -> Self {
        let mut clone = Message::new();
        clone.message = self.message.clone();
        clone.ack = self.ack;
        clone.p_id = self.p_id;

        return clone;
    }
}

impl Message {
    pub fn new() -> Message {
        Message {
            ack: false,
            m_id: Message::next_id(),
            p_id: 0,
            message: String::new(),
        }
    }

    pub fn with_reply<T: Encodable>(message: &T, message_id: u32) -> Message {
        let encoded_message = json::encode(&message).unwrap();
        Message {
            ack: false,
            m_id: message_id,
            p_id: REPLY_MESSAGE_PARSER_ID,
            message: encoded_message,
        }
    }

    pub fn with_message<T: Encodable>(message: &T, parser_id: u32) -> Message {
        let encoded_message = json::encode(&message).unwrap();
        Message {
            ack: false,
            m_id: Message::next_id(),
            p_id: parser_id,
            message: encoded_message,
        }
    }

    pub fn get_message(&self) -> &String {
        &self.message
    }

    pub fn set_ack(&mut self, ack: bool) {
        self.ack = ack;
    }

    pub fn set_message<T: Encodable>(&mut self, message: &T) {
        let encoded_message = json::encode(&message).unwrap();
        self.message = encoded_message;
    }

    pub fn set_parser_id(&mut self, parser_id: u32) {
        self.p_id = parser_id;
    }

    fn next_id() -> u32 {
        GLOBAL_MESSAGE_ID.fetch_add(1, Ordering::SeqCst) as u32
    }


    pub fn get_parser_id(&self) -> u32 {
        self.p_id
    }
}

mod tests {
    use super::*;
    use std::collections::{HashMap};
    use rustc_serialize::json;
    use core_messages::Ping;


    #[test]
    fn test_encode() {

        let payload = Ping::new();
        let payload_str = json::encode(&payload).unwrap();

        let pimessage = Message {
            ack: true,
            m_id: 2,
            p_id: 2,
            message: payload_str
        };

        let encoded = json::encode(&pimessage).unwrap();

        let decode: Message = json::decode(&encoded).unwrap();
        let decoded_payload: Ping = json::decode(&decode.message).unwrap();

        assert_eq!(decode, pimessage);
        assert_eq!(decoded_payload, payload);
    }
}

