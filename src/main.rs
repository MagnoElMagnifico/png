use std::{env, fs, path::Path, process};

const SIGN: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
const IHDR: [u8; 4] = [73, 72, 68, 82];
const IDAT: [u8; 4] = [73, 68, 65, 84];
const IEND: [u8; 4] = [73, 69, 78, 68];

fn print_bytes(data: &[u8]) {
    for byte in data {
        print!("{} ", byte);
    }
    println!();
}

fn print_bytes_as_chars(data: &[u8]) {
    for byte in data {
        print!("{}", *byte as char);
    }
    println!();
}

fn print_bytes_hex(data: &[u8]) {
    for byte in data {
        print!("{:#x} ", byte);
    }
    println!();
}

fn main() {
    let mut args = env::args();
    let program_name = args.next().expect("Program name not found");

    let file_data = match args.next() {
        None => {
            println!("USAGE: {} <filepath.png>", program_name);
            process::exit(1);
        }

        Some(file_param) => match fs::read(Path::new(&file_param)) {
            Err(error) => {
                println!("ERROR: {}", error);
                process::exit(1);
            }
            Ok(file_data) => file_data,
        },
    };

    let mut cursor = 0_usize;

    // Following the official spec: libpng.org/pub/png/spec/1.2/PNG-Structure.html
    //
    // A PNG consists in a signature (that every PNG should have) and a series of chunks, that may
    // be of different types.
    if file_data[cursor..cursor + 8] != SIGN {
        println!("ERROR: the file provided does not appear to be a PNG file (invalid signature)");
        process::exit(1);
    }
    cursor += 8;

    print_bytes(&file_data[..8]);

    // Each chunk has the following structure:
    //  - length of the data section: u32
    //  - chunk type code: u32
    //  - chunk data section
    //  - cyclic redundency check
    // Note that the bytes are stored in Big-Endian

    loop {
        let chunk_length = u32::from_be_bytes(
            file_data[cursor..cursor + 4]
                .try_into()
                .expect("slice of wrong size"),
        );
        cursor += 4;

        let chunk_type = &file_data[cursor..cursor + 4];
        cursor += 4;

        let chunk_data = &file_data[cursor..cursor + chunk_length as usize];
        cursor += chunk_length as usize;

        let chunk_crc = &file_data[cursor..cursor + 4];
        cursor += 4;

        // Then, a whole chunk would be chunk_length + 3*4 bytes

        println!("chunk length: {}", chunk_length);
        print!("chunk type: ");
        print_bytes(chunk_type);
        // print!("chunk data: {:?}", chunk_data);
        print!("chunk crc: ");
        print_bytes_hex(chunk_crc);
        println!();

        if chunk_type == IEND {
            break;
        }
    }
}
