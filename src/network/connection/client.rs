use thiserror::Error;
use crate::network::channel::Channel;
use crate::protocol::message::Message;
use crate::protocol::message::MessageFlags;
use crate::protocol::message::MessageType;
use crate::protocol::types::{Packet, ProtocolVersion, ServerMessage, ClientServer, Serverdata, DeltaUserCommand};
use crate::utils::userinfo::Userinfo;

use crate::crc::generate_checksum;

use crate::utils::ascii_converter::AsciiConverter;

use serde::Serialize;


#[derive(Error, Debug, Serialize)]
pub enum ClientError {
    #[error("Unhandled Packet")]
    UnhandledPacket
}


#[derive(Default, Serialize, PartialEq, Eq)]
pub enum ClientConnectionState {
    #[default] Initialized,
    ConnectionNegotiatonChallengeSend,
    ConnectionNegotiatonChallengeRecieved,
    ConnectionNegotiatonConnectionAccepted,
    Connected,
    Disconnected,
    ErrorState
}

#[derive(Default, Serialize)]
pub struct Client {
    pub ip: String,
    pub local_port: u16,
    pub channel: Channel,
    pub state: ClientConnectionState,
    pub userinfo: Userinfo,
    pub protocol: MessageFlags,
    pub serverdata: Serverdata,
    pub prespawn_send: bool,
    pub map_crc: u32,
}

#[derive(Default, Serialize)]
pub struct ClientStatus {
    pub packet: Option<Packet>,
    pub response: Option<Vec<u8>>,
}

pub fn print_seq(out: bool, first: u32, second: u32) {
    if out {
        print!("--> ");
    } else {
        print!("<-- ");
    }
    println!("s={}({}) a={}({})",
    first & !(1 << 31), (first & (1 << 31)) != 0, 
    second & !(1 << 31), (second & (1 << 31)) != 0);
}

impl Client {
    pub fn new(ip: String, ascii_converter: AsciiConverter) -> Client {
        let mut userinfo = Userinfo::new_with_ascii_converter(ascii_converter);
        userinfo.update_from_string("*rust_quakeworld", env!("CARGO_PKG_VERSION"));
        Client {
            ip,
            userinfo,
            ..Default::default()
        }
    }

    // setup connection and return challenge packet
    pub fn connect(&mut self, port: u16) -> Vec<u8> {
        self.local_port = port;
        self.get_challenge()
    }


    fn write_empty_move_cmd(&mut self, message: &mut Message) ->Result<(), Box<dyn std::error::Error>> {
        message.write_u8(ClientServer::Move as u8);
        let position = message.position;
        message.write_u8(0); // checksum
        message.write_u8(0); // pl
        // usercmd_t, thrice 
        message.write_delta_usercommand(
            DeltaUserCommand{
                msec: Some(0),
                ..Default::default()});
        message.write_delta_usercommand(
            DeltaUserCommand{
                msec: Some(0),
                ..Default::default()});
        message.write_delta_usercommand(
            DeltaUserCommand{
                msec: Some(0),
                ..Default::default()});

        let crc = generate_checksum(message.clone(), position+1, message.position, self.channel.outgoing.sequence);
        message.replace_at_position([(crc & 0xff) as u8], position)?;
        Ok(())
    }

    pub fn handle_timeout(&mut self) -> Result<ClientStatus, Box<dyn std::error::Error>> {
        let mut message = Message::empty();
        if self.state == ClientConnectionState::ConnectionNegotiatonChallengeSend {
            return Ok(ClientStatus {
                response: Some(self.get_challenge()),
                packet: None});
        } else if self.state != ClientConnectionState::Connected {
            return Ok(ClientStatus { 
                response: None ,
                packet: None });
        }
        let (out, ack) = self.channel.unreliable();
        message.write_u32(out);
        message.write_u32(ack);
        message.write_u16(self.local_port);
        self.write_empty_move_cmd(&mut message)?;
        Ok(ClientStatus {
            response: Some(*message.buffer.clone()),
            packet: None})
    }

