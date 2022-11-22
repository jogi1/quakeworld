use crate::protocol::message::errors::MessageError;
use paste::paste;
use serde::Serialize;

#[cfg(feature = "ascii_strings")]
use crate::utils::ascii_converter::AsciiConverter;

use crate::protocol::types::*;
use crate::protocol::message::trace::{trace_start, trace_stop, trace_annotate, MessageTrace, TraceValue, ToTraceValue};

pub mod errors;
pub mod trace;

#[derive(Serialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MessageType {
    #[default] None,
    Connection,
    Mvd,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Serialize)]
pub struct MessageFlags {
    pub protocol: u32,
    pub fte_protocol_extensions: FteProtocolExtensions,
    pub fte_protocol_extensions_2: FteProtocolExtensions2,
    pub mvd_protocol_extension: MvdProtocolExtensions,
}


impl Default for MessageFlags{
    fn default() -> Self {
        MessageFlags {
            protocol: 0,
            fte_protocol_extensions: FteProtocolExtensions::empty(),
            fte_protocol_extensions_2: FteProtocolExtensions2::empty(),
            mvd_protocol_extension: MvdProtocolExtensions::empty(),
        }
    }
}

impl MessageFlags {
    pub fn new_empty() -> MessageFlags {
        MessageFlags{
            protocol: 0,
            fte_protocol_extensions: FteProtocolExtensions::empty(),
            fte_protocol_extensions_2: FteProtocolExtensions2::empty(),
            mvd_protocol_extension: MvdProtocolExtensions::empty()
        }
    }
}

#[derive(Serialize, Clone, Default)]
pub struct Message
{
    pub start: usize, // starting position in the buffer
    pub length: usize, // length of the message
    pub position: usize, // current read position after start
    pub buffer: Box<Vec<u8>>,
    pub bigendian: bool,
    pub flags: MessageFlags,
    #[cfg(feature = "ascii_strings")]
    pub ascii_converter: AsciiConverter,
    #[cfg(feature = "trace")]
    pub trace: MessageTrace,
    pub r#type: MessageType
}

macro_rules! endian_read {
    ($($ty:ty), *) => {
        $(
            paste! {
                pub fn [< read_$ty >] (&mut self, readahead: bool) ->  Result<$ty, MessageError> {
                    const TYPE_SIZE:usize = std::mem::size_of::<$ty>();
                    trace_start!(self, readahead);
                    self.check_read_size(TYPE_SIZE)?;
                    let mut a: [u8; TYPE_SIZE] = [0; TYPE_SIZE];
                    for n in 0..TYPE_SIZE {
                        a[n] = self.buffer[self.start + self.position + n ];
                    }
                    self.read_ahead(readahead, TYPE_SIZE);

                    let v;
                    if self.bigendian {
                        v = $ty::from_be_bytes(a);
                    } else {
                        v = $ty::from_le_bytes(a);
                    }
                    trace_stop!(self, v, $ty);
                    Ok(v)
                }
            }
         )*
    }
}

// maybe beter
macro_rules! endian_write{
    ($($ty:ty), *) => {
        $(
            paste! {
                pub fn [< write_$ty >] (&mut self, v: impl Into<$ty>) ->  usize {
                    let v = v.into();
                    const TYPE_SIZE:usize = std::mem::size_of::<$ty>();
                    let a: [u8; TYPE_SIZE];
                    if self.bigendian {
                        a = v.to_be_bytes();
                    } else {
                        a = v.to_le_bytes();
                    }
                    self.buffer.extend_from_slice(&a);
                    self.position += TYPE_SIZE;
                    TYPE_SIZE
                }
            }
         )*
    }
}

macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3].to_string()
    }}
}

impl From<Message> for Vec<u8> {
    fn from(m: Message) -> Vec<u8> {
        *m.buffer
    }
}

impl From<&Message> for Vec<u8> {
    fn from(m: &Message) -> Vec<u8> {
        *m.buffer.clone()
    }
}

impl From<&mut Message> for Vec<u8> {
    fn from(m: &mut Message) -> Vec<u8> {
        *m.buffer.clone()
    }
}

impl Message {
    endian_read!(u8, u16, u32, i8, i16, i32, f32);
    endian_write!(u8, u16, u32, i8, i16, i32, f32);

