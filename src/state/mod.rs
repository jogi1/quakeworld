use serde::Serialize;
use crate::protocol::types::*;
use crate::utils::userinfo::Userinfo;
#[cfg(feature = "ascii_strings")]
use crate::utils::ascii_converter::AsciiConverter;
use crate::mvd::MvdTarget;
use std::collections::HashMap;


pub type Stat = [i32;32];

#[derive(Serialize, Clone, Debug, Default)]
pub struct Player {
    pub frags: i16,
    pub ping: u16,
    pub pl: u8,
    pub entertime: f32,
    pub uid: u32,
    pub userinfo: Userinfo,
    pub name: StringByte,
    pub team: StringByte,
    pub spectator: bool,
    pub top_color: StringByte,
    pub bottom_color: StringByte,
    pub origin: CoordinateVector,
    pub angle: AngleVector,
    pub model: u8,
    pub skinnum: u8,
    pub effects: u8,
    pub weaponframe: u8,
    pub stats: Stat,
}

impl Player {
/// updates the userinfo of this [`Player`].
#[cfg(feature = "ascii_strings")]
    fn update_userinfo(&mut self) {
        for (k, v) in self.userinfo.values.iter() {
            if k.string == "team" {
                self.team = v.clone();
            }
            if k.string == "name" {
                self.name= v.clone();
            }
        }
    }

/// the userinfo isnt read into the [`Player`] when ascii_strings is disabled.
#[cfg(not(feature = "ascii_strings"))]
    fn update_userinfo(&mut self) {
    }
}

#[derive(Serialize, Copy, Clone, Debug, Default)]
pub struct Entity {
    pub index: u16,
    pub model: u16,
    pub frame: u8,
    pub colormap: u8,
    pub skinnum: u8,
    pub effects: u8,
    pub origin: CoordinateVector,
    pub angle: AngleVector,
}

impl Entity {
    /// apply [`ServerMessage::Packetentity`] as a deltapacket_entity to this [`Entity`]
    pub fn apply_delta(&mut self, delta: &Packetentity) {
        if delta.model.is_some() {
            let v = delta.model.unwrap();
            self.model = v;
        }
        if delta.frame.is_some() {
            let v = delta.frame.unwrap();
            self.frame= v;
        }
        if delta.colormap.is_some() {
            let v = delta.colormap.unwrap();
            self.frame= v;
        }
        if delta.skin.is_some() {
            let v = delta.skin.unwrap();
            self.frame= v;
        }
        if delta.effects.is_some() {
            let v = delta.effects.unwrap();
            self.frame= v;
        }
        if delta.origin.is_some() {
            let v = delta.origin.unwrap();
            v.apply_to(&mut self.origin);
        }
        if delta.angle.is_some() {
            let v = delta.angle.unwrap();
            v.apply_to(&mut self.angle);
        }

    }

    /// create [`Entity`] from [`ServerMessage::Spawnbaseline`]
    pub fn from_baseline(baseline: &Spawnbaseline) -> Entity {
        Entity {
            index: baseline.model_index as u16,
            frame: baseline.model_frame,
            colormap: baseline.colormap,
            skinnum: baseline.skinnum,
            origin: baseline.origin,
            angle: baseline.angle,
            ..Default::default()
        }
    }

    /// create [`Entity`] from [`ServerMessage::Spawnstatic`]
    pub fn from_static(static_ent: &Spawnstatic) -> Entity {
        Entity {
            index: static_ent.model_index as u16,
            frame: static_ent.model_frame,
            colormap: static_ent.colormap,
            skinnum: static_ent.skinnum,
            origin: static_ent.origin,
            angle: static_ent.angle,
            ..Default::default()
        }
    }

