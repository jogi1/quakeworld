use serde::Serialize;
use crate::protocol::message::Message;
use crate::protocol::message::MessageFlags;
use crate::protocol::types::*;
use crate::protocol::errors::MvdParseError;

#[cfg(feature = "ascii_strings")]
use crate::utils::ascii_converter::AsciiConverter;



#[derive(Serialize)]
pub struct Mvd {
    pub size: usize,
    pub finished: bool,
    message: Message,
    last_to: u32,
    last_command_type: DemoCommand,
    pub frame: u32,
    pub time: f64,
}

#[derive(Serialize)]
pub struct MvdFrame {
    pub messages: Vec<ServerMessage>,
    pub frame: u32,
    pub time: f64,
    pub to: u32,
    pub command_type: DemoCommand,
}

impl MvdFrame {
    fn empty() -> MvdFrame {
        return MvdFrame {
            command_type: DemoCommand::Empty,
            messages: vec!(),
            frame: 0,
            to: 0,
            time: 0.0}
    }
}

impl Mvd {
    pub fn empty() -> Mvd {
        return Mvd {
            size:0,
            message: Message::empty(),
            finished: false,
            last_to: 0,
            last_command_type: DemoCommand::Empty,
            frame: 0,
            time: 0.0,
        }
    }

#[cfg(not(feature = "ascii_strings"))]
    pub fn new(buffer: Vec<u8>) -> Result<Mvd, std::io::Error> {
        let buffer_heap = Box::new(buffer.clone());
        let message = Message::new(buffer_heap, 0, buffer.len(), false, MessageFlags::new_empty());

        return Ok(Mvd {
            size:buffer.len(),
            message,
            finished: false,
            last_to: 0,
            last_command_type: DemoCommand::Empty,
            frame: 0,
            time: 0.0,
        })
    }

#[cfg(feature = "ascii_strings")]
    pub fn new(buffer: Vec<u8>, maybe_ascii_converter: Option<AsciiConverter>) -> Result<Mvd, std::io::Error> {
        let buffer_heap = Box::new(buffer.clone());
        let message = Message::new(buffer_heap, 0, buffer.len(), false, MessageFlags::new_empty(), maybe_ascii_converter);

        return Ok(Mvd {
            size:buffer.len(),
            message,
            finished: false,
            last_to: 0,
            last_command_type: DemoCommand::Empty,
            frame: 0,
            time: 0.0,
        })
    }

    pub fn parse_frame(&mut self) -> Result<Box<MvdFrame>, MvdParseError> {
        let mut frame = Box::new(MvdFrame::empty());
        frame.frame = self.frame;
        self.frame += 1;

        let demo_time = self.message.read_u8(false)?;
        self.time += demo_time as f64 * 0.001;
        frame.time = self.time;

        let cmd = self.message.read_u8(false)?;
        let msg_type_try = DemoCommand::try_from(cmd & 7);
        let msg_type = match msg_type_try {
            Ok(msg_type) => msg_type,
            Err(_) => return Err(MvdParseError::UnhandledCommand(cmd&7))
        };
        if msg_type == DemoCommand::Command {
            return Err(MvdParseError::QwdCommand)
        }
        frame.command_type = msg_type;
        if msg_type >= DemoCommand::Multiple && msg_type <= DemoCommand::All {
            match msg_type {
                DemoCommand::Multiple => {
                    self.last_to = self.message.read_u32(false)?;
                    self.last_command_type = msg_type;
                }
                DemoCommand::Single => {
                    self.last_to = (cmd >> 3) as u32;
                    self.last_command_type = msg_type;
                }
                DemoCommand::All => {
                    self.last_to = 0;
                    self.last_command_type = msg_type;
                }
                DemoCommand::Stats => {
                    self.last_to = 0;
                    self.last_command_type = msg_type;
                }
                DemoCommand::Command => {
                }
                DemoCommand::Empty=> {
                }
                DemoCommand::Set => {
                    // incoming
                    let _ = self.message.read_u32(false);
                    // outgoing 
                    let _ = self.message.read_u32(false);
                    return Ok(frame);
                }
                DemoCommand::Read => {
                }
            }
        }
        let mut loop_read_packet = true;
        while  loop_read_packet {
            loop_read_packet = self.read_packet(&mut frame)?;
        }
        return Ok(frame)
    }

    pub fn read_packet(&mut self, frame: &mut Box<MvdFrame>) -> Result<bool, MvdParseError> {
        let size = self.message.read_u32(false)?;
        if size == 0 {
            return Ok(false)
        }

#[cfg(feature = "ascii_strings")]
        let mut message = Message::new(self.message.buffer.clone(), self.message.position, size as usize, false, self.message.flags, Some(self.message.ascii_converter.clone()));
#[cfg(not(feature = "ascii_strings"))]
        let mut message = Message::new(self.message.buffer.clone(), self.message.position, size as usize, false, self.message.flags);
        self.message.position += size as usize;
        if self.last_command_type == DemoCommand::Multiple && self.last_to == 0 {
            return Ok(false)
        }

        loop {
            let msg_cmd = message.read_u8(false)?;

            // handle EndOfDemo
            if msg_cmd == 69 {
                let s = message.read_stringbyte(true)?;
#[cfg(feature = "ascii_strings")]
                if s.string == "ndOfDemo".to_string() {
                    self.finished =  true;
                    return Ok(false)
                }
                #[cfg(not(feature = "ascii_strings"))]
                if String::from_utf8_lossy(&s) == "ndOfDemo".to_string() {
                    self.finished =  true;
                    return Ok(false)
                }
            }
            let cmd = match ServerClient::try_from(msg_cmd) {
                Ok(cmd) => cmd,
                Err(_) => return Err(MvdParseError::UnhandledCommand(msg_cmd))
            };

            let ret = cmd.read_message(&mut message)?;

            match ret {
                ServerMessage::Serverdata(r) => {
                    frame.messages.push(ServerMessage::Serverdata(r.clone()));
                    //frame.messages.push(r.clone());
                    self.message.flags.fte_protocol_extensions = r.fte_protocol_extension;
                    self.message.flags.fte_protocol_extensions_2 = r.fte_protocol_extension_2;
                    message.flags = self.message.flags;
                }
                _ => {
                    frame.messages.push(ret);
                }
            }

            if message.position >= message.length {
                break;
            }
        }

        return Ok(false)
    }
}
