use quakeworld::protocol::message::{Message, MessageFlags, MessageType};
use quakeworld::protocol::types::{ServerClient, Print, ServerMessage};
use quakeworld::utils::trace::print_message_trace;
fn main() {
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
            ServerMessage::Print(p) => {
                assert_eq!(p.from, 2);
                assert_eq!(p.message.string, "hello");
                println!("{:?}", p);
            },
            _ => { panic!("its not print!");},
        }
        print_message_trace(&message, false)?;
    }