    /// create [`Entity`] from [`ServerMessage::Packetentity`]
    pub fn from_packetentity(packet_entity: &Packetentity) -> Entity {
        let mut angle = AngleVector{ ..Default::default()};
        if packet_entity.angle.is_some() {
            packet_entity.angle.unwrap().apply_to(&mut angle);
        }
        let mut origin  = CoordinateVector{ ..Default::default()};
        if packet_entity.origin.is_some() {
            packet_entity.origin.unwrap().apply_to(&mut origin);
        }
        Entity {
            index: packet_entity.entity_index as u16,
            frame: packet_entity.frame.unwrap_or(0),
            model: packet_entity.model.unwrap_or(0),
            colormap: packet_entity.colormap.unwrap_or(0),
            skinnum: packet_entity.skin.unwrap_or(0),
            effects: packet_entity.effects.unwrap_or(0),
            origin,
            angle,
        }
    }
}

    /*
#[derive(Serialize, Copy, Clone, Debug)]
pub struct Sound {
    pub channel: u16,
    pub entity: u16,
    pub index: u8,
    pub volume: u8,
    pub attenuation: u8,
    pub origin: CoordinateVector
}

impl Sound {
    pub fn from_static(static_sound: &Spawnstaticsound) -> Sound {
        return Sound {
        }
    }
}
*/

#[derive(Serialize, Clone, Default, Debug)]
pub struct State {
#[cfg(feature = "ascii_strings")]
    ascii_converter: AsciiConverter,
    pub serverdata: Serverdata,
    pub players: HashMap<u16, Player>,
    pub sounds: Vec<StringByte>,
    pub models: Vec<StringByte>,
    pub baseline_entities: HashMap<u16, Entity>,
    pub static_entities: Vec<Spawnstatic>,
    pub entities: HashMap<u16, Entity>,
    pub temp_entities: HashMap<u16, Tempentity>,
    pub static_sounds: Vec<Spawnstaticsound>,
}

impl State {
    pub fn new() -> State {
        State{
            ..Default::default()
        }
    }

#[cfg(feature = "ascii_strings")]
    pub fn new_with_ascii_conveter(ascii_converter: AsciiConverter) -> State {
        State{
            ascii_converter,
            ..Default::default()
        }
    }

    fn update_player(&mut self, player_index: u16, message: &ServerMessage) {
        let p = self.players.get_mut(&player_index);
        let mut player = match p {
            Some(player) =>  player,
            None => { self.players.insert(player_index, Player{..Default::default()});
                self.players.get_mut(&player_index).unwrap()
            },
        };
        /*
        if let Some(player) = p {
            player
        } else {
            self.players.insert(player_index, Player{..Default::default()});
            self.players.get_mut(&player_index).unwrap()
        };
        */
        match message {
            ServerMessage::Updatefrags(data) => {
                player.frags = data.frags;
            }
            ServerMessage::Updateping(data) => {
                player.ping = data.ping;
            }
            ServerMessage::Updatepl(data) => {
                player.pl = data.pl;
            }
            ServerMessage::Updateentertime(data) => {
                player.entertime = data.entertime;
            }
            ServerMessage::Updateuserinfo(data) => {
                player.uid = data.uid;
                player.userinfo.update(&data.userinfo);
                player.update_userinfo();
            }
            ServerMessage::Playerinfo(_data) => {
                //panic!("FIXME");
                /*
                if data.origin.is_some() {
                    data.origin.unwrap().apply_to(&mut player.origin);
                }
                if data.angle.is_some() {
                    data.angle.unwrap().apply_to(&mut player.angle);
                }
                player.model = data.model.unwrap_or(0);
                player.skinnum = data.skinnum.unwrap_or(0);
                player.effects = data.effects.unwrap_or(0);
                player.weaponframe = data.weaponframe.unwrap_or(0);
                */
            }
            ServerMessage::Updatestatlong(data) => {
                player.stats[data.stat as usize] = data.value;
            }
            ServerMessage::Updatestat(data) => {
                player.stats[data.stat as usize] = data.value as i32;
            }
            ServerMessage::Setinfo(data) => {
                player.userinfo.update_key_value(&data.key, &data.value);
                player.update_userinfo();
            }
            ServerMessage::Setangle(data) => {
                player.angle = data.angle;
            }
            _ => { panic!("{:?}, is not applicable to player", message)}
        }
    }

