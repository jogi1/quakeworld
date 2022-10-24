use num_enum::TryFromPrimitive;
use strum_macros::Display;
use paste::paste;
use bitflags::bitflags;
use serde::Serialize;

use crate::protocol::message::*;

use protocol_macros::ParseMessage;

#[cfg(not(feature = "ascii_strings"))]
pub type StringByte = Vec<u8>;

#[cfg(feature = "ascii_strings")]
#[derive(PartialOrd, PartialEq, Clone, Debug, Serialize)]
pub struct StringByte {
    pub bytes: Vec<u8>,
    pub string: String,
}

pub type StringVector = Vec<StringByte>;

pub type Coordinate = f32;
#[derive(Debug, PartialEq, PartialOrd, Default, Serialize)]
pub struct CoordinateVector { 
    pub x: Coordinate,
    pub y: Coordinate,
    pub z: Coordinate,
}


#[derive(Debug, PartialEq, PartialOrd, Copy, Clone, Serialize)]
pub struct CoordinateVectorOption { 
    pub x: Option<Coordinate>,
    pub y: Option<Coordinate>,
    pub z: Option<Coordinate>,
}

impl CoordinateVectorOption {
    fn empty() -> CoordinateVectorOption {
        return CoordinateVectorOption{
            x: None,
            y: None,
            z:None}
    }

    fn is_empty(self) -> bool {
        if self.x == None &&
            self.y == None &&
                self.z == None {
                    return true
                }
        return false
    }
}

pub type Angle = f32;
#[derive(Debug, PartialEq, PartialOrd, Serialize)]
pub struct AngleVector { 
    pub x: Angle,
    pub y: Angle,
    pub z: Angle,
}


#[derive(Debug, PartialEq, PartialOrd, Copy, Clone, Serialize)]
pub struct AngleVectorOption { 
    pub x: Option<Angle>,
    pub y: Option<Angle>,
    pub z: Option<Angle>,
}

impl AngleVectorOption {
    fn empty() -> AngleVectorOption {
        return AngleVectorOption{
            x: None,
            y: None,
            z:None}
    }

    fn is_empty(self) -> bool {
        if self.x == None &&
            self.y == None &&
                self.z == None {
                    return true
                }
        return false
    }
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, TryFromPrimitive, Display, Copy, Clone, Serialize)]
#[repr(u8)]
pub enum DemoCommand {
    Command = 0,
	Read,
	Set,
	Multiple,
	Single,
	Stats,
	All,
    Empty
}


#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, TryFromPrimitive, Display, Clone, Serialize)]
#[repr(u32)]
pub enum ProtocolVersion {
	Standard = 28,
    Fte = u32::from_ne_bytes(*b"FTEX"),
    Fte2 = u32::from_ne_bytes(*b"FTE2"),
    Mvd1 = u32::from_ne_bytes(*b"MVD1"),
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, TryFromPrimitive, Display, Serialize)]
#[repr(u32)]
pub enum ProtocolExtensionFte {
	Trans             = 0x00000008, // .alpha support
	AccurateTimings   = 0x00000040,
	HlBsp             = 0x00000200, //stops fte servers from complaining
	Modeldbl          = 0x00001000,
	Entitydbl         = 0x00002000, //max =of 1024 ents instead of 512
	Entitydbl2        = 0x00004000, //max =of 1024 ents instead of 512
	Floatcoords       = 0x00008000, //supports =floating point origins.
	Spawnstatic2      = 0x00400000, //Sends =an entity delta instead of a baseline.
	Packetentities256 = 0x01000000, //Client =can recieve 256 packet entities.
	ChunkedDownloads  = 0x20000000 //alternate =file download method. Hopefully it'll give quadroupled download speed, especially on higher pings.
}


#[derive(Debug, PartialEq, PartialOrd, Clone, Serialize)]
pub struct Serverdata {
    pub protocol: ProtocolVersion,
    pub fte_protocol_extension: FteProtocolExtensions,
    pub fte_protocol_extension_2: FteProtocolExtensions,
    pub mvd_protocol_extension: MvdProtocolExtensions,
    pub servercount: u32,
    pub gamedir: StringByte,
    pub demotime: f32,
    pub map: StringByte,
    pub movevars: [f32;10]
}

