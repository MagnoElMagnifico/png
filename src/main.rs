mod chunks;
mod crc;
mod filter;
mod png;

use crate::png::Png;
use std::{env, path::Path, process::exit};

fn main() {
    let mut args = env::args();
    let program_name = args.next().expect("Program name not found");

    let png = match args.next() {
        None => {
            println!("USAGE: {} <filepath.png>", program_name);
            exit(1);
        }

        Some(file_param) => match Png::read(Path::new(&file_param)) {
            Err(error) => {
                println!("ERROR: {}", error);
                exit(1);
            }
            Ok(png) => png,
        },
    };

    println!("{:#?}", png.chunks);
}

// These tests have to be checked manually opening the PNG output for the moment.
#[cfg(test)]
mod tests {
    use crate::chunks::{GenericChunk, ImageHeader, ImageTrailer, IDAT};
    use crate::png::Png;
    use std::path::Path;

    #[test]
    fn open_png() {
        todo!();
    }

    #[test]
    fn create_png() {
        let mut png = Png::empty();

        png.chunks
            .push(Box::new(ImageHeader::new((1, 1), 8, 2, false)));
        png.chunks.push(Box::new(GenericChunk::from_bytes(
            IDAT,
            &[
                0, // Filter method
                0, 0, 0, // Pixel
            ],
        )));
        png.chunks.push(Box::new(ImageTrailer));

        png.write(Path::new("generated.png")).unwrap();
    }
}