    fn packet_entities(&mut self, packet_entities: &Packetentities) {
        for packet_entity in &packet_entities.entities {
            self.baseline_entities.insert(packet_entity.entity_index, Entity::from_packetentity(packet_entity));
        }
    }

    fn deltapacket_entities(&mut self, deltapacket_entities: &Deltapacketentities) {
        for deltapacket_entity in &deltapacket_entities.entities {
            if deltapacket_entity.remove {
                self.entities.remove(&deltapacket_entity.entity_index);
            } else {
                let e = self.entities.get_mut(&deltapacket_entity.entity_index);
                if let Some(value) = e {
                    value.apply_delta(deltapacket_entity);
                } else {
                    // @TODO: FIXME
                    //println!("{:?}", deltapacket_entity);
                    //panic!("deltapacket implementation");
                }
            }
            //self.baseline_entities.insert(deltapacket_entity.entity_index, Entity::from_packetentity(&deltapacket_entity));
        }
    }

    fn temp_entities(&mut self, temp_entity: &Tempentity) {
        self.temp_entities.insert(temp_entity.entity,  temp_entity.clone());
    }

    pub fn apply_messages_mvd(&mut self, messages: &'_ Vec<ServerMessage>, last: MvdTarget) {
        for message in messages {
            match message {
                ServerMessage::Serverdata(data) => {
                    self.serverdata = data.clone();
                },
                ServerMessage::Soundlist(data) => {
                    self.sounds.extend(data.sounds.clone());
                }
                ServerMessage::Modellist(data) => {
                    self.models.extend(data.models.clone());
                }
                ServerMessage::Spawnbaseline(data) => {
                    self.baseline_entities.insert(data.index, Entity::from_baseline(data));
                }
                ServerMessage::Spawnstatic(data) => {
                    self.static_entities.push(*data)
                }
                ServerMessage::Cdtrack(_) => {
                    continue
                }
                ServerMessage::Stufftext(_) => {
                    continue
                },
                ServerMessage::Spawnstaticsound(data) => {
                    self.static_sounds.push(data.clone());
                }
                ServerMessage::Updatefrags(data) => {
                    self.update_player(data.player_number as u16, message);
                }
                ServerMessage::Updateping(data) => {
                    self.update_player(data.player_number as u16, message);
                }
                ServerMessage::Updatepl(data) => {
                    self.update_player(data.player_number as u16, message);
                }
                ServerMessage::Updateentertime(data) => {
                    self.update_player(data.player_number as u16, message);
                }
                ServerMessage::Updateuserinfo(data) => {
                    self.update_player(data.player_number as u16, message);
                }
                ServerMessage::Playerinfo(_data) => {
                    //panic!("FIXME");
                    /*
                    self.update_player(data.player_number as u16, message);
                    */
                }
                ServerMessage::Updatestatlong(_) => {
                    self.update_player(last.to as u16, message);
                }
                ServerMessage::Updatestat(_) => {
                    self.update_player(last.to as u16, message);
                }
                ServerMessage::Setinfo(data) => {
                    self.update_player(data.player_number as u16, message);
                }
                ServerMessage::Lightstyle(_) => {
                    // ignore
                }
                ServerMessage::Serverinfo(_) => {
                    // ignore, but probably shouldnt be
                }
                ServerMessage::Centerprint(_) => {
                    // ignore
                }
                ServerMessage::Packetentities(data) => {
                    self.packet_entities(data);
                }
                ServerMessage::Deltapacketentities(data) => {
                    self.deltapacket_entities(data);
                }
                ServerMessage::Tempentity(data) => {
                    self.temp_entities(data);
                }
                ServerMessage::Print(_) => {
                    // ignore
                }
                ServerMessage::Sound(_) => {
                    // maybe keep the sounds?
                }
                ServerMessage::Damage(_) => {
                    // ignore
                }
                ServerMessage::Setangle(data) => {
                    self.update_player(data.index as u16, message);
                }
                ServerMessage::Smallkick(_) => {
                    // ignore
                }
                ServerMessage::Muzzleflash(_) => {
                    // ignore
                }
                ServerMessage::Chokecount(_) => {
                    // ignore
                }
                ServerMessage::Bigkick(_) => {
                    // ignore
                }
                ServerMessage::Intermission(_) => {
                    // ignore
                }
                ServerMessage::Disconnect(_) => {
                    // ignore
                }
                _ => { panic!("noooo! {:?} to: {}, type: {}", message, last.to, last.command)}
            }
        }
    }

