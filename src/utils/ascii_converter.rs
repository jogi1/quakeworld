use std::error::Error;
use serde::Serialize;
use simple_error::bail;
use crate::protocol::types::StringByte;



#[derive(Debug, Serialize,Clone)]
pub struct AsciiConverter {
    table: Vec<u8>,
}

const ASCII_TABLE: &str = "________________[]0123456789____ !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_'abcdefghijklmnopqrstuvwxyz{|}~_________________[]0123456789____ !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_'abcdefghijklmnopqrstuvwxyz{|}~_";


lazy_static! {
    static ref GLOBAL_ASCII_TABLE: Vec<u8> = {
        ASCII_TABLE.as_bytes().to_vec()
    };
}

impl Default for AsciiConverter {
    fn default() -> Self {
        AsciiConverter{
           table: GLOBAL_ASCII_TABLE.to_vec(),
        }
    }
}

impl AsciiConverter {
    pub fn new_with_table(table: Vec<u8>) -> Result<AsciiConverter, Box<dyn Error>> {
        if table.len() != 256 {
            bail!("table size needs to be 256 was {}", table.len())
        }
        Ok(AsciiConverter{
            table,
        })
    }

    pub fn new() -> AsciiConverter {
        AsciiConverter{
            ..Default::default()
        }
    }
    pub fn convert(&self, string: impl Into<Vec<u8>>) -> String {
        let string = string.into();
        let mut out:String = String::new();
        for c in string {
            out.push(self.table[c as usize] as char);
        }
        out
    }

    pub fn convert_single(&self, single: u8) -> u8 {
        self.table[single as usize]
    }

#[cfg(feature = "ascii_strings")]
    pub fn convert_to_stringbyte(&self, bytes: impl Into<Vec<u8>>) -> StringByte {
        let bytes = bytes.into();
        StringByte::new(bytes.to_vec(), self)
    }
#[cfg(not(feature = "ascii_strings"))]
    pub fn convert_to_stringbyte(&self, bytes: impl Into<Vec<u8>>) -> StringByte {
        let bytes = bytes.into();
        StringByte{ bytes }
    }

}
