#![allow(dead_code)]

//! Each chunk has the following structure:
//!
//! - length of the data section: u32
//! - chunk type code: u32
//! - chunk data section
//! - cyclic redundency check
//!
//! Note that the bytes (u32) are stored in Big-Endian

use std::mem::size_of;

use crate::crc::Crc;

/// The ChunkCode consists in four bytes whose values are between 65-90 and 97-122 decimal, so
/// uppercase and lowercase ASCII letters. However they should be always treated as integers and not
/// chars.
///
/// The 5th bit of a ASCII char determines if it is uppercase (0) or lowercase (1).
///
/// - 1st byte: 0: critical, 1: optional
/// - 2nd byte: 0: public special-purpose code, 1: private unregistered code
/// - 3rd byte: 0: using current version of PNG
/// - 4th byte: 0: not safe to copy, 1: save to copy (related to PNG
/// editors and they should handle unrecognized chunks: if it is unsafe to copy, it means the
/// chunk is dependent on the image data, and if the image was modified, it it no longer valid)
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ChunkType([u8; 4]);

pub const IHDR: ChunkType = ChunkType([73, 72, 68, 82]);
pub const IDAT: ChunkType = ChunkType([73, 68, 65, 84]);
pub const IEND: ChunkType = ChunkType([73, 69, 78, 68]);

impl ChunkType {
    pub fn from_code(code: &str) -> Self {
        // TODO: Return error instead
        assert_eq!(
            4,
            code.len(),
            "The code length should be 4, got {}",
            code.len()
        );

        let mut chunk_code = [0; 4];

        for (i, char) in code.chars().enumerate() {
            chunk_code[i] = char as u8;
        }

        Self(chunk_code)
    }

    pub fn from_slice(data: &[u8]) -> Result<Self, std::array::TryFromSliceError> {
        let bytes = data.try_into()?;
        Ok(Self(bytes))
    }

    pub fn get_char_code(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.0)
    }

    pub fn is_critical(&self) -> bool {
        self.0[0] & (1 << 5) == 0
    }

    pub fn is_public(&self) -> bool {
        self.0[2] & (1 << 5) == 0
    }

    pub fn is_safe_to_copy(&self) -> bool {
        self.0[3] & (1 << 5) == 1
    }
}

////////////////////////////////////////////////////////////////////////////////

pub trait Chunk: std::fmt::Debug {
    /// Returns the size of the data section (not including type)
    fn data_size(&self) -> u32;
    fn get_type(&self) -> ChunkType;
    fn data_to_bytes(&self) -> Vec<u8>;

    fn to_bytes(&self, crc: &Crc) -> Vec<u8> {
        let data_size = self.data_size();

        let mut bytes =
            Vec::with_capacity(size_of::<ChunkType>() + data_size as usize + 2 * size_of::<u32>());
        bytes.extend_from_slice(&data_size.to_be_bytes());
        bytes.extend_from_slice(&self.get_type().0);
        bytes.extend_from_slice(&self.data_to_bytes());

        // CRC calculation
        let crc = crc.calculate(&bytes[4..]); // Jump size
        bytes.extend_from_slice(&crc.to_be_bytes());

        bytes
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug, Clone)]
pub struct GenericChunk {
    pub chunk_type: ChunkType,
    pub data: Vec<u8>,
}

impl GenericChunk {
    pub fn new(chunk_type: ChunkType, data: &[u8]) -> Self {
        Self {
            chunk_type,
            data: data.to_owned(),
        }
    }
}

impl Chunk for GenericChunk {
    fn get_type(&self) -> ChunkType {
        self.chunk_type
    }

    fn data_size(&self) -> u32 {
        self.data.len() as u32
    }

    fn data_to_bytes(&self) -> Vec<u8> {
        self.data.clone()
    }
}

////////////////////////////////////////////////////////////////////////////////

/// IHDR Chunk must appear first:
///
///  - Width (px):         4 bytes
///  - Height (px):        4 bytes
///  - Bit depth:          1 byte
///  - Color type:         1 byte
///  - Compression method: 1 byte
///  - Filter method:      1 byte
///  - Interlace method:   1 byte
#[derive(Debug, Copy, Clone, Default)]
pub struct ImageHeader {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: u8,
    pub compression: u8,
    pub filter: u8,
    pub interlace: u8,
}

impl ImageHeader {
    pub fn new(data: &[u8]) -> Self {
        Self {
            width: u32::from_be_bytes(data[0..4].try_into().unwrap()),
            height: u32::from_be_bytes(data[4..8].try_into().unwrap()),
            bit_depth: data[8],
            color_type: data[9],
            compression: data[10],
            filter: data[11],
            interlace: data[12],
        }
    }
}

impl Chunk for ImageHeader {
    fn data_size(&self) -> u32 {
        size_of::<ImageHeader>() as u32
    }

    fn get_type(&self) -> ChunkType {
        IHDR
    }

    fn data_to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.data_size() as usize);
        bytes.extend_from_slice(&self.width.to_be_bytes());
        bytes.extend_from_slice(&self.height.to_be_bytes());
        bytes.push(self.bit_depth);
        bytes.push(self.compression);
        bytes.push(self.filter);
        bytes.push(self.interlace);
        bytes
    }
}
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone)]
struct ImageTrailer;

impl Chunk for ImageTrailer {
    fn data_size(&self) -> u32 {
        0
    }

    fn get_type(&self) -> ChunkType {
        IEND
    }

    fn data_to_bytes(&self) -> Vec<u8> {
        vec![ 0 ]
    }

}

////////////////////////////////////////////////////////////////////////////////

/// This function returns the most apropiated Chunk for the data read.
/// The first 4 bytes are considered as the type and the rest are data.
pub fn from_bytes(bytes: &[u8]) -> Box<dyn Chunk> {
    match ChunkType::from_slice(&bytes[..4]) {
        Ok(IHDR) => Box::new(ImageHeader::new(&bytes[4..])),
        Ok(IEND) => Box::new(ImageTrailer {}),
        Ok(other) => Box::new(GenericChunk::new(other, &bytes[4..])),
        Err(error) => unreachable!("{}", error),
    }
}
