mod chunks;
mod crc;
mod filter;
mod png;

use std::{env, path::Path, process::exit};
use crate::png::Png;

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
    use std::path::Path;
    use crate::chunks::{IDAT, ImageTrailer, GenericChunk, ImageHeader};
    use crate::png::Png;

    #[test]
    fn open_png() {
        todo!();
    }

    #[test]
    fn create_png() {
        let mut png = Png::empty();

        png.chunks.push(Box::new(ImageHeader::new((1, 1), 8, 2, false)));
        png.chunks.push(Box::new(GenericChunk::from_bytes(IDAT, &[
                                                          0, 0, 0
        ])));
        png.chunks.push(Box::new(ImageTrailer));

        png.write(Path::new("generated.png"));
    }
}
