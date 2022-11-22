
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
        Userinfo{ ..Default::default()}
    }
#[cfg(feature = "ascii_strings")]
    pub fn new_with_ascii_converter(ascii_converter: AsciiConverter) -> Userinfo {
        Userinfo{
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
                                    string: self.ascii_converter.convert(key_vec.clone()),
                                    bytes: key_vec.clone()};
                        let sb_v = StringByte{
                                    string: self.ascii_converter.convert(v.clone()),
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
        for i in 0..userinfo.bytes.len() {
            if userinfo.bytes[i] == 92 {
                if start {
                    start = false;
                    continue;
                } else {
                    if key {
                        key_vec = v.clone();
                    } else {
                        self.values.push((
                                    StringByte{ bytes: key_vec.clone()},
                                    StringByte{ bytes: v.clone()},
                                    ))
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
    pub fn update_from_string(&mut self, key: impl Into<Vec<u8>>, value: impl Into<Vec<u8>>) {
        let key = key.into();
        let value = value.into();
        let sb_k = StringByte::new(key);
        let sb_v = StringByte::new(value);
        self.update_key_value(&sb_k, &sb_v);
    }

#[cfg(feature = "ascii_strings")]
    pub fn update_from_string(&mut self, key: impl Into<Vec<u8>>, value: impl Into<Vec<u8>>) {
        let key = key.into();
        let value = value.into();
        let sb_k = StringByte::new(key, &self.ascii_converter);
        let sb_v = StringByte::new(value, &self.ascii_converter);
        self.update_key_value(&sb_k, &sb_v);
    }

    pub fn as_bytes(&mut self) -> Vec<u8> {
        let mut rb: Vec<u8> = Vec::new();
        for i in 0..self.values.len() {
            let (k, v) = &self.values[i];
#[cfg(feature = "ascii_strings")]
            {
                rb.push(b'\\');
                rb.extend(k.bytes.clone());
                rb.push(b'\\');
                rb.extend(v.bytes.clone());
            }
#[cfg(not(feature = "ascii_strings"))]
            {
                rb.push(b'\\');
                rb.extend(k.bytes.clone());
                rb.push(b'\\');
                rb.extend(v.bytes.clone());
            }
        }
        rb
    }
}