    pub fn read_bytes (&mut self, count: u32, readahead: bool) -> Result<Vec<u8>, MessageError> {
        trace_start!(self, readahead);
        let mut buf = Vec::new();
        for i in 0..count {
            buf.push(self.buffer[self.start + self.position + (i as usize) ]);
        }
        if !readahead {
            self.position += count as usize;
        }

        trace_stop!(self, buf.clone());
        Ok(buf)
    }

    pub fn write_stringbyte (&mut self, string: String ) ->  usize {
        let mut size: usize = 0;
        let s = string.as_bytes();
        for c in s {
            self.buffer.push(*c);
            size += 1;
        }
        self.buffer.push(0_u8);
        size += 1;
        size
    }

    pub fn write_client_command_string (&mut self, string: impl Into<String>) ->  usize {
        let string = string.into();
        let mut size: usize = 1;
        self.write_u8(ClientServer::StringCommand as u8);
        let s = string.as_bytes();
        for c in s {
            self.write_u8(*c);
            size += 1;
        }
        self.write_u8(0);
        size += 1;
        size
    }

    pub fn write_client_command_string_vec (&mut self, string: Vec<u8>) ->  usize {
        let mut s: usize = 1;
        self.write_u8(ClientServer::StringCommand as u8);
        s += string.len();
        self.buffer.extend(string);
        self.buffer.push(0_u8);
        s
    }

    pub fn write_angle16(&mut self, angle: f32) -> usize {
        self.write_u16((angle *65536.0 / 360.0) as u16)
    }

    pub fn write_delta_usercommand(&mut self, delta_usercommand: DeltaUserCommand) ->  usize {
        let mut s: usize = 0;
        let mut bits = UserCommandFlags::from_bits_truncate(0);

        let position = self.position;
        s += self.write_u8(0);

        if delta_usercommand.angle.x.is_some() {
            bits |= UserCommandFlags::ANGLE1;
            s += self.write_angle16(delta_usercommand.angle.x.unwrap());
        }
        if delta_usercommand.angle.y.is_some() {
            bits |= UserCommandFlags::ANGLE2;
            s += self.write_angle16(delta_usercommand.angle.y.unwrap());
        }
        if delta_usercommand.angle.z.is_some() {
            bits |= UserCommandFlags::ANGLE3;
            s += self.write_angle16(delta_usercommand.angle.z.unwrap());
        }

        if delta_usercommand.forward.is_some() {
            bits |= UserCommandFlags::FORWARD;
            s += self.write_i16(delta_usercommand.forward.unwrap());
        }
        if delta_usercommand.side.is_some() {
            bits |= UserCommandFlags::SIDE;
            s += self.write_i16(delta_usercommand.side.unwrap());
        }
        if delta_usercommand.up.is_some() {
            bits |= UserCommandFlags::UP;
            s += self.write_i16(delta_usercommand.up.unwrap());
        }

        if delta_usercommand.buttons.is_some() {
            bits |= UserCommandFlags::BUTTONS;
            s += self.write_u8(delta_usercommand.buttons.unwrap());
        }

        if delta_usercommand.impulse.is_some() {
            bits |= UserCommandFlags::IMPULSE;
            s += self.write_u8(delta_usercommand.impulse.unwrap());
        }

        if delta_usercommand.msec.is_some() {
            s += self.write_u8(delta_usercommand.msec.unwrap());
        }
        self.buffer[position] = bits.bits();
        s
    }

#[cfg(not(feature = "ascii_strings"))]
    pub fn read_stringbyte (&mut self, readahead: bool) ->  Result<StringByte, MessageError> {
        trace_start!(self, readahead);
        let mut buf =  Vec::new();
        let original_position = self.position;
        buf.clear();
        loop {
            let b = self.read_u8(false)?;
            if b == 255 {
                continue;
            } else if b == 0 {
                break
            }

            buf.push(b);
        }

        if readahead {
            self.position = original_position;
        }
        trace_stop!(self, buf.clone());
        Ok(StringByte{ bytes: buf})
    }


#[cfg(feature = "ascii_strings")]
    pub fn to_ascii(&mut self, buffer: impl Into<Vec<u8>>) -> String {
        let buffer = buffer.into();
        self.ascii_converter.convert(buffer)
    }

#[cfg(feature = "ascii_strings")]
    pub fn read_stringbyte (&mut self, readahead: bool) ->  Result<StringByte, MessageError> {
        trace_start!(self, readahead);
        let mut buf : Vec<u8> =  Vec::new();
        let original_position = self.position;
        buf.clear();
        loop {
            let b = self.read_u8(false)?;
            if b == 255 {
                continue;
            } else if b == 0 {
                break
            }

            buf.push(b);
        }

        if readahead {
            self.position = original_position;
        }

        let string = self.to_ascii(buf.clone());
        let v = StringByte{
            bytes: buf,
            string,
        };

        trace_stop!(self, v);
        Ok(v)
    }

