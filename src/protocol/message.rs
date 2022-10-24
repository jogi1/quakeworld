use thiserror::Error;
use paste::paste;
use serde::Serialize;
#[cfg(feature = "ascii_strings")]
use crate::utils::ascii_converter::AsciiConverter;

use crate::protocol::types::*;


#[derive(Error, Debug, Serialize)]
pub enum MessageError {
    #[error("attempting to read beyond demo size({0}) with position({1}) and size({2})")]
    ReadBeyondSize(usize, usize, usize),
    #[error("reading unhandled type: {0}")]
    UnhandledType(ServerClient),
    #[error("{0}")]
    StringError(String)
}

impl From<String> for MessageError {
    fn from(err: String) -> MessageError{
        return MessageError::StringError(err);
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Serialize)]
pub struct MessageFlags {
    pub fte_protocol_extensions: FteProtocolExtensions,
    pub fte_protocol_extensions_2: FteProtocolExtensions,
    pub mvd_protocol_extension: MvdProtocolExtensions,
}

impl MessageFlags {
    pub fn new_empty() -> MessageFlags {
        return MessageFlags{
            fte_protocol_extensions: FteProtocolExtensions::empty(),
            fte_protocol_extensions_2: FteProtocolExtensions::empty(),
            mvd_protocol_extension: MvdProtocolExtensions::empty()
        }
    }
}

#[derive(Serialize)]
pub struct ReadTrace {
    pub start: usize,
    pub length: usize,
    pub readahead: bool,
    pub function: String,
}

#[derive(Serialize)]
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
    pub read_tracing: bool,
    pub read_traces: Vec<ReadTrace>
}


