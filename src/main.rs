use png::{wav::WavSamples, Wav};
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

        "write-wav" => {
            Wav::from_data(create_wav(44100), 44100).write(Path::new(&file_name)).expect("error writing WAV file");
        }

        _ => println!("Unknown option: {}", file_type),
    }
}

fn create_wav(sample_rate: u32) -> WavSamples {
    let seconds = 3;
    let volume = 128;

    let mut data = vec![0_u8; seconds * sample_rate as usize];

    for (i, sample) in data.iter_mut().enumerate() {
        *sample = volume * ( i % 100 == 0 ) as u8;
    }

    WavSamples::Mono8(data)
}