    pub fn read_stringvector (&mut self, readahead: bool) ->  Result<Vec<StringByte>, MessageError> {
        let mut strings =  Vec::new();
        strings.clear();
        loop {
            let s = self.read_stringbyte(readahead)?;
            if s.bytes.is_empty() {
                break;
            }
            strings.push(s);
        }

        Ok(strings)
    }

    pub fn read_coordinate(&mut self, readahead: bool) ->  Result<Coordinate, MessageError> {
        if self.flags.fte_protocol_extensions.contains(FteProtocolExtensions::FLOATCOORDS) {
            let f = self.read_f32(readahead)?;
            return Ok(f);
        }
        let s = self.read_i16(readahead)? as f32;
        Ok(s * (1.0/8.0))
    }

    pub fn read_coordinatevector(&mut self, readahead: bool) ->  Result<CoordinateVector, MessageError> {
        let x = self.read_coordinate(readahead)?;
        let y = self.read_coordinate(readahead)?;
        let z = self.read_coordinate(readahead)?;
        Ok(CoordinateVector{x, y, z})
    }

    pub fn read_angle(&mut self, readahead: bool) ->  Result<Angle, MessageError> {
        if self.flags.fte_protocol_extensions.contains(FteProtocolExtensions::FLOATCOORDS) {
            let f = self.read_angle16(readahead)?;
            return Ok(f);
        }
        let s = self.read_u8(readahead)? as f32;
        Ok(s * (360.0/256.0))
    }

    pub fn read_angle16(&mut self, readahead: bool) ->  Result<Angle, MessageError> {
        let s = self.read_u16(readahead)? as f32;
        Ok(s * (360.0/65535.0))
    }

    pub fn read_anglevector(&mut self, readahead: bool) ->  Result<AngleVector, MessageError> {
        let x = self.read_angle(readahead)?;
        let y = self.read_angle(readahead)?;
        let z = self.read_angle(readahead)?;
        Ok(AngleVector{x, y, z})
    }

    pub fn empty() -> Message {
        Message {
            start: 0,
            length: 0,
            position: 0,
            buffer: Box::new(vec![]),
            bigendian: false,
            flags: MessageFlags::new_empty(),
            #[cfg(feature = "ascii_strings")]
            ascii_converter: AsciiConverter::new(),
            r#type: MessageType::Connection,
            ..Default::default()
        }
    }

    pub fn empty_be() -> Message {
        Message {
            bigendian: true,
            r#type: MessageType::Connection,
            ..Default::default()
        }
    }

    #[cfg(feature = "ascii_strings")]
    pub fn new (buffer: Box<Vec<u8>>, start: usize, length: usize, bigendian: bool, flags: MessageFlags, maybe_ascii_converter: Option<AsciiConverter>, r#type: MessageType) -> Message {
        let ascii_converter: AsciiConverter;
        if let Some(ascii_convter_in) = maybe_ascii_converter {
            ascii_converter = ascii_convter_in;
        } else {
            ascii_converter= AsciiConverter::new();
        }

        Message {
            start,
            length,
            position: 0,
            buffer,
            bigendian,
            flags,
            ascii_converter,
            r#type,
            ..Default::default()
        }
    }

    #[cfg(not(feature = "ascii_strings"))]
    pub fn new (buffer: Box<Vec<u8>>, start: usize, length: usize, bigendian: bool, flags: MessageFlags, r#type: MessageType) -> Message {

        Message {
            start,
            length,
            position: 0,
            buffer,
            bigendian,
            flags,
            r#type,
            ..Default::default()
        }
    }

    pub fn get_range(&mut self, start: usize, length: usize) -> Vec<u8> {
        let mut buf = Vec::new();
        for i in self.start + start .. self.start + start + length {
            buf.push(self.buffer[i]);
        }
        buf
    }

    #[inline]
    fn read_ahead(&mut self, readahead: bool, length: usize) {
        if !readahead {
            self.position += length;
        }
    }

    #[inline]
    fn check_read_size(&mut self, length: usize) -> Result<(), MessageError>{
        if self.position + length > self.length {
            return Err(MessageError::ReadBeyondSize(self.length, self.position, length));
        }
        Ok(())
    }

