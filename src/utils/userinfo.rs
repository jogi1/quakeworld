
use serde::Serialize;

#[cfg(feature = "ascii_strings")]
use crate::utils::ascii_converter::AsciiConverter;
use crate::protocol::types::StringByte;


#[derive(Clone, Debug, Serialize, Default)]
pub struct Userinfo {
#[cfg(feature = "ascii_strings")]
    ascii_converter: AsciiConverter,
    pub values: Vec<(StringByte, StringByte)>,
}

impl Userinfo {
    pub fn new() -> Userinfo {
        return Userinfo{ ..Default::default()}
    }
#[cfg(feature = "ascii_strings")]
    pub fn new_with_ascii_converter(ascii_converter: AsciiConverter) -> Userinfo {
        return Userinfo{
            ascii_converter,
            ..Default::default()
        }
    }

#[cfg(feature = "ascii_strings")]
    pub fn update_key_value(&mut self, key: &StringByte, value: &StringByte) {
        for i in 0..self.values.len() {
            let (k, _) = &self.values[i];
            if k.string == key.string {
                self.values[i] = (key.clone(), value.clone());
                return
            }
        }
        self.values.push((key.clone(), value.clone()));
    }

#[cfg(not(feature = "ascii_strings"))]
    pub fn update_key_value(&mut self, key: &StringByte, value: &StringByte) {
        for i in 0..self.values.len() {
            let (k, _) = &self.values[i];
            if *k == *key {
                self.values[i] = (key.clone(), value.clone());
                return
            }
        }
        self.values.push((key.clone(), value.clone()));
    }


#[cfg(feature = "ascii_strings")]
    pub fn update(&mut self, userinfo: &StringByte) {
        let mut start = true;
        let mut key_vec: Vec<u8> = vec![];
        let mut v:Vec<u8> = vec![];
        let mut key = true;
        for i in 0..userinfo.bytes.len() {
            if userinfo.bytes[i] == 92 {
                if start {
                    start = false;
                    continue;
                } else {
                    if key {
                        key_vec = v.clone();
                    } else {
                        let sb_k = StringByte{
                                    string: self.ascii_converter.convert(&key_vec),
                                    bytes: key_vec.clone()};
                        let sb_v = StringByte{
                                    string: self.ascii_converter.convert(&v),
                                    bytes: v.clone()};
                        self.values.push((sb_k ,sb_v))
                    }
                    key = !key;
                    v.clear();
                    continue;
                }
            }
            v.push(userinfo.bytes[i]);
        }
    }

#[cfg(not(feature = "ascii_strings"))]
    pub fn update(&mut self, userinfo: &StringByte) {
        let mut start = true;
        let mut key_vec: Vec<u8> = vec![];
        let mut v:Vec<u8> = vec![];
        let mut key = true;
        for i in 0..userinfo.len() {
            if userinfo[i] == 92 {
                if start {
                    start = false;
                    continue;
                } else {
                    if key {
                        key_vec = v.clone();
                    } else {
                        self.values.push((
                                    key_vec.clone(),
                                    v.clone(),
                                    ))
                    }
                    key = !key;
                    v.clear();
                    continue;
                }
            }
            v.push(userinfo[i]);
        }
    }

}