    pub fn handle_packet(&mut self, packet: Vec<u8>) -> Result<ClientStatus, Box<dyn std::error::Error>> {
        let mut message = Message::new(Box::new(packet.clone()), 0, packet.len(), false, self.protocol, None, MessageType::Connection);
        message.trace.enabled = true;
        let p = match message.read_packet() {
            Ok(p) => p,
            Err(e) => {
                return Err(Box::new(e));
            }
        };

        match p {
            Packet::ConnectionLessServerChallenge(t) => {
                self.state = ClientConnectionState::ConnectionNegotiatonChallengeRecieved;
                self.protocol = t.protocol;

                let message = format!("connect 28 {} {} \"", self.local_port, t.challenge);
                let mut msg_bytes = message.into_bytes();
                let mut msg = vec!(0xff, 0xff, 0xff, 0xff);
                msg.append(&mut msg_bytes);
                let ui = self.userinfo.as_bytes();
                msg.extend(ui);
                msg.extend(b"\"\n");

                if t.protocol.fte_protocol_extensions.bits() != 0 {
                    let fte = format!("{:#01x} {:#01x}\n", ProtocolVersion::Fte as u32, t.protocol.fte_protocol_extensions.bits());
                    msg_bytes = fte.into_bytes();
                    msg.append(&mut msg_bytes);
                }

                if t.protocol.fte_protocol_extensions_2.bits() != 0 {
                    let fte = format!("{:#01x} {:#01x}\n", ProtocolVersion::Fte2 as u32, t.protocol.fte_protocol_extensions_2.bits());
                    msg_bytes = fte.into_bytes();
                    msg.append(&mut msg_bytes);
                }

                if t.protocol.mvd_protocol_extension.bits() != 0 {
                    let fte = format!("{:#01x} {:#01x}\n", ProtocolVersion::Mvd1 as u32, t.protocol.mvd_protocol_extension.bits());
                    msg_bytes = fte.into_bytes();
                    msg.append(&mut msg_bytes);
                }
                msg.extend(b"\n");
                return Ok(ClientStatus{
                    response: Some(msg),
                    packet: None});
            },
            Packet::ConnectionLessServerConnection => {
                let mut message = Message::empty();
                self.state = ClientConnectionState::ConnectionNegotiatonConnectionAccepted;
                let (out, ack) = self.channel.unreliable();
                message.write_u32(out);
                message.write_u32(ack);
                message.write_u16(self.local_port);
                message.write_client_command_string("new");

                message.write_u8(ClientServer::Nop as u8);
                self.write_empty_move_cmd(&mut message)?;
                return Ok(ClientStatus{
                    response: Some(*message.buffer.clone()),
                    packet: None});
            },
            Packet::Connected(p)=> {
                self.state = ClientConnectionState::Connected;
                self.channel.recieved(p.sequence, p.sequence_ack);
                let mut message = Message::empty();

                let (out, ack) = self.channel.unreliable();
                message.write_u32(out);
                message.write_u32(ack);
                message.write_u16(self.local_port);
                for server_message in &p.messages {
                    match server_message {
                        ServerMessage::Soundlist(soundlist) => {
                            if soundlist.offset > 0 {
                                message.write_client_command_string(format!("soundlist {} {}", self.serverdata.servercount, soundlist.offset));
                            } else {
                                message.write_client_command_string(format!("modellist {} {}", self.serverdata.servercount, 0));
                            }
                        },
                        ServerMessage::Modellist(modellist) => {
                            if modellist.offset > 0 {
                                message.write_client_command_string(format!("modellist {} {}", self.serverdata.servercount, modellist.offset));
                            } else {
                                message.write_client_command_string(format!("prespawn {} 0 {}", self.serverdata.servercount, self.map_crc));

                                message.write_client_command_string(format!("setinfo pmodel {}", 3316));
                                message.write_client_command_string(format!("setinfo emodel {}", 6967));
                                self.prespawn_send = true;
                            }
                        },
                        ServerMessage::Stufftext(stufftext) => {
                            if stufftext.text.string == "cmd pext_" {
                                let mut s:String = "pext ".to_owned();
                                if self.protocol.fte_protocol_extensions.bits() != 0 {
                                    let fte = format!("{:#01x} {:#01x} ", ProtocolVersion::Fte as u32, self.protocol.fte_protocol_extensions.bits());
                                    s.push_str(fte.as_str());
                                }

                                if self.protocol.fte_protocol_extensions_2.bits() != 0 {
                                    let fte = format!("{:#01x} {:#01x}\n", ProtocolVersion::Fte2 as u32, self.protocol.fte_protocol_extensions_2.bits());
                                    s.push_str(fte.as_str());
                                }

                                if self.protocol.mvd_protocol_extension.bits() != 0 {
                                    let fte = format!("{:#01x} {:#01x}\n", ProtocolVersion::Mvd1 as u32, self.protocol.mvd_protocol_extension.bits());
                                    s.push_str(fte.as_str());
                                }
                                message.write_client_command_string(s);
                            } else if stufftext.text.string == "cmd new_" {
                                message.write_client_command_string("new");
                            } else if stufftext.text.string.starts_with("cmd prespawn") {

                                let s = stufftext.text.string.trim_end_matches('_');
                                let splits = s.split(' ');
                                let vec: Vec<&str> = splits.collect();
                                if vec.len() != 4 {
                                    panic!("couldnt parse prespawn!");
                                }

                                message.write_client_command_string(format!("prespawn {} {}", vec[2], vec[3]));
                            } else if stufftext.text.string.starts_with("cmd spawn") {

                                let s = stufftext.text.string.trim_end_matches('_');
                                let splits = s.split(' ');
                                let vec: Vec<&str> = splits.collect();
                                if vec.len() != 4 {
                                    panic!("couldnt parse spawn!");
                                }
                                message.write_client_command_string(format!("spawn {} {}", vec[2], vec[3]));
                            } else if stufftext.text.string.starts_with("fullserverinfo") {
                            } else if stufftext.text.string.starts_with("skins") {
                                message.write_client_command_string(format!("begin {}", self.serverdata.servercount));
                            }
                        },
                        ServerMessage::Serverdata(serverdata) => {
                            self.serverdata = serverdata.clone();
                                message.write_client_command_string(format!("soundlist {} 0", serverdata.servercount));
                            self.protocol.fte_protocol_extensions = serverdata.fte_protocol_extension;
                            self.protocol.fte_protocol_extensions_2 = serverdata.fte_protocol_extension_2;
                            self.protocol.mvd_protocol_extension = serverdata.mvd_protocol_extension;
                        },
                        _ => {},
                    }
                }
                self.write_empty_move_cmd(&mut message)?;
                return Ok(ClientStatus{
                    response: Some(*message.buffer.clone()),
                    packet: Some(Packet::Connected(p))});
            },
            _ => {},
        }
        Err(Box::new(ClientError::UnhandledPacket))
    }

    fn get_challenge(&mut self) -> Vec<u8> {
        self.state = ClientConnectionState::ConnectionNegotiatonChallengeSend;
        let mut m: Vec<u8> = Vec::from([255, 255, 255, 255]);
        m.extend(b"getchallenge\n".to_vec());
        m
    }
}