impl Serverdata {
    pub fn read(message: &mut Message) -> Result<ServerMessage, MessageError> {
        let mut protocol: ProtocolVersion;
        let mut fte_protocol_extension = FteProtocolExtensions::empty();
        let mut fte_protocol_extension_2 = FteProtocolExtensions::empty();
        let mut mvd_protocol_extension = MvdProtocolExtensions::empty();
        loop {
            let p = message.read_u32(false)?;
            protocol = ProtocolVersion::try_from(p)?;

            match protocol {
                ProtocolVersion::Standard => break,
                ProtocolVersion::Mvd1 => {
                    let u = message.read_u32(false)?;
                    mvd_protocol_extension = MvdProtocolExtensions::from_bits_truncate(u);
                    continue;
                },
                ProtocolVersion::Fte  => {
                    let u = message.read_u32(false)?;
                    fte_protocol_extension = FteProtocolExtensions::from_bits_truncate(u);
                    continue;
                },
                ProtocolVersion::Fte2  => {
                    let u = message.read_u32(false)?;
                    fte_protocol_extension_2 = FteProtocolExtensions::from_bits_truncate(u);
                    continue;
                },
            }
        }
        let servercount = message.read_u32(false)?;
        let gamedir = message.read_stringbyte(false)?;
        let demotime = message.read_f32(false)?;
        let map = message.read_stringbyte(false)?;
        let mut movevars:  [f32;10] = [0.0; 10];
        for i in 0..movevars.len() {
            movevars[i] = message.read_f32(false)?;
        }

        return Ok(
            ServerMessage::Serverdata(
                Serverdata {
                    protocol,
                    fte_protocol_extension,
                    fte_protocol_extension_2,
                    mvd_protocol_extension,
                    servercount,
                    gamedir,
                    demotime,
                    map,
                    movevars
                })
            );
    }
}

#[derive(Debug, PartialEq, PartialOrd, ParseMessage, Serialize)]
pub struct Soundlist {
    pub start: u8,
    pub sounds: StringVector,
    pub offset: u8
}

#[derive(Debug, PartialEq, PartialOrd, ParseMessage, Serialize)]
pub struct Modellist {
    pub start: u8,
    pub models: StringVector,
    pub offset: u8
}

#[derive(Debug, PartialEq, PartialOrd,ParseMessage, Serialize)]
pub struct Cdtrack {
    pub track: u8
}

#[derive(Debug, PartialEq, PartialOrd, ParseMessage, Serialize)]
pub struct Stufftext {
    pub text: StringByte 
}

#[derive(Debug, PartialEq, PartialOrd, ParseMessage, Serialize)]
pub struct Spawnstatic {
    pub model_index: u8,
    pub model_frame: u8,
    pub colormap: u8,
    pub skinnum: u8,
    pub origin: CoordinateVector,
    pub angle: AngleVector
}

#[derive(Debug, PartialEq, PartialOrd, ParseMessage, Serialize)]
pub struct Spawnbaseline {
    pub index: u16,
    pub model_index: u8,
    pub model_frame: u8,
    pub colormap: u8,
    pub skinnum: u8,
    pub origin: CoordinateVector,
    pub angle: AngleVector
}

#[derive(Debug, PartialEq, PartialOrd, ParseMessage, Serialize)]
pub struct Spawnstaticsound{
    pub origin: CoordinateVector,
    pub index: u8,
    pub volume: u8,
    pub attenuation: u8
}

#[derive(Debug, PartialEq, PartialOrd, ParseMessage, Serialize)]
pub struct Updatefrags {
    pub player_number: u8,
    pub frags: u16
}

#[derive(Debug, PartialEq, PartialOrd, ParseMessage, Serialize)]
pub struct Updateping {
    pub player_number: u8,
    pub frags: u16
}

#[derive(Debug, PartialEq, PartialOrd, ParseMessage, Serialize)]
pub struct Updatepl {
    pub player_number: u8,
    pub pl: u8
}

