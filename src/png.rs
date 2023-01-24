#![allow(dead_code)]
use std::{fs, io, mem::size_of, path::Path};

// Signature and common ChunkCodes
// TODO: http://libpng.org/pub/png/spec/1.2/PNG-Chunks.html
pub const SIGN: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
pub const IHDR: ChunkCode = ChunkCode([73, 72, 68, 82]);
pub const IDAT: ChunkCode = ChunkCode([73, 68, 65, 84]);
pub const IEND: ChunkCode = ChunkCode([73, 69, 78, 68]);

/// A PNG consists in a signature (that every PNG should have) and a series of chunks, that may be
/// of different types. The order of these last ones do not matter.
///
/// The official spec: http://libpng.org/pub/png/spec/1.2/PNG-Structure.html
#[derive(Debug, Clone)]
pub struct Png {
    pub chunks: Vec<Chunk>,
    crc: Crc,
}

impl Png {
    pub fn read(input_file: &Path) -> io::Result<Self> {
        let file_data = fs::read(input_file)?;

        let mut p = 0_usize;

        if file_data[p..p + 8] != SIGN {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "The given file is not a PNG file",
            ));
        }
        p += 8;

        let mut png = Self {
            chunks: Vec::with_capacity(3),
            crc: Crc::new(), // Compute CRC as well
        };

        let mut should_end = false;
        while !should_end {
            let chunk_size = u32::from_be_bytes(file_data[p..p + 4].try_into().unwrap()) as usize;
            let chunk_bytes = &file_data[p + 4..(p + 4) + 4 + chunk_size + 4];
            let chunk = Chunk::from_bytes(chunk_size, chunk_bytes);

            // TODO: this fails
            // assert_eq!(chunk.crc, png.crc.calculate(chunk_bytes));

            p += chunk_size + 12;

            if p >= file_data.len() {
                should_end = true;
            }

            png.chunks.push(chunk);
        }

        Ok(png)
    }

    pub fn write(&self, output_file: &Path) -> io::Result<()> {
        let mut bytes = Vec::with_capacity(
            size_of::<[u8; 8]>() + self.chunks.iter().map(|c| c.size()).sum::<usize>(),
        );
        bytes.extend_from_slice(&SIGN);

        for chunk in &self.chunks {
            bytes.extend_from_slice(&(chunk.data.len() as u32).to_be_bytes());
            bytes.extend_from_slice(&chunk.chunk_type.0);
            bytes.extend_from_slice(&chunk.data);
            bytes.extend_from_slice(&chunk.crc.to_be_bytes());
        }

        fs::write(output_file, bytes)
    }
}

const CRC_MASK: u32 = 0xEDB88320;
const CRC_TABLE_SZ: usize = 256;

/// A Cyclic redundancy check (CRC) is an error-detecting code. Blocks of data entering these
/// systems get a short check value attached, based on the remainder of a polynomial division of
/// their contents.
///
/// Specification of a CRC code requires definition of a so-called generator polynomial. This
/// polynomial becomes the divisor in a polynomial long division, which takes the message as the
/// dividend and in which the quotient is discarded and the remainder becomes the result. The
/// important caveat is that the polynomial coefficients are calculated according to the arithmetic
/// of a finite field, so the addition operation can always be performed bitwise-parallel (there is
/// no carry between digits). In practice, all commonly used CRCs employ the Galois field, or more
/// simply a finite field, of two elements, GF(2). The two elements are usually called 0 and 1,
/// comfortably matching computer architecture.
///
/// A CRC is called an n-bit CRC when its check value is n bits long. For a given n, multiple CRCs
/// are possible, each with a different polynomial. Such a polynomial has highest degree n, which
/// means it has n + 1 terms. In other words, the polynomial has a length of n + 1; its encoding
/// requires n + 1 bits. Note that most polynomial specifications either drop the MSB or LSB, since
/// they are always 1.
///
/// On the PNG's case, the CRC used is CRC-32, whose polynomial is:
///
/// x^32 + x^26 + x^23 + x^22 + x^16 + x^12 + x^11 + x^10 + x^8 + x^7 + x^5 + x^4 + x^2 + x + 1
///
/// Thus the coefficients are (1 - 32, ignoring 32): 1110 1101 1011 1000 1000 0110 0100 0000
/// which is exactly EBD88320 in hex.
///
/// Source (modified): https://en.wikipedia.org/wiki/Cyclic_redundancy_check
///
/// Translated from the C code avaliable here:
/// http://libpng.org/pub/png/spec/1.2/PNG-CRCAppendix.html

#[derive(Debug, Clone)]
struct Crc([u32; CRC_TABLE_SZ]);

impl Crc {
    pub fn new() -> Self {
        let mut table = [0; CRC_TABLE_SZ];

        for (i, element) in table.iter_mut().enumerate() {
            let mut c = i as u32;
            for _ in 0..8 {
                if (c & 1) == 1 {
                    c = CRC_MASK ^ (c >> 1);
                } else {
                    c >>= 1;
                }
            }

            *element = c;
        }

        Crc(table)
    }

    fn update_crc(&self, crc: u32, buffer: &[u8]) -> u32 {
        let mut c = crc;

        for n in buffer {
            c = self.0[(c as u8 ^ n) as usize] ^ (c >> 8);
        }

        c
    }

    /// Returns the CRC of the bytes on buffer.
    pub fn calculate(&self, buffer: &[u8]) -> u32 {
        // 1's complement of update_crc
        self.update_crc(0xffff_ffff_u32, buffer) ^ 0xffff_ffff_u32
    }
}

/// Each chunk has the following structure:
///
/// - length of the data section: u32
/// - chunk type code: u32
/// - chunk data section
/// - cyclic redundency check
///
/// Note that the bytes (u32) are stored in Big-Endian
#[derive(Default, Debug, Clone)]
pub struct Chunk {
    pub chunk_type: ChunkCode,
    pub data: Vec<u8>,
    pub crc: u32,
}

impl Chunk {
    pub fn new(chunk_type: ChunkCode, data: &[u8]) -> Self {
        Self {
            chunk_type,
            data: data.to_owned(),
            crc: 0,
        }
    }

    pub fn from_bytes(size: usize, data: &[u8]) -> Self {
        assert_eq!(size + 8, data.len());

        Self {
            chunk_type: ChunkCode::from_slice(&data[..4]).unwrap(),
            data: data[4..4 + size].to_owned(),
            crc: u32::from_be_bytes(data[size + 4..].try_into().unwrap()),
        }
    }

    pub fn size(&self) -> usize {
        self.data.len() + size_of::<ChunkCode>() + 2 * size_of::<u32>()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.size());
        bytes.extend_from_slice(&(self.data.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&self.chunk_type.0);
        bytes.extend_from_slice(&self.data);
        bytes
    }
}

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
#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct ChunkCode([u8; 4]);

impl ChunkCode {
    pub fn from_code(code: &str) -> Self {
        let mut chars = code.chars();

        ChunkCode([
            chars.next().unwrap() as u8,
            chars.next().unwrap() as u8,
            chars.next().unwrap() as u8,
            chars.next().unwrap() as u8,
        ])
    }

    pub fn from_slice(data: &[u8]) -> Result<Self, std::array::TryFromSliceError> {
        let bytes = data.try_into()?;
        Ok(Self(bytes))
    }

    pub fn get_code(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.0)
    }
}