    pub fn is_oob(&mut self) -> Result<bool, MessageError> {
        let i = self.read_i32(true)?;
        if i == -1 {
            return Ok(true);
        }
         Ok(false)
    }

    pub fn read_packet(&mut self) -> Result<Packet, MessageError> {
        let is_oob = self.is_oob()?;
        if is_oob {
            return self.read_oob_packet();
        }
        self.read_connected_packet()
    }

    pub fn read_connected_packet(&mut self) -> Result<Packet, MessageError> {
        #[cfg(feature = "trace")]
        self.read_trace_start(function!(), false);

        #[cfg(feature = "trace")]
        self.read_trace_annotate("sequence");
        let sequence = self.read_u32(false)?;
        #[cfg(feature = "trace")]
        self.read_trace_annotate("sequence_ack");
        let sequence_ack = self.read_u32(false)?;
        let mut messages = Vec::new();

        loop {

            #[cfg(feature = "trace")]
            self.read_trace_annotate("message type");
            let t = match self.read_u8(false) {
                Ok(t) => t,
                Err(e) => {
                    match e {
                        MessageError::ReadBeyondSize(_,_,_) => break,
                        _ => return Err(e),
                    }
                },
            };

            let cmd = match ServerClient::try_from(t) {
                Ok(cmd) => cmd,
                Err(_) => {

                    #[cfg(feature = "trace")]
                    {
                        let p = Packet::Connected(Connected{
                            sequence,
                            sequence_ack,
                            messages,
                        });
                        self.read_trace_stop(TraceValue::Packet(p));
                    }
                    return Err(MessageError::UnknownType(t));
                }
            };
            let ret = cmd.read_message(self)?;
            messages.push(ret);
        }
        let p = Packet::Connected(Connected{
            sequence,
            sequence_ack,
            messages,
        });

        #[cfg(feature = "trace")]
        self.read_trace_stop(TraceValue::Packet(p.clone()));
        Ok(p)
    }

    pub fn read_oob_packet(&mut self) -> Result<Packet, MessageError> {
        let _ = self.read_i32(false)?;
        let _packet_type = self.read_u8(false)?;
        let packet_type = CommandCode::try_from(_packet_type)?;
        match packet_type {
            CommandCode::S2cChallenge => {
                self.read_packet_s2c_challenge()
            }
            CommandCode::S2cConnection => {
                Ok(Packet::ConnectionLessServerConnection)
            },
        }
    }

    fn read_packet_s2c_challenge(&mut self) -> Result<Packet, MessageError> {
        let mut flags = MessageFlags::new_empty();
        let s = self.read_stringbyte(false)?;

        #[cfg(not(feature = "ascii_strings"))]
        let ss = match std::str::from_utf8(&s.bytes) {
            Ok(v) => v,
            Err(_) => return Err(MessageError::StringError(format!("could not parse challlenge: {}", s))),
        };

        #[cfg(feature = "ascii_strings")]
        let ss = s.string;

        let challenge = ss.parse::<i32>().unwrap();
        while let Ok(prot_r) = self.read_u32(false) {
        /*
        loop {
            let prot_r = match self.read_u32(false) {
                Ok(v) => v,
                Err(_) => {break;}
            };
            */
            let prot = ProtocolVersion::try_from(prot_r)?;
            match prot {
                ProtocolVersion::Fte => {
                    let i = self.read_u32(false)?;
                    flags.fte_protocol_extensions = FteProtocolExtensions::from_bits_truncate(i);
                }
                ProtocolVersion::Fte2 => {
                    let i = self.read_u32(false)?;
                    flags.fte_protocol_extensions_2 = FteProtocolExtensions2::from_bits_truncate(i);
                }
                ProtocolVersion::Mvd1 => {
                    let i = self.read_u32(false)?;
                    flags.mvd_protocol_extension = MvdProtocolExtensions::from_bits_truncate(i);
                }
                _ => {
                    flags.protocol = self.read_u32(false)?;
                }
            }
        };
        Ok(Packet::ConnectionLessServerChallenge(ConnectionLessServerChallenge{
            protocol: flags,
            challenge,
        }))
    }

    pub fn replace_at_position(&mut self, bytes: impl Into<Vec<u8>>, position: usize) -> Result<(), MessageError> {
        let bytes = bytes.into();
        for (i, c) in bytes.iter().enumerate() {
            self.buffer[position+i] = *c;
        }
        Ok(())
    }
}