#[derive(Debug, PartialEq, PartialOrd, ParseMessage, Serialize)]
pub struct Updateentertime {
    pub player_number: u8,
    pub entertime: f32
}

#[derive(Debug, PartialEq, PartialOrd, ParseMessage, Serialize)]
pub struct Updateuserinfo {
    pub player_number: u8,
    pub uid: u32,
    pub userinfo: StringByte
}

#[derive(Debug, PartialEq, PartialOrd, ParseMessage, Serialize)]
pub struct Updatestatlong{
    pub stat: u8,
    pub value: u32
}

#[derive(Debug, PartialEq, PartialOrd, ParseMessage, Serialize)]
pub struct Updatestat{
    pub stat: u8,
    pub value: u8
}

#[derive(Debug, PartialEq, PartialOrd, ParseMessage, Serialize)]
pub struct Lightstyle {
    pub index: u8,
    pub style: StringByte 
}

#[derive(Debug, PartialEq, PartialOrd, ParseMessage, Serialize)]
pub struct Serverinfo {
    pub key: StringByte,
    pub value: StringByte 
}

#[derive(Debug, PartialEq, PartialOrd, ParseMessage, Serialize)]
pub struct Centerprint {
    pub message: StringByte 
}


bitflags! {
#[derive(Serialize)]
    pub struct DfTypes: u16 {
        const ORIGIN = 1;
        const ORIGIN2 = 1 << 1;
        const ORIGIN3 = 1 << 2;
        const ANGLE = 1 << 3;
        const ANGLE2 = 1 << 4;
        const ANGLE3 = 1 << 5;
        const EFFECTS = 1 << 6;
        const SKINNUM = 1 << 7;
        const DEAD = 1 << 8;
        const GIB = 1 << 9;
        const WEAPONFRAME = 1 << 10;
        const MODEL = 1 << 11;
    }
}

#[derive(Debug, PartialEq, PartialOrd, Serialize)]
pub struct Playerinfo {
    pub player_number: u8,
    pub flags: DfTypes,
    pub frame: u8,
    pub origin: Option<CoordinateVectorOption>,
    pub angle: Option<AngleVectorOption>,
    pub model: Option<u8>,
    pub skinnum: Option<u8>,
    pub effects: Option<u8>,
    pub weaponframe: Option<u8>,
}

impl Playerinfo  {
    pub fn read(message: &mut Message) -> Result<ServerMessage, MessageError> {
        let player_number = message.read_u8(false)?;
        let u = message.read_u16(false)?;
        let flags = DfTypes::from_bits_truncate(u);
        let frame = message.read_u8(false)?;
        let mut origin = None;
        let mut origin_read = false;
        let mut origin_vec =  CoordinateVectorOption{x: None, y: None, z: None};
        for i in 0..3 {
            let f = DfTypes::from_bits_truncate(DfTypes::ORIGIN.bits() << i);
            if flags.contains(f) {
                if !origin_read {
                    origin_read = true;
                }
                let f = message.read_coordinate(false)?;
                match i {
                    0 => origin_vec.x = Some(f),
                    1 => origin_vec.y = Some(f),
                    2 => origin_vec.z = Some(f),
                    _ => return Err(MessageError::StringError("this is weird".to_string())),
                }
            }
        }
        if origin_read {
            origin = Some(origin_vec);
        }
        let mut angle = None;
        let mut angle_read = false;
        let mut angle_vec =  AngleVectorOption{x: None, y: None, z: None};
        for i in 0..3 {
            let f = DfTypes::from_bits_truncate(DfTypes::ANGLE.bits() << i);
            if flags.contains(f) {
                if !angle_read {
                    angle_read = true;
                }
                let f = message.read_angle16(false)?;
                match i {
                    0 => angle_vec.x = Some(f),
                    1 => angle_vec.y = Some(f),
                    2 => angle_vec.z = Some(f),
                    _ => return Err(MessageError::StringError("this is weird".to_string())),
                }
            }
        }
        if angle_read {
            angle = Some(angle_vec);
        }

        let mut model = None;
        if flags.contains(DfTypes::MODEL) {
            let i = message.read_u8(false)?;
            model = Some(i);
        }

        let mut skinnum = None;
        if flags.contains(DfTypes::SKINNUM) {
            let i = message.read_u8(false)?;
            skinnum = Some(i);
        }

        let mut effects = None;
        if flags.contains(DfTypes::EFFECTS) {
            let i = message.read_u8(false)?;
            effects = Some(i);
        }

        let mut weaponframe = None;
        if flags.contains(DfTypes::WEAPONFRAME) {
            let i = message.read_u8(false)?;
            weaponframe = Some(i);
        }
        Ok(ServerMessage::Playerinfo(Playerinfo{
            player_number,
            flags,
            frame,
            origin,
            angle,
            model,
            skinnum,
            effects,
            weaponframe
        }))

    }
}

