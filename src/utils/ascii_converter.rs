use std::error::Error;
use serde::Serialize;
use simple_error::bail;


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
    pub fn new_with_table(table: Box<Vec<u8>>) -> Result<AsciiConverter, Box<dyn Error>> {
        if table.len() != 256 {
            bail!("table size needs to be 256 was {}", table.len())
        }
        return Ok(AsciiConverter{
            table: *table,
        })
    }

    pub fn new() -> AsciiConverter {
        AsciiConverter{
            ..Default::default()
        }
    }
    pub fn convert(&self, string: &Vec<u8>) -> String {
        let mut out:String = String::new();
        for c in string {
            out.push(self.table[*c as usize] as char);
        }
        return out
    }
}
