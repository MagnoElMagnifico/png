mod chunks;
mod crc;
mod filter;
mod png;

#[allow(unused_imports)]
use crate::chunks::{GenericChunk, IDAT, ImageHeader, ImageTrailer};
use crate::png::Png;
use std::{path::Path, fs::File, io::Write};

#[allow(dead_code)]
const SIZE: usize = 500;

// Create a simple PNG image
fn main() {
    let png = Png::read(Path::new("assets/pnglogo.png")).unwrap();

    for (i, idat) in png.chunks.iter().filter(|x| x.get_type() == IDAT).enumerate() {
        println!("{:?}", idat);
        File::create(format!("idat{i}.dat")).unwrap().write_all(&idat.data_to_bytes()).unwrap();
    }

    // let mut png = Png::empty();
    //
    // let image_data = [0xAA; 3*SIZE*SIZE + SIZE];
    //
    // // Greyscale, 1 byte per sample (8 bits), no interlacing
    // png.chunks.push(Box::new(ImageHeader::new((SIZE as u32, SIZE as u32), 8, 2, false)));
    // png.chunks.push(Box::new(GenericChunk::from_bytes(IDAT, &image_data)));
    // png.chunks.push(Box::new(ImageTrailer));
    //
    // png.write(Path::new("test_output.png")).unwrap();
}

/*
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
*/

// These tests have to be checked manually opening the PNG output for the moment.
/* #[cfg(test)]
mod tests {
    use crate::chunks::{GenericChunk, ImageHeader, ImageTrailer, IDAT};
    use crate::png::Png;
    use std::path::Path;

    #[test]
    fn create_png() {
        let mut png = Png::empty();

        png.chunks
            .push(Box::new(ImageHeader::new((100, 100), 8, 2, false)));
        png.chunks.push(Box::new(GenericChunk::from_bytes(
            IDAT,
            &[0; 100 + 100 * 100],
        )));
        png.chunks.push(Box::new(ImageTrailer));

        png.write(Path::new("generated.png")).unwrap();
    }

    #[test]
    // cargo test read_png -- --show-output
    fn read_png() {
        let path = Path::new("/home/magno/Prog/png/assets/good_normal_one-black-pixel.png");
        let png = Png::read(path).unwrap();
        println!("{:#?}", png.chunks);
    }
} */