bitflags! {
#[derive(Serialize)]
    pub struct FteProtocolExtensions: u32 {
	const TRANS             = 0x00000008; // .alpha support
	const ACCURATETIMINGS   = 0x00000040;
	const HLBSP             = 0x00000200; //stops fte servers from complaining
	const MODELDBL          = 0x00001000;
	const ENTITYDBL         = 0x00002000; //max =of 1024 ents instead of 512
	const ENTITYDBL2        = 0x00004000; //max =of 1024 ents instead of 512
	const FLOATCOORDS       = 0x00008000; //supports =floating point origins.
	const SPAWNSTATIC2      = 0x00400000; //Sends =an entity delta instead of a baseline.
	const PACKETENTITIES_256 = 0x01000000; //Client =can recieve 256 packet entities.
	const CHUNKEDDOWNLOADS  = 0x20000000; //alternate =file download method. Hopefully it'll give quadroupled download speed, especially on higher pings.
    }
}

bitflags! {
#[derive(Serialize)]
    pub struct MvdProtocolExtensions: u32 {
        const FLOATCOORDS = 0x00000001; // FTE_PEXT_FLOATCOORDS but for entity/player coords only
        const HIGHLAGTELEPORT = 0x00000002; // Adjust movement direction for frames following teleport
    }
}

bitflags! {
#[derive(Serialize)]
    struct UpdateTypes: u16 {
        const ORIGIN1  = 1 << 9;
        const ORIGIN2  = 1 << 10;
        const ORIGIN3  = 1 << 11;
        const ANGLE2   = 1 << 12;
        const FRAME    = 1 << 13;
        const REMOVE   = 1 << 14; // REMOVE this entity, don't add it;
        const MOREBITS = 1 << 15;
        // if MOREBITS is set, these additional flags are read in next
        const ANGLE1   = 1 << 0;
        const ANGLE3   = 1 << 1;
        const MODEL    = 1 << 2;
        const COLORMAP = 1 << 3;
        const SKIN     = 1 << 4;
        const EFFECTS  = 1 << 5;
        const SOLID    = 1 << 6; // the entity should be solid for prediction;
    }
}

#[derive(Debug, PartialEq, PartialOrd, Serialize)]
pub struct Packetentity {
    pub baseline_index: u8,
    pub bits: u16,
    pub remove: bool,
    pub model: Option<u8>,
    pub frame: Option<u8>,
    pub colormap: Option<u8>,
    pub skin: Option<u8>,
    pub effects: Option<u8>,
    pub origin: Option<CoordinateVectorOption>,
    pub angle: Option<AngleVectorOption>
}

#[derive(Debug, PartialEq, PartialOrd, Serialize)]
pub struct Packetentities {
    pub entities: Vec<Packetentity>
}

