use paste::paste;
/// Common structs shared between different formats
use serde::Serialize;

use std::ops::Index;

use protocol_macros::DataTypeRead;

use crate::datatypes::reader::{
    DataTypeBoundCheck, DataTypeRead, DataTypeReader, DataTypeReaderError,
};

#[cfg(feature = "trace")]
use crate::trace::{trace_annotate, trace_start, trace_stop};

use super::bsp;
use super::mdl;

/// A trait to get an ascii string
pub trait AsciiString {
    fn ascii_string(&self) -> String;
}

/// A vector or position
#[derive(Serialize, Clone, Debug, Copy, DataTypeRead, Default)]
pub struct Vector3<T: DataTypeRead + 'static>
where
    T: Clone,
{
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Index<usize> for Vector3<T>
where
    T: DataTypeRead + Clone,
{
    type Output = T;
    fn index(&self, index: usize) -> &T {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            n => panic!("Invalid Vector3d index: {}", n),
        }
    }
}
// impl <T> DataTypeRead for Vector3<T> {
//     fn to_datatype(&self) -> DataType {
//         DataType::VECTOR3
//     }
// }

impl AsciiString for Vec<u8> {
    fn ascii_string(&self) -> String {
        let conv = String::from_utf8_lossy(self);
        conv.chars().filter(|&c| c != '\0').collect()
    }
}

/// Bounding box
#[derive(Serialize, Clone, Debug, Copy, Default, DataTypeRead)]
#[datatyperead(types("Vertex", "u8"))]
pub struct BoundingBox<T: DataTypeRead + 'static>
where
    T: Clone,
{
    pub min: T,
    pub max: T,
}

#[derive(Debug, Serialize, Clone)]
pub enum BoundingBoxValue<T> {
    BoundingBox(T),
}

#[derive(Serialize, Clone, Debug, Default)]
pub enum DataType {
    #[default]
    None,
    U8(u8),
    U16(u16),
    U32(u32),
    I8(i8),
    I16(i16),
    I32(i32),
    F32(f32),
    PAK(crate::pak::Pak),
    PAKHEADER(crate::datatypes::pak::Header),
    PAKFILE(crate::datatypes::pak::File),
    BSPHEADER(bsp::Header),
    DIRECTORYENTRY(DirectoryEntry),
    MDLSKIN(mdl::Skin),
    MDLFRAME(mdl::Frame),
    MDLHEADER(mdl::Header),
    GENERICSTRING(String),
    GENERICVECTOR(usize),
    TRIANGLE(Triangle),
    TEXTURECOORDINATE(TextureCoordinate),
    VERTEX(Vertex),
    VECTOR3GENERIC,
    BOUNDINGBOXGENERIC,
    BOUNDINGBOXU8(BoundingBox<u8>),
    BOUNDINGBOXVERTEX(BoundingBox<Vertex>),
}

impl DataType {
    #[allow(unused)]
    fn to_datatype(&self) -> DataType {
        self.clone()
    }
}

/// Directory entry: describes the position and size of a chunk of data inside a BSP File
#[derive(Serialize, Clone, Debug, Copy, DataTypeRead)]
pub struct DirectoryEntry {
    /// Offset from the sart of the file
    pub offset: u32,
    /// Size of the chunk
    pub size: u32,
}

impl DataTypeBoundCheck for DirectoryEntry {
    fn check_bounds(&self, datatypereader: &mut DataTypeReader) -> Result<(), DataTypeReaderError> {
        let size = datatypereader.data.len() as u32;
        if self.offset + self.size > size {
            return Err(DataTypeReaderError::BoundCheckError(
                self.offset.into(),
                self.size.into(),
                (self.offset + self.size - size).into(),
                size.into(),
            ));
        }
        Ok(())
    }
}

impl<T: DataTypeBoundCheck> DataTypeBoundCheck for Vec<T> {
    fn check_bounds(&self, datatypereader: &mut DataTypeReader) -> Result<(), DataTypeReaderError> {
        for e in self {
            e.check_bounds(datatypereader)?
        }
        Ok(())
    }
}

#[derive(Serialize, Debug, Default, Clone, DataTypeRead)]
pub struct TextureCoordinate {
    pub onseam: i32,
    pub s: i32,
    pub t: i32,
}

#[derive(Serialize, Clone, Debug, Copy, DataTypeRead, Default)]
pub struct Triangle {
    pub faces_front: u32,
    pub vertex: Vector3<i32>,
}

#[derive(Serialize, Clone, Debug, Copy, DataTypeRead, Default)]
pub struct Vertex {
    pub v: Vector3<u8>,
    pub normal_index: u8,
}
