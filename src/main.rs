use std::{env, fs, path::Path, process};

fn main() {
    let mut args = env::args();
    let program_name = args.next().expect("Program name not found");

    let _ = match args.next() {
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
        }
    };
}