impl Packetentities  {
    pub fn read(message: &mut Message) -> Result<ServerMessage, MessageError> {
        let mut entities = Vec::new();
        let mut i = 0;
        loop {
            i = i +1;
            let mut bits = message.read_u16(false)?;
            if bits == 0 {
                break
            }
            let baseline_index = (bits & 511) as u8;
            bits = bits & !511;
            let mut flags = UpdateTypes::from_bits_truncate(bits);
            if flags.contains(UpdateTypes::MOREBITS) {
                let morebits = message.read_u8(false)?;
                bits |= morebits as u16;
                flags = UpdateTypes::from_bits_truncate(bits);
            }

            let mut remove = false;
            if flags.contains(UpdateTypes::REMOVE) {
                remove = true;
            }

            let mut model = None;
            if flags.contains(UpdateTypes::MODEL) {
                let tmp = message.read_u8(false)?;
                model = Some(tmp);
            }

            let mut frame = None;
            if flags.contains(UpdateTypes::FRAME) {
                let tmp = message.read_u8(false)?;
                frame = Some(tmp);
            }

            let mut colormap = None;
            if flags.contains(UpdateTypes::COLORMAP) {
                let tmp = message.read_u8(false)?;
                colormap = Some(tmp);
            }

            let mut skin = None;
            if flags.contains(UpdateTypes::SKIN) {
                let tmp = message.read_u8(false)?;
                skin = Some(tmp);
            }

            let mut effects = None;
            if flags.contains(UpdateTypes::EFFECTS) {
                let tmp = message.read_u8(false)?;
                effects = Some(tmp);
            }

            let mut origin = None;
            let mut origin_internal = CoordinateVectorOption::empty();

            let mut angle = None;
            let mut angle_internal = AngleVectorOption::empty();

            if flags.contains(UpdateTypes::ORIGIN1) {
                let tmp = message.read_coordinate(false)?;
                origin_internal.x = Some(tmp);
            }

            if flags.contains(UpdateTypes::ANGLE1) {
                let tmp = message.read_angle(false)?;
                angle_internal.x = Some(tmp);
            }

            if flags.contains(UpdateTypes::ORIGIN2) {
                let tmp = message.read_coordinate(false)?;
                origin_internal.y = Some(tmp);
            }

            if flags.contains(UpdateTypes::ANGLE2) {
                let tmp = message.read_angle(false)?;
                angle_internal.y = Some(tmp);
            }

            if flags.contains(UpdateTypes::ORIGIN3) {
                let tmp = message.read_coordinate(false)?;
                origin_internal.z = Some(tmp);
            }

            if flags.contains(UpdateTypes::ANGLE3) {
                let tmp = message.read_angle(false)?;
                angle_internal.z = Some(tmp);
            }

            if !origin_internal.is_empty() {
                origin = Some(origin_internal);
            }

            if !angle_internal.is_empty() {
                angle = Some(angle_internal);
            }

            let p = Packetentity{
                baseline_index,
                bits,
                remove,
                model,
                frame,
                colormap,
                skin,
                effects,
                origin,
                angle
            };
            entities.push(p);
            if i == 65 {
                panic!();
            }
        }
        return Ok(ServerMessage::Packetentities(Packetentities{
            entities
        }))
    }
}

#[derive(Debug, PartialEq, PartialOrd, Serialize)]
pub struct Deltapacketentities {
    pub from: u8,
    pub entities: Vec<Packetentity>
}