    pub fn apply_messages(&mut self, messages: &'_ Vec<ServerMessage>) {
        for message in messages {
            match message {
                ServerMessage::Serverdata(data) => {
                    self.serverdata = data.clone();
                },
                ServerMessage::Soundlist(data) => {
                    self.sounds.extend(data.sounds.clone());
                }
                ServerMessage::Modellist(data) => {
                    self.models.extend(data.models.clone());
                }
                ServerMessage::Spawnbaseline(data) => {
                    self.baseline_entities.insert(data.index, Entity::from_baseline(data));
                }
                ServerMessage::Spawnstatic(data) => {
                    self.static_entities.push(*data)
                }
                ServerMessage::Cdtrack(_) => {
                    continue
                }
                ServerMessage::Stufftext(_) => {
                    continue
                },
                ServerMessage::Spawnstaticsound(data) => {
                    self.static_sounds.push(data.clone());
                }
                ServerMessage::Updatefrags(data) => {
                    self.update_player(data.player_number as u16, message);
                }
                ServerMessage::Updateping(data) => {
                    self.update_player(data.player_number as u16, message);
                }
                ServerMessage::Updatepl(data) => {
                    self.update_player(data.player_number as u16, message);
                }
                ServerMessage::Updateentertime(data) => {
                    self.update_player(data.player_number as u16, message);
                }
                ServerMessage::Updateuserinfo(data) => {
                    self.update_player(data.player_number as u16, message);
                }
                ServerMessage::Playerinfo(_data) => {
                    panic!("FIXME");
                    //self.update_player(data.player_number as u16, message);
                }
                ServerMessage::Updatestatlong(_) => {
                //    self.update_player(last_to as u16, message);
                }
                ServerMessage::Updatestat(_) => {
                //    self.update_player(last_to as u16, message);
                }
                ServerMessage::Setinfo(data) => {
                    self.update_player(data.player_number as u16, message);
                }
                ServerMessage::Lightstyle(_) => {
                    // ignore
                }
                ServerMessage::Serverinfo(_) => {
                    // ignore, but probably shouldnt be
                }
                ServerMessage::Centerprint(_) => {
                    // ignore
                }
                ServerMessage::Packetentities(data) => {
                    self.packet_entities(data);
                }
                ServerMessage::Deltapacketentities(data) => {
                    self.deltapacket_entities(data);
                }
                ServerMessage::Tempentity(data) => {
                    self.temp_entities(data);
                }
                ServerMessage::Print(_) => {
                    // ignore
                }
                ServerMessage::Sound(_) => {
                    // maybe keep the sounds?
                }
                ServerMessage::Damage(_) => {
                    // ignore
                }
                ServerMessage::Setangle(data) => {
                    self.update_player(data.index as u16, message);
                }
                ServerMessage::Smallkick(_) => {
                    // ignore
                }
                ServerMessage::Muzzleflash(_) => {
                    // ignore
                }
                ServerMessage::Chokecount(_) => {
                    // ignore
                }
                ServerMessage::Bigkick(_) => {
                    // ignore
                }
                ServerMessage::Intermission(_) => {
                    // ignore
                }
                ServerMessage::Disconnect(_) => {
                    // ignore
                }
                _ => {}
            }
        }
    }
}

