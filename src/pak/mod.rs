use std::io::Cursor;
use std::io::Read;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::num::TryFromIntError;

use serde::Serialize;
use thiserror::Error;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};




#[derive(Error, Debug)]
pub enum PakError {
    #[error("read error")]
    ParseError,
    #[error("header mismath: expected {0}, got {1}")]
    HeaderError(u32, u32),
    #[error("io error {0}")]
    IoError(std::io::Error),
    #[error("utf8 error {0}")]
    UtfConversionError(std::string::FromUtf8Error),
    #[error("from int error {0}")]
    IntConversionError(std::num::TryFromIntError),
    #[error("supplied file name is longer than {0} >= {1}")]
    NameLengthError(usize, usize),
}

impl From<std::io::Error> for PakError {
    fn from(err: std::io::Error) -> PakError {
        PakError::IoError(err)
    }
}

impl From<std::string::FromUtf8Error> for PakError {
    fn from(err: std::string::FromUtf8Error) -> PakError {
        PakError::UtfConversionError(err)
    }
}

impl From<std::num::TryFromIntError> for PakError {
    fn from(err: std::num::TryFromIntError) -> PakError {
        PakError::IntConversionError(err)
    }
}

static HEADER: u32 = 0x4b434150;
const MAX_NAME_LENGTH: usize = 55;
const NAME_LENGTH: u32 = 56;



#[derive(Serialize, Debug)]
pub struct Pak {
    pub data: Vec<u8>,
    pub files: Vec<PakFile>,
}

#[derive(Serialize, Debug)]
pub struct PakFile {
    pub name: Vec<u8>,
    pub position: u32,
    pub size: u32,
}

impl Pak {
    pub fn parse(mut reader: impl Read) -> Result<Pak, PakError> {
        let mut data = Vec::new();
        match reader.read_to_end(&mut data) {
            Ok(size) => size,
            Err(err) => {
                return Err(PakError::IoError(err))
            }
        };
        let mut cursor = Cursor::new(&data);

        let header = cursor.read_u32::<LittleEndian>().unwrap();
        if header != HEADER {
            return Err(PakError::HeaderError(HEADER, header))
        }
        let dir_offset = cursor.read_u32::<LittleEndian>().unwrap();
        let dir_length = cursor.read_u32::<LittleEndian>().unwrap();
        let file_count = dir_length / (NAME_LENGTH + 4 * 2);

        cursor.seek(SeekFrom::Start((dir_offset).into()))?;
        let mut files = Vec::new();
        for _ in 0..file_count { 
            let pos = cursor.position();
            let mut s_buf = Vec::new();
            let mut b = cursor.read_u8()?;
            while b != 0 {
                    s_buf.push(b);
                    b = cursor.read_u8()?;
            }
            cursor.seek(SeekFrom::Start(pos + NAME_LENGTH as u64))?;
            files.push(PakFile{
                name: s_buf,
                position: cursor.read_u32::<LittleEndian>()?,
                size: cursor.read_u32::<LittleEndian>()?,
            });
        }
        return Ok(Pak{
            data,
            files
        })
    }

    pub fn get_data(&self, file: &PakFile) -> Result<Vec<u8>, PakError> {

        let mut cursor = Cursor::new(&self.data);
        let size: usize = file.size.try_into()?;
        let mut buf = vec![0; size];
        cursor.seek(SeekFrom::Start(file.position as u64))?;
        cursor.read_exact(&mut buf)?;
        return Ok(buf)
    }
}

#[derive(Serialize, Debug)]
pub struct PakWriter {
    files: Vec<PakWriterFile>,
}

#[derive(Serialize, Debug)]
struct PakWriterFile {
    name: Vec<u8>,
    data: Vec<u8>,
}

impl PakWriter {
    pub fn new() -> PakWriter {
        return PakWriter{
            files: Vec::new(),
        };
    }

    pub fn file_add(&mut self, name: Vec<u8>, mut data: impl Read) -> Result<(), PakError> {
        if name.len() > MAX_NAME_LENGTH {
            return Err(PakError::NameLengthError(name.len(), MAX_NAME_LENGTH))
        }
        let mut file_data = Vec::new();

        data.read_to_end(&mut file_data)?;
        self.files.push(PakWriterFile{
            name,
            data: file_data,
        });
        return Ok(())
    }

    pub fn write_data(self) -> Result<Vec<u8>, PakError> {
        let mut buffer: Vec<u8> = Vec::new();
        let mut c = Cursor::new(&mut buffer);
        c.write(&HEADER.to_le_bytes())?;
        let dir_offset : u32 = 4 * 3;
        c.write(&dir_offset.to_le_bytes())?;
        let dir_size: u32 = (self.files.len() * (MAX_NAME_LENGTH +1 + 8)) as u32 ;
        c.write(&dir_size.to_le_bytes())?;
        let mut file_position = dir_offset + dir_size;
        for file in &self.files {
            let mut name_buffer: [u8;NAME_LENGTH as usize] = [0;NAME_LENGTH as usize];
            name_buffer[..file.name.len()].copy_from_slice(&file.name);
            c.write(&name_buffer)?;
            c.write(&file_position.to_le_bytes())?;
            c.write(&(file.data.len() as u32).to_le_bytes())?;
            file_position += file.data.len() as u32;
        }
        for file in &self.files {
            c.write(&file.data)?;
        }
        return Ok(buffer)
    }
}


#[cfg(test)]
mod tests {
    #[test]
    pub fn pak_creation_and_reading() -> Result<(), crate::pak::PakError> {
        const FILE1_NAME: &[u8; 9] = b"dir/file1";
        const FILE1_DATA: &[u8; 8] = b"01234567";
        const FILE2_NAME: &[u8; 11] = b"dir_a/file2";
        const FILE2_DATA: &[u8; 8] = b"76543210";
        let mut pack = crate::pak::PakWriter::new();
        pack.file_add(FILE1_NAME.to_vec(), &FILE1_DATA[..])?;
        pack.file_add(FILE2_NAME.to_vec(), &FILE2_DATA[..])?;
        assert_eq!(pack.files[0].name, FILE1_NAME.to_vec());
        assert_eq!(pack.files[0].data, FILE1_DATA.to_vec());
        assert_eq!(pack.files[1].name, FILE2_NAME.to_vec());
        assert_eq!(pack.files[1].data, FILE2_DATA.to_vec());
        let data = pack.write_data()?;
        let read_pack = crate::pak::Pak::parse(&data[..])?;
        assert_eq!(2, read_pack.files.len());
        // names
        assert_eq!(FILE1_NAME.to_vec(), read_pack.files[0].name);
        assert_eq!(FILE2_NAME.to_vec(), read_pack.files[1].name);
        // data size
        assert_eq!(FILE1_DATA.to_vec().len() as u32, read_pack.files[0].size);
        assert_eq!(FILE2_DATA.to_vec().len() as u32, read_pack.files[1].size);
        // data
        assert_eq!(FILE1_DATA.to_vec(), read_pack.get_data(&read_pack.files[0])?);
        assert_eq!(FILE2_DATA.to_vec(), read_pack.get_data(&read_pack.files[1])?);
        return Ok(())
    }
}
