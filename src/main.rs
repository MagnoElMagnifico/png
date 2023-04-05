use png::{Oscilator, Wav, WaveForm};
use png::{Png, IDAT};
use std::{env::args, path::Path};

fn main() {
    let file_type = args().nth(1).unwrap();
    let file_name = args().nth(2).unwrap();

    match &file_type[..] {
        "png" => {
            let png = Png::read(Path::new(&file_name)).unwrap();
            for idat in png.chunks.iter().filter(|x| x.get_type() == IDAT) {
                println!("{:?}", idat);
            }
        }

        "wav" => {
            let wav = Wav::read(Path::new(&file_name)).unwrap();
            println!("{:?}", wav.data);
        }

        "sin-wav" => Oscilator::new(44100, 220.0, WaveForm::Sin, 64, 128)
            .to_wav(5000)
            .write(Path::new(&file_name))
            .unwrap(),
        "saw-wav" => Oscilator::new(44100, 220.0, WaveForm::Saw, 64, 128)
            .to_wav(5000)
            .write(Path::new(&file_name))
            .unwrap(),
        "sqr-wav" => Oscilator::new(44100, 220.0, WaveForm::Sqr(0.5), 64, 128)
            .to_wav(5000)
            .write(Path::new(&file_name))
            .unwrap(),
        _ => println!("Unknown option: {}", file_type),
    }
}
