//! Each chunk has the following structure:
//!
//! - length of the data section: u32
//! - chunk type code: u32
//! - chunk data section
//! - cyclic redundency check: u32
//!
//! Note that the bytes (u32) are stored in Big-Endian

use super::crc::Crc;
use std::mem::size_of;

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
        // TODO: clippy: incompatible bit mask: `_ & 32` can never be equal to `1`
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

        let mut bytes = Vec::with_capacity(data_size as usize + 3 * size_of::<u32>());
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
    pub fn from_bytes(chunk_type: ChunkType, data: &[u8]) -> Self {
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
///  - Width (4 bytes) and Height (4 bytes) store the size of the image in pixels. Valid range is
///    0..=2^31.
///
///  - Bit Depth (1 byte) : is the number of bits per sample or per palette index (not per pixel). Valid values
///    are 1, 2, 4, 8, and 16, although not all values are allowed for all color types.
///
///  - Color type (1 byte): represent sums of the following values:
///        - 0: Grayscale used
///        - 1: Palette used         (1st bit set)
///        - 2: Color used           (2nd bit set)
///        - 4: Alpha channel used   (3rd bit set)
///    Valid values are 0, 2, 3, 4, and 6.
///
///  - Compression method (1 byte): indicates the method used to compress the image data. At
///    present, only compression method 0 (deflate/inflate compression with a sliding window of at
///    most 32768 bytes) is defined. All standard PNG images must be compressed with this scheme.
///
///  - Filter method (1 byte): indicates the preprocessing method applied to the image data before
///    compression. At present, only filter method 0 (adaptive filtering with five basic filter
///    types) is defined.
///
///  - Interlace method (1 byte): indicates the transmission order of the image data. Two values
///    are currently defined: 0 (no interlace) or 1 (Adam7 interlace).
///
/// Bit depth restrictions for each color type are imposed to simplify implementations and to
/// prohibit combinations that do not compress well:
///
/// | PNG image type        | Color type | Allowed bit depths | Interpretation                                                  |
/// |:----------------------|:-----------|:-------------------|:----------------------------------------------------------------|
/// | Greyscale             | 0          | 1, 2, 4, 8, 16     | Each pixel is a greyscale sample                                |
/// | Truecolour            | 2          | 8, 16              | Each pixel is an R,G,B triple                                   |
/// | Indexed-colour        | 3          | 1, 2, 4, 8         | Each pixel is a palette index; a PLTE chunk shall appear.       |
/// | Greyscale with alpha  | 4          | 8, 16              | Each pixel is a greyscale sample followed by an alpha sample.   |
/// | Truecolour with alpha | 6          | 8, 16              | Each pixel is an R,G,B triple followed by an alpha sample.      |
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
    pub fn new(size: (u32, u32), bit_depth: u8, color_type: u8, adam7_interlace: bool) -> Self {
        // TODO: check for valid combinations of bit_depth and color_type
        Self {
            width: size.0,
            height: size.1,
            bit_depth,
            color_type,
            compression: 0,
            filter: 0,
            interlace: u8::from(adam7_interlace),
        }
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        assert_eq!(
            data.len(),
            13,
            "ImageHeader must be 13 bytes long, got {}",
            data.len()
        );
        // TODO: check for valid combinations of bit_depth and color_type
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
        13
    }

    fn get_type(&self) -> ChunkType {
        IHDR
    }

    fn data_to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(13);
        bytes.extend_from_slice(&self.width.to_be_bytes());
        bytes.extend_from_slice(&self.height.to_be_bytes());
        bytes.push(self.bit_depth);
        bytes.push(self.color_type);
        bytes.push(self.compression);
        bytes.push(self.filter);
        bytes.push(self.interlace);
        bytes
    }
}

////////////////////////////////////////////////////////////////////////////////

/// IEND describes the end of the PNG. It must be empty.
#[derive(Debug, Copy, Clone)]
pub struct ImageTrailer;

impl Chunk for ImageTrailer {
    fn data_size(&self) -> u32 {
        0
    }

    fn get_type(&self) -> ChunkType {
        IEND
    }

    fn data_to_bytes(&self) -> Vec<u8> {
        vec![]
    }
}

////////////////////////////////////////////////////////////////////////////////

/// This function returns the most apropiated Chunk for the data read.
/// The first 4 bytes are considered as the type and the rest are data.
pub fn from_bytes(bytes: &[u8]) -> Box<dyn Chunk> {
    match ChunkType::from_slice(&bytes[..4]) {
        Ok(IHDR) => Box::new(ImageHeader::from_bytes(&bytes[4..])),
        Ok(IEND) => Box::new(ImageTrailer {}),
        Ok(other) => Box::new(GenericChunk::from_bytes(other, &bytes[4..])),
        Err(error) => unreachable!("{}", error),
    }
}

// TODO: http://libpng.org/pub/png/spec/1.2/PNG-Chunks.html