impl Deltapacketentities  {
    pub fn read(message: &mut Message) -> Result<ServerMessage, MessageError> {
        let mut entities = Vec::new();
        let mut i = 0;
        let from = message.read_u8(false)?;
        loop {

            i = i +1;
            let mut bits = message.read_u16(false)?;
            if bits == 0 {
                break
            }
            let baseline_index = (bits & 511) as u8;
            bits = bits & !511;
            let mut flags = UpdateTypes::from_bits_truncate(bits);
            if flags.contains(UpdateTypes::MOREBITS) {
                let morebits = message.read_u8(false)?;
                bits |= morebits as u16;
                flags = UpdateTypes::from_bits_truncate(bits);
            }

            if bits == 0 {
                break;
            }

            let mut remove = false;
            if flags.contains(UpdateTypes::REMOVE) {
                remove = true;
            }

            let mut model = None;
            if flags.contains(UpdateTypes::MODEL) {
                let tmp = message.read_u8(false)?;
                model = Some(tmp);
            }

            let mut frame = None;
            if flags.contains(UpdateTypes::FRAME) {
                let tmp = message.read_u8(false)?;
                frame = Some(tmp);
            }

            let mut colormap = None;
            if flags.contains(UpdateTypes::COLORMAP) {
                let tmp = message.read_u8(false)?;
                colormap = Some(tmp);
            }

            let mut skin = None;
            if flags.contains(UpdateTypes::SKIN) {
                let tmp = message.read_u8(false)?;
                skin = Some(tmp);
            }

            let mut effects = None;
            if flags.contains(UpdateTypes::EFFECTS) {
                let tmp = message.read_u8(false)?;
                effects = Some(tmp);
            }

            let mut origin = None;
            let mut origin_internal = CoordinateVectorOption::empty();

            let mut angle = None;
            let mut angle_internal = AngleVectorOption::empty();

            if flags.contains(UpdateTypes::ORIGIN1) {
                let tmp = message.read_coordinate(false)?;
                origin_internal.x = Some(tmp);
            }

            if flags.contains(UpdateTypes::ANGLE1) {
                let tmp = message.read_angle(false)?;
                angle_internal.x = Some(tmp);
            }

            if flags.contains(UpdateTypes::ORIGIN2) {
                let tmp = message.read_coordinate(false)?;
                origin_internal.z = Some(tmp);
            }

            if flags.contains(UpdateTypes::ANGLE2) {
                let tmp = message.read_angle(false)?;
                angle_internal.y = Some(tmp);
            }

            if flags.contains(UpdateTypes::ORIGIN3) {
                let tmp = message.read_coordinate(false)?;
                origin_internal.z = Some(tmp);
            }

            if flags.contains(UpdateTypes::ANGLE3) {
                let tmp = message.read_angle(false)?;
                angle_internal.z = Some(tmp);
            }

            if !origin_internal.is_empty() {
                origin = Some(origin_internal);
            }

            if !angle_internal.is_empty() {
                angle = Some(angle_internal);
            }

            let p = Packetentity{
                baseline_index,
                bits,
                remove,
                model,
                frame,
                colormap,
                skin,
                effects,
                origin,
                angle
            };
            entities.push(p);
        }
        return Ok(ServerMessage::Deltapacketentities(Deltapacketentities{
            from,
            entities
        }))
    }
}

#[derive(Debug, PartialEq, PartialOrd, ParseMessage, Serialize)]
pub struct Setinfo {
    pub player_number: u8,
    pub key: StringByte,
    pub value: StringByte,
}

#[derive(Debug, PartialEq, PartialOrd, ParseMessage, Serialize)]
pub struct Print {
    pub from: u8,
    pub message: StringByte,
}

#[derive(Debug, PartialEq, PartialOrd, Serialize)]
pub struct Sound {
    pub channel: u16,
    pub entity: u16,
    pub index: u8,
    pub volume: Option<u8>,
    pub attenuation: Option<u8>,
    pub origin: CoordinateVector
}

impl Sound {
    pub fn read(message: &mut Message) -> Result<ServerMessage, MessageError> {
        let channel = message.read_u16(false)?;
        let mut volume = None;
        let mut attenuation = None;
        let mut origin = CoordinateVector{..Default::default()};

        if channel & 1 << 15 == 1 << 15 {
            let b = message.read_u8(false)?;
            volume  = Some(b);
        }

        if channel & 1 << 14 == 1 << 14 {
            let b = message.read_u8(false)?;
            attenuation = Some(b);
        }

        let entity  = (channel >> 3) & 1023;

        let index = message.read_u8(false)?;

        origin.x = message.read_coordinate(false)?;
        origin.y = message.read_coordinate(false)?;
        origin.z = message.read_coordinate(false)?;

        return Ok(ServerMessage::Sound(Sound{
            channel,
            entity,
            index,
            volume,
            attenuation,
            origin
        }))
    }
}

#[derive(Debug, PartialEq, PartialOrd, ParseMessage, Serialize)]
pub struct Damage {
    pub armor: u8,
    pub blood: u8,
    pub origin: CoordinateVector
}