// maybe beter
macro_rules! endian_read {
    ($($ty:ty), *) => {
        $(
            paste! {
                pub fn [< read_$ty >] (&mut self, readahead: bool) ->  Result<$ty, MessageError> {
                    const TYPE_SIZE:usize = std::mem::size_of::<$ty>();
                    if self.read_tracing {
                        self.read_traces.push(ReadTrace{
                            start: self.start +self.position,
                            length: TYPE_SIZE,
                            readahead,
                            function: stringify!([< read_$ty >]).to_string(),
                        })
                    }
                    self.check_read_size(TYPE_SIZE)?;
                    let mut a: [u8; TYPE_SIZE] = [0; TYPE_SIZE];
                    for n in 0..TYPE_SIZE {
                        a[n] = self.buffer[self.start + self.position + n ];
                    }
                    self.read_ahead(readahead, TYPE_SIZE);

                    if self.bigendian {
                        return Ok($ty::from_be_bytes(a));
                    } else {
                        return Ok($ty::from_le_bytes(a));
                    }
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
                pub fn [< write_$ty >] (&mut self, v: $ty) ->  usize {
                    const TYPE_SIZE:usize = std::mem::size_of::<$ty>();
                    let a: [u8; TYPE_SIZE];
                    if self.bigendian {
                        a = v.to_be_bytes();
                    } else {
                        a = v.to_le_bytes();
                    }
                    self.buffer.extend_from_slice(&a);
                    return TYPE_SIZE
                }
            }
         )*
    }
}

impl Message {
    endian_read!(u8, u16, u32, i8, i16, i32, f32);
    endian_write!(u8, u16, u32, i8, i16, i32, f32);

    pub fn read_bytes (&mut self, count: u32, readahead: bool) -> Result<Vec<u8>, MessageError> {

        if self.read_tracing {
            self.read_traces.push(ReadTrace{
                start: self.start +self.position,
                length: count as usize,
                readahead,
                function: "read_bytes".to_string(),
            })
        }
        let mut buf = Vec::new();
        for i in 0..count {
            buf.push(self.buffer[self.start + self.position + (i as usize) ]);
        }
        if !readahead {
            self.position += count as usize;
        }
        return Ok(buf);
    }


    pub fn write_stringbyte (&mut self, string: String ) ->  usize {
        let mut size: usize = 0;
        let s = string.as_bytes();
        for i in 0..s.len() {
            self.buffer.push(s[i]);
            size += 1;
        }
        self.buffer.push(0 as u8);
        size += 1;
        return size;
    }

#[cfg(not(feature = "ascii_strings"))]
    pub fn read_stringbyte (&mut self, readahead: bool) ->  Result<StringByte, MessageError> {
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

        if self.read_tracing {
            self.read_traces.push(ReadTrace{
                start: self.start + original_position,
                length: self.position - original_position as usize,
                readahead,
                function: "read_stringbyte".to_string(),
            })
        }
        if readahead {
            self.position = original_position;
        }

        return Ok(buf);
    }


#[cfg(feature = "ascii_strings")]
    pub fn to_ascii(&mut self, buffer: &Vec<u8>) -> String {
        return self.ascii_converter.convert(&buffer);
    }

#[cfg(feature = "ascii_strings")]
    pub fn read_stringbyte (&mut self, readahead: bool) ->  Result<StringByte, MessageError> {
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

        if self.read_tracing {
            self.read_traces.push(ReadTrace{
                start: self.start + original_position,
                length: self.position - original_position as usize,
                readahead,
                function: "read_stringbyte".to_string(),
            })
        }
        if readahead {
            self.position = original_position;
        }

        let string = self.to_ascii(&buf);
        return Ok(StringByte{
            bytes: buf,
            string,
        });
    }

#[cfg(feature = "ascii_strings")]
    pub fn read_stringvector (&mut self, readahead: bool) ->  Result<Vec<StringByte>, MessageError> {
        let mut strings =  Vec::new();
        strings.clear();
        loop {
            let s = self.read_stringbyte(readahead)?;
            if s.bytes.len() == 0 {
                break;
            }
            strings.push(s);
        }

        return Ok(strings);
    }

#[cfg(not(feature = "ascii_strings"))]
    pub fn read_stringvector (&mut self, readahead: bool) ->  Result<Vec<Vec<u8>>, MessageError> {
        let mut strings =  Vec::new();
        strings.clear();
        loop {
            let s = self.read_stringbyte(readahead)?;
            if s.len() == 0 {
                break;
            }
            strings.push(s);
        }

        return Ok(strings);
    }

    pub fn read_coordinate(&mut self, readahead: bool) ->  Result<Coordinate, MessageError> {
        if self.flags.fte_protocol_extensions.contains(FteProtocolExtensions::FLOATCOORDS) {
            let f = self.read_f32(readahead)?;
            return Ok(f);
        }
        let s = self.read_i16(readahead)? as f32;
        return Ok(s * (1.0/8.0));
    }

    pub fn read_coordinatevector(&mut self, readahead: bool) ->  Result<CoordinateVector, MessageError> {
        let x = self.read_coordinate(readahead)?;
        let y = self.read_coordinate(readahead)?;
        let z = self.read_coordinate(readahead)?;
        return Ok(CoordinateVector{x, y, z});
    }

    pub fn read_angle(&mut self, readahead: bool) ->  Result<Angle, MessageError> {
        if self.flags.fte_protocol_extensions.contains(FteProtocolExtensions::FLOATCOORDS) {
            let f = self.read_angle16(readahead)?;
            return Ok(f);
        }
        let s = self.read_u8(readahead)? as f32;
        return Ok(s * (360.0/256.0));
    }

    pub fn read_angle16(&mut self, readahead: bool) ->  Result<Angle, MessageError> {
        let s = self.read_u16(readahead)? as f32;
        return Ok(s * (360.0/65535.0));
    }

    pub fn read_anglevector(&mut self, readahead: bool) ->  Result<AngleVector, MessageError> {
        let x = self.read_angle(readahead)?;
        let y = self.read_angle(readahead)?;
        let z = self.read_angle(readahead)?;
        return Ok(AngleVector{x, y, z});
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
            read_tracing: false,
            read_traces: vec![],
        }
    }

    #[cfg(feature = "ascii_strings")]
    pub fn new (buffer: Box<Vec<u8>>, start: usize, length: usize, bigendian: bool, flags: MessageFlags, maybe_ascii_converter: Option<AsciiConverter>) -> Message {
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
            read_tracing: false,
            read_traces: vec![],
        }
    }

    #[cfg(not(feature = "ascii_strings"))]
    pub fn new (buffer: Box<Vec<u8>>, start: usize, length: usize, bigendian: bool, flags: MessageFlags) -> Message {

        Message {
            start,
            length,
            position: 0,
            buffer,
            bigendian,
            flags,
            read_tracing: false,
            read_traces: vec![],
        }
    }

    pub fn get_range(&mut self, start: usize, length: usize) -> Vec<u8> {
        let mut buf = Vec::new();
        for i in self.start + start .. self.start + start + length {
            buf.push(self.buffer[i]);
        }
        return buf;
    }

    #[inline]
    fn read_ahead(&mut self, readahead: bool, length: usize) {
        if readahead == false {
            self.position += length;
        }
    }

    #[inline]
    fn check_read_size(&mut self, length: usize) -> Result<(), MessageError>{
        if self.position + length > self.length {
            return Err(MessageError::ReadBeyondSize(self.length, self.position, length));
        }
        return Ok(())
    }

    pub fn is_oob(&mut self) -> Result<bool, MessageError> {
        let i = self.read_i32(true)?;
        if i == -1 {
            return Ok(true);
        }
        return Ok(false);
    }

    pub fn read_packet(&mut self) -> Result<Packet, MessageError> {
        let is_oob = self.is_oob()?;
        if is_oob {
            return self.read_oob_packet();
        } else {
            return self.read_connected_packet();
        }
    }

    pub fn read_connected_packet(&mut self) -> Result<Packet, MessageError> {
        let outgoing_sequence = self.read_u32(false)?;
        let incoming_sequence = self.read_u32(false)?;
        let mut messages = Vec::new();

        loop {
            let t = match self.read_u8(false) {
                Ok(t) => t,
                Err(_) => { println!("we broke here!");break},
            };

            let cmd = match ServerClient::try_from(t) {
                Ok(cmd) => cmd,
                Err(_) => return Err(MessageError::StringError(format!("unhandled command: {}", t)))
            };
            let ret = cmd.read_message(self)?;
            messages.push(ret);
        }

        return Ok(Packet::Connected(Connected{
            outgoing_sequence,
            incoming_sequence,
            messages,
        }))

    }

    pub fn read_oob_packet(&mut self) -> Result<Packet, MessageError> {
        let _ = self.read_i32(false)?;
        let _packet_type = self.read_u8(false)?;
        let packet_type = CommandCode::try_from(_packet_type)?;
        match packet_type {
            CommandCode::S2cChallenge => {
                return self.read_packet_s2c_challenge();
            }
            CommandCode::S2cConnection => {
                return Ok(Packet::ConnectionLessServerConnection);
            },
        }
    }

    fn read_packet_s2c_challenge(&mut self) -> Result<Packet, MessageError> {
        let mut flags = MessageFlags::new_empty();
        let s = self.read_stringbyte(false)?;

        #[cfg(not(feature = "ascii_strings"))]
        let ss = match std::str::from_utf8(&s) {
            Ok(v) => v,
            Err(_) => return Err(MessageError::StringError("could not parse challlenge".to_string())),
        };

        #[cfg(feature = "ascii_strings")]
        let ss = s.string;

        let challenge = ss.parse::<i32>().unwrap();
        loop {
            let prot_r = match self.read_u32(false) {
                Ok(v) => v,
                Err(_) => {break;}
            };
            let prot = ProtocolVersion::try_from(prot_r)?;
            match prot {
                ProtocolVersion::Fte => {
                    let i = self.read_u32(false)?;
                    flags.fte_protocol_extensions = FteProtocolExtensions::from_bits_truncate(i);
                }
                ProtocolVersion::Fte2 => {
                    let i = self.read_u32(false)?;
                    flags.fte_protocol_extensions_2 = FteProtocolExtensions::from_bits_truncate(i);
                }
                ProtocolVersion::Mvd1 => {
                    let i = self.read_u32(false)?;
                    flags.mvd_protocol_extension = MvdProtocolExtensions::from_bits_truncate(i);
                }
                _ => {
                    let _ = self.read_u32(false)?;
                }
            }
        };
        return Ok(Packet::ConnectionLessServerChallenge(ConnectionLessServerChallenge{
            protocol: flags,
            challenge,
        }));
    }
}
