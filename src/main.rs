use std::{env, path::Path, process};

mod chunks;
mod crc;
mod png;

use crate::chunks::{ChunkType, GenericChunk};
use crate::png::Png;

fn main() {
    let mut args = env::args();
    let program_name = args.next().expect("Program name not found");

    let mut png = match args.next() {
        None => {
            println!("USAGE: {} <filepath.png>", program_name);
            process::exit(1);
        }

        Some(file_param) => match Png::read(Path::new(&file_param)) {
            Err(error) => {
                println!("ERROR: {}", error);
                process::exit(1);
            }
            Ok(png) => png,
        },
    };

    png.chunks.insert(
        1,
        Box::new(GenericChunk::new(
            ChunkType::from_code("teSt"),
            "*secret code here*".as_bytes(),
        )),
    );
    png.write(Path::new("./output.png")).unwrap();
}
