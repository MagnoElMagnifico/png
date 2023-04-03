use png::{Png, IDAT};
use std::{env::args, path::Path};

// Create a simple PNG image
fn main() {
    let file = args().nth(1).unwrap();
    let png = Png::read(Path::new(&file)).unwrap();

    for idat in png.chunks.iter().filter(|x| x.get_type() == IDAT) {
        println!("{:?}", idat);
    }
}
