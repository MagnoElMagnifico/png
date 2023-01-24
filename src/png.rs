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
            crc: Crc::new(),
        };

        let mut should_end = false;
        while !should_end {
            let chunk_size = u32::from_be_bytes(file_data[p..p + 4].try_into().unwrap()) as usize;
            let chunk_bytes = &file_data[p + 4..(p + 4) + 4 + chunk_size + 4];
            let chunk = Chunk::from_bytes(chunk_size, chunk_bytes);

            let calculated_crc = png.crc.calculate(&chunk_bytes[..chunk_bytes.len() - 4]);
            if calculated_crc == chunk.crc {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "The CRCs do not match: read {}, calculated {}",
                        chunk.crc, calculated_crc
                    ),
                ));
            }

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
        assert_eq!(
            size + 8,
            data.len(),
            "The data length should be {}, got {}",
            size + 8,
            data.len()
        );

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

        ChunkCode(chunk_code)
    }

    pub fn from_slice(data: &[u8]) -> Result<Self, std::array::TryFromSliceError> {
        let bytes = data.try_into()?;
        Ok(Self(bytes))
    }

    pub fn get_code(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.0)
    }
}

const CRC_MASK: u32 = 0xEDB88320;
const CRC_TABLE_SZ: usize = u8::MAX as usize + 1;

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
/// # Example
///
/// In this example, we shall encode 14 bits of message with a 3-bit CRC, with a polynomial `x^3 + x
/// + 1` (coefficients 1011). Start with the message to be encoded: `11 0100 1110 1100` and execute
/// a bitwise XOR:
///
/// ```
/// 11010011101100 000 <--- input right padded by 3 bits
/// 1011               <--- divisor
/// 01100011101100 000 <--- result (note the first four bits are the XOR with the divisor beneath, the rest of the bits are unchanged)
///  1011              <--- divisor ...
/// 00111011101100 000
///   1011
/// 00010111101100 000
///    1011
/// 00000001101100 000 <--- note that the divisor moves over to align with the next 1 in the dividend (since quotient for that step was zero)
///        1011             (in other words, it doesn't necessarily move one bit per iteration)
/// 00000000110100 000
///         1011
/// 00000000011000 000
///          1011
/// 00000000001110 000
///           1011
/// 00000000000101 000
///            101 1
/// -----------------
/// 00000000000000 100 <--- remainder (3 bits).  Division algorithm stops here as dividend is equal to zero.
/// ```
///
/// Now, to check the validity of the message, the operation will be repeated with the remainder
/// instead of zeroes. It should equal zero if there are no detectable errors.
///
/// ```
/// 11010011101100 100 <--- input with check value
/// 1011               <--- divisor
/// 01100011101100 100 <--- result
///   ...
/// 00000000000101 100
///            101 1
/// ------------------
/// 00000000000000 000 <--- remainder
/// ```
///
/// A practical algorithm for the CRC-32 variant of CRC is the CRCTable, which is a memoization
/// (storage of all the possibilities -- 256) of a calculation that would have to be repeated for
/// each byte of the message.
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

        for (i, table_byte) in table.iter_mut().enumerate() {
            let mut byte = i as u32;
            for _ in 0..8 {
                if (byte & 1) == 1 {
                    byte = CRC_MASK ^ (byte >> 1);
                } else {
                    byte >>= 1;
                }
            }

            *table_byte = byte;
        }

        Crc(table)
    }

    /// Returns the CRC of the bytes on buffer.
    pub fn calculate(&self, buffer: &[u8]) -> u32 {
        let mut crc = 0xFFFF_FFFF_u32;

        for byte in buffer {
            let index = crc as u8 ^ byte;
            crc = (crc >> 8) ^ self.0[index as usize];
        }

        // Invert the bits (1's complement)
        crc ^ 0xFFFF_FFFF_u32
    }
}
