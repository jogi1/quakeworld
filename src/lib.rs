//! library for doing quakeworld things
#[macro_use]
extern crate lazy_static;
extern crate simple_error;
extern crate paste;
extern crate quote;
extern crate unstringify;

#[cfg(feature = "utils")]
pub mod utils;
#[cfg(feature = "protocol")]
pub mod protocol;
#[cfg(feature = "mvd")]
pub mod mvd;
#[cfg(feature = "state")]
pub mod state;
#[cfg(feature = "network")]
pub mod network;

#[cfg(feature = "crc")]
pub mod crc;

#[cfg(feature = "pak")]
pub mod pak;

#[cfg(test)]
mod tests {
    use crate::utils::ascii_converter::AsciiConverter;
    #[test]
    fn ascii_converter() {
        let ascii_converter = AsciiConverter::new();
        let b: Vec<u8> = vec![177, 178, 179, 180];
        let s = ascii_converter.convert(b);
        assert_eq!(s, "1234".to_string());
    }

    use crate::protocol::message::{Message, MessageFlags, MessageType};
    use crate::protocol::types::ServerClient;
    #[test]
    fn message_parsing() {
        let b: Vec<u8> = vec![8, 2, 0x68, 0x65, 0x6c, 0x6c, 0x6f,0x0];
        let mut message = Message::new(Box::new(b.clone()), 0, b.len(), false, MessageFlags {..Default::default()}, None, MessageType::Connection);
        message.trace.enabled = true;
        let msg_cmd = match message.read_u8(false) {
            Ok(cmd) => cmd,
            Err(e) => panic!("{}", e),
        };
        let cmd = match ServerClient::try_from(msg_cmd) {
            Ok(cmd) => cmd,
            Err(_) => panic!("failed reading print cmd"), 
        };

        let ret = match cmd.read_message(&mut message) {
            Ok(cmd) => cmd,
            Err(_) => panic!("failed reading print"), 
        };
        match ret {
            crate::protocol::types::ServerMessage::Print(p) => {
                assert_eq!(p.from, 2);
                assert_eq!(p.message.string, "hello");
            },
            _ => { panic!("its not print!");},
        }
    }
}
