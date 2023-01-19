#![allow(dead_code)]

use std::{path::Path, io, fs, mem::size_of};

pub const SIGN: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
pub const IHDR: ChunkType = ChunkType([73, 72, 68, 82]);
pub const IDAT: ChunkType = ChunkType([73, 68, 65, 84]);
pub const IEND: ChunkType = ChunkType([73, 69, 78, 68]);

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct ChunkType([u8; 4]);

impl ChunkType {
    pub fn from_slice(data: &[u8]) -> Result<Self, std::array::TryFromSliceError> {
        let bytes = data.try_into()?;
        Ok(Self(bytes))
    }

    pub fn get_code<'a>(&'a self) -> Result<&'a str, std::str::Utf8Error> {
        std::str::from_utf8(&self.0)
    }
}



#[derive(Default, Debug, Clone)]
pub struct Chunk {
    pub chunk_type: ChunkType,
    pub data: Vec<u8>,
    pub crc: u32,
}

impl Chunk {
    pub fn new(size: usize, data: &[u8]) -> Self {
        assert_eq!(size + 8, data.len());

        Self {
            chunk_type: ChunkType::from_slice(&data[..4]).unwrap(),
            data: data[4 .. 4+size].to_owned(),
            crc: u32::from_be_bytes(data[size+4 ..].try_into().unwrap()),
        }
    }

    pub fn check_crc(&self) -> bool {
        todo!()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(8 + self.data.len());
        bytes.extend_from_slice(&self.chunk_type.0);
        bytes.extend_from_slice(&self.data);
        bytes.extend_from_slice(&self.crc.to_be_bytes());
        bytes
    }

    pub fn size(&self) -> usize {
        self.data.len() + size_of::<ChunkType>() + size_of::<u32>()
    }
}


#[derive(Default, Debug, Clone)]
pub struct Png {
    pub chunks: Vec<Chunk>,
}

impl Png {
    pub fn read(input_file: &Path) -> io::Result<Self> {
        let file_data = fs::read(input_file)?;

        let mut p = 0_usize;

        // Following the official spec: http://libpng.org/pub/png/spec/1.2/PNG-Structure.html
        //
        // A PNG consists in a signature (that every PNG should have) and a series of chunks, that may
        // be of different types.
        if file_data[p .. p+8] != SIGN {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "The given file is not a PNG file",
            ));
        }
        p += 8;

        let mut png = Self {
            chunks: Vec::with_capacity(3),
        };

        // Each chunk has the following structure:
        //  - length of the data section: u32
        //  - chunk type code: u32
        //  - chunk data section
        //  - cyclic redundency check
        // Note that the bytes are stored in Big-Endian
        let mut should_end = false;
        while !should_end {
            let chunk_size = u32::from_be_bytes(file_data[p .. p+4].try_into().unwrap()) as usize;
            let chunk = Chunk::new(chunk_size, &file_data[p+4 .. (p+4)+4+chunk_size+4]);
            p += chunk_size + 12;

            if chunk.chunk_type == IEND {
                should_end = true;
            }

            png.chunks.push(chunk);
        }

        Ok(png)
    }

    pub fn write(&self, output_file: &Path) -> io::Result<()> {
        let mut bytes = Vec::with_capacity(self.chunks.iter().map(|c| c.size()).sum());

        for chunk in &self.chunks {
            bytes.extend_from_slice(&chunk.chunk_type.0);
            bytes.extend_from_slice(&chunk.data);
            bytes.extend_from_slice(&chunk.crc.to_be_bytes());
        }

        fs::write(output_file, bytes)
    }
}
