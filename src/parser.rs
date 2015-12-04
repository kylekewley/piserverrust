/**
 * This defines the struct to be used for tracking the functions used for responding to different
 * messeges based on their message ID. Functions can be registered for certain IDs and when a
 * message comes in with that ID, it will be passed to the proper function.
 */

use std::collections::{HashMap};
use std::sync::{Arc, Mutex};

use message::Message;


pub trait Parser {
    fn parse_message(&self, message: &Message) -> Option<Message>;
    fn get_id(&self) -> u32;
}

pub struct ParserManager<'a> {
    parsers: HashMap<u32, Box<Parser+Send+Sync + 'a>>
}


impl <'a>ParserManager<'a> {
    pub fn new() -> ParserManager<'a> {
        ParserManager { parsers: HashMap::new() }
    }

    pub fn parse_message(&self, message: &Message) -> Option<Message> {
        let parser_id = message.get_parser_id();
        let f = self.parsers.get(&parser_id);

        if f.is_some() {
            let f = f.unwrap();
            let result = f.parse_message(message);

            if message.get_ack() && !result.is_some() {
                let empty_message = ();
                return Some(Message::with_reply(&empty_message, message));
            }

            return result;
        }

        // No parser registered for the ID. Ignore the message
        None
    }

    /**
      * Register the function to be called for a message with the given parser id
      *
      * @return true if the parser was registered. false if a parser already exists for the given
      * ID
      */

    pub fn register_parser<T: 'a+Parser+Send+Sync>(&mut self, parser: T) -> bool {
        let parser_id = parser.get_id();

        if self.parsers.contains_key(&parser_id) {
            return false;
        }

        self.parsers.insert(parser_id, Box::new(parser));

        true
    }
}