#[derive(Debug, PartialEq, PartialOrd, ParseMessage, Serialize)]
pub struct Setangle {
    pub index: u8,
    pub angle:AngleVector 
}

#[derive(Debug, PartialEq, PartialOrd, ParseMessage, Serialize)]
pub struct Setview {
    pub setview: u16,
}

#[derive(Debug, PartialEq, PartialOrd, ParseMessage, Serialize)]
pub struct Smallkick {
}

#[derive(Debug, PartialEq, PartialOrd, ParseMessage, Serialize)]
pub struct Bigkick {
}

#[derive(Debug, PartialEq, PartialOrd, ParseMessage, Serialize)]
pub struct Muzzleflash {
    pub unknown: u16,
}

#[derive(Debug, PartialEq, PartialOrd, ParseMessage, Serialize)]
pub struct Chokecount {
    pub chokecount: u8,
}

#[derive(Debug, PartialEq, PartialOrd, ParseMessage, Serialize)]
pub struct Intermission {
    pub origin: CoordinateVector,
    pub angle: AngleVector,
}


#[derive(Debug, PartialEq, PartialOrd, ParseMessage, Serialize)]
pub struct Disconnect {
}



#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, TryFromPrimitive, Display, Serialize)]
#[repr(u8)]
pub enum TempEntityType {
    Spike = 0,
    SuperSpike,
    Gunshot,
    Explosion,
    Tarexplosion,
    Lightning1,
    Lightning2,
    Wizspike,
    Knightspike,
    Lightning3,
    LavaSplash,
    Teleport,
    Blood,
    LightningBlood
}

#[derive(Debug, PartialEq, PartialOrd, Serialize)]
pub struct Tempentity {
    pub r#type: TempEntityType,
    pub origin: CoordinateVector,
    pub start: CoordinateVector,
    pub entity: u16,
    pub count: i8
}

impl Tempentity {
    pub fn read(message: &mut Message) -> Result<ServerMessage, MessageError> {
        let t = message.read_u8(false)?;
        let r#type = TempEntityType::try_from(t)?;

        let mut count = 0 as i8;
        if r#type == TempEntityType::Gunshot || r#type == TempEntityType::Blood {
            count = message.read_i8(false)?;
        }

