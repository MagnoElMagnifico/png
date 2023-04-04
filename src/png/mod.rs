use chunks::Chunk;
use crc::Crc;
use std::{fs, io, mem::size_of, path::Path};

pub mod chunks;
pub mod crc;
pub mod filter;

// Signature
pub const SIGN: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

/// A PNG consists in a signature (that every PNG should have) and a series of chunks, that may be
/// of different types. The order of these last ones do not matter.
///
/// The official spec: http://libpng.org/pub/png/spec/1.2/PNG-Structure.html
pub struct Png {
    // TODO?: Store directly IHDR since every PNG must have one
    pub chunks: Vec<Box<dyn Chunk>>,
    crc: Crc,
}

impl Png {
    pub fn empty() -> Self {
        Self {
            chunks: Vec::with_capacity(3),
            crc: Crc::new(),
        }
    }

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

        let mut png = Self::empty();

        loop {
            // Read chunk data size
            let data_size = u32::from_be_bytes(file_data[p..p + 4].try_into().unwrap()) as usize;
            p += 4;

            // Chunk type and data
            let chunk_data = &file_data[p..p + 4 + data_size];
            let chunk = chunks::from_bytes(chunk_data);
            p += 4 + data_size;

            // CRC checking
            // TODO: make optional
            let calculated_crc = png.crc.calculate(chunk_data);
            let read_crc = u32::from_be_bytes(file_data[p..p + 4].try_into().unwrap());
            p += 4;

            if calculated_crc != read_crc {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "The CRCs do not match: read {}, calculated {}",
                        read_crc, calculated_crc
                    ),
                ));
            }

            png.chunks.push(chunk);

            // TODO: Handle unexpected end of file
            if p >= file_data.len() {
                break;
            }
        }

        Ok(png)
    }

    pub fn write(&self, output_file: &Path) -> io::Result<()> {
        let mut bytes = Vec::with_capacity(
            size_of::<[u8; 8]>()
                + self
                    .chunks
                    .iter()
                    .map(|c| c.data_size() as usize + 4)
                    .sum::<usize>(),
        );
        bytes.extend_from_slice(&SIGN);

        for chunk in &self.chunks {
            bytes.extend_from_slice(&chunk.to_bytes(&self.crc));
        }

        fs::write(output_file, bytes)
    }
}
