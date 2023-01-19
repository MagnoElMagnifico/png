use std::{env, path::Path, process};

mod png;
use png::Png;

fn main() {
    let mut args = env::args();
    let program_name = args.next().expect("Program name not found");

    let png = match args.next() {
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

    for (i,chunk) in png.chunks.into_iter().enumerate() {
        println!("{} {}", i, chunk.chunk_type.get_code().unwrap());
    }
}