        let mut entity = 0 as u16;
        let mut start =  CoordinateVector{..Default::default()};
        if r#type == TempEntityType::Lightning1
            || r#type == TempEntityType::Lightning2 
            || r#type == TempEntityType::Lightning3 {
            entity = message.read_u16(false)?;
            start.x = message.read_coordinate(false)?;
            start.y = message.read_coordinate(false)?;
            start.z = message.read_coordinate(false)?;
        }

        let mut origin =  CoordinateVector{..Default::default()};
        origin.x = message.read_coordinate(false)?;
        origin.y = message.read_coordinate(false)?;
        origin.z = message.read_coordinate(false)?;

        return Ok(ServerMessage::Tempentity(Tempentity{
            r#type,
            origin,
            start,
            entity,
            count,
        }))
    }
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, TryFromPrimitive, Display, Serialize)]
#[repr(u8)]
pub enum ServerClient {
    Bad                = 0,
    Nop                 = 1,
    Disconnect          = 2,
    Updatestat          = 3,  // [byte] [byte]
    NQVersion          = 4,  // [long] server version
    Setview             = 5,  // [short] entity number
    Sound               = 6,  // <see code>
    NQTime             = 7,  // [float] server time
	Print               = 8,  // [byte] id [string] null terminated string
	Stufftext           = 9,  // [string] stuffed into client's console buffer
	Setangle            = 10, // [angle3] set the view angle to this absolute value
	Serverdata          = 11, // [long] protocol ...
	Lightstyle          = 12, // [byte] [string]
	NQUpdatename       = 13, // [byte] [string]
	Updatefrags         = 14, // [byte] [short]
	NQClientdata       = 15, // <shortbits + data>
	Stopsound           = 16, // <see code>
	NQUpdatecolors     = 17, // [byte] [byte] [byte]
	NQParticle         = 18, // [vec3] <variable>
	Damage              = 19,
	Spawnstatic         = 20,
	Spawnbinary         = 21,
	Spawnbaseline       = 22,
	Tempentity         = 23, // variable
	Setpause            = 24, // [byte] on / off
	NQSignonnum        = 25, // [byte]  used for the signon sequence
	Centerprint         = 26, // [string] to put in center of the screen
	Killedmonster       = 27,
	Foundsecret         = 28,
	Spawnstaticsound    = 29, // [coord3] [byte] samp [byte] vol [byte] aten
	Intermission        = 30, // [vec3_t] origin [vec3_t] angle
	Finale              = 31, // [string] text
	Cdtrack             = 32, // [byte] track
	Sellscreen          = 33,
	//NQCutscene         = 34, // same as svc_smallkick
	Smallkick           = 34, // set client punchangle to 2
	Bigkick             = 35, // set client punchangle to 4
	Updateping          = 36, // [byte] [short]
	Updateentertime     = 37, // [byte] [float]
	Updatestatlong      = 38, // [byte] [long]
	Muzzleflash         = 39, // [short] entity
	Updateuserinfo      = 40, // [byte] slot [long] uid [string] userinfo
	Download            = 41, // [short] size [size bytes]
	Playerinfo          = 42, // variable
	Nails               = 43, // [byte] num [48 bits] xyzpy 12 12 12 4 8
	Chokecount          = 44, // [byte] packets choked
	Modellist           = 45, // [strings]
	Soundlist           = 46, // [strings]
	Packetentities      = 47, // [...]
	Deltapacketentities = 48, // [...]
	Maxspeed            = 49, // maxspeed change, for prediction
	Entgravity          = 50, // gravity change, for prediction
	Setinfo             = 51, // setinfo on a client
	Serverinfo          = 52, // serverinfo
	Updatepl            = 53, // [byte] [byte]
	Nails2              = 54,
	FteModellistshort  = 60,
	FteSpawnbaseline2  = 66,
	Qizmovoice          = 83,
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, TryFromPrimitive, Display, Serialize)]
#[repr(u8)]
pub enum CommandCode {
    S2cChallenge = 'c' as u8,
    S2cConnection = 'j' as u8,
}

#[derive(Debug, PartialEq, PartialOrd, Serialize)]
pub struct ConnectionLessServerChallenge {
    pub challenge: i32,
    pub protocol: MessageFlags,
}

#[derive(Debug, PartialEq, PartialOrd, Default, Serialize)]
pub struct Connected {
    pub incoming_sequence: u32,
    pub outgoing_sequence: u32,
    pub messages: Vec<ServerMessage>,
}

#[derive(Debug, PartialEq, PartialOrd, Display, Serialize)]
pub enum Packet {
    Error,
    ConnectionLessServerChallenge(ConnectionLessServerChallenge),
    ConnectionLessServerConnection,
    Connected(Connected)
}

macro_rules! initialize_message_type {
    ($($ty:ident), *) => {
        paste! {
            #[derive(Debug, PartialEq, PartialOrd, Display, Serialize)]
            pub enum ServerMessage {
                $(
                [< $ty >]([< $ty >]),
                )*
            }
        impl ServerClient {
            pub fn  read_message  (self,  message: &mut Message) ->  Result<ServerMessage, MessageError> {
                match self {
                $(
                    ServerClient::[<$ty>] => {
                        return [< $ty >]::read(message);
                    }
                 )*
                    _ => {
                        return Err(MessageError::UnhandledType(self))
                    }
                }
            }
            }
        }
    }
}

initialize_message_type!(Serverdata, Soundlist, Modellist,Cdtrack, Stufftext, Spawnstatic,Spawnbaseline, Spawnstaticsound, Updatefrags, Updateping, Updatepl, Updateentertime, Updateuserinfo, Playerinfo, Updatestatlong, Updatestat, Lightstyle, Serverinfo, Centerprint, Packetentities, Deltapacketentities, Tempentity, Setinfo, Print, Sound, Damage, Setangle, Smallkick, Bigkick, Muzzleflash, Chokecount, Intermission, Disconnect, Setview);

