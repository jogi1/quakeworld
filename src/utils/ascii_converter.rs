use std::error::Error;
use serde::Serialize;
use simple_error::bail;

#[derive(Debug, Serialize,Clone)]
pub struct AsciiConverter {
    table: Vec<u8>,
}

impl AsciiConverter {
    pub fn new_with_table(table: Vec<u8>) -> Result<AsciiConverter, Box<dyn Error>> {
        if table.len() != 256 {
            bail!("table size needs to be 256 was {}", table.len())
        }
        return Ok(AsciiConverter{
            table
        })
    }

    pub fn new() -> AsciiConverter {
        // TODO: this is somewhat wrong
        let table :Vec<u8> = b"________________[]0123456789____ !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]^__'abcdefghijklmnopqrstuvwxyz{|}~_________________[]0123456789____ !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]^_'abcdefghijklmnopqrstuvwxyz{|}~_".to_vec();
        AsciiConverter{
            table
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
