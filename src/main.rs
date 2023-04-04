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

        "sin-wav" => create_sinwav(&file_name),
        "sqr-wav" => create_sqrwav(&file_name),
        _ => println!("Unknown option: {}", file_type),
    }
}

use std::f32::consts::PI;

fn create_sinwav(file_name: &str) {
    let sample_rate = 44100;
    let seconds = 5;
    let frecuency = 100.0;

    let volume = 64.0;
    let volume_offset = 128.0;

    let mut data = vec![0_u8; seconds * sample_rate as usize];

    for (i, sample) in data.iter_mut().enumerate() {
        let t = i as f32 / sample_rate as f32;
        *sample = (volume * f32::sin(2.0 * PI * frecuency * t) + volume_offset) as u8;
    }

    Wav::from_data(WavSamples::Mono8(data), sample_rate)
        .write(Path::new(&file_name))
        .expect("error writing WAV file");
}

fn create_sqrwav(file_name: &str) {
    let sample_rate = 44100;
    let seconds = 5;

    let volume = 32000;
    let period = 1.0 / 100.0;
    let samples_per_period = sample_rate as f32 * period;
    let high_samples = 0.5 * samples_per_period;

    let mut data = vec![0_i16; seconds * sample_rate as usize];

    for (i, sample) in data.iter_mut().enumerate() {
        *sample = volume * (i as f32 % samples_per_period < high_samples) as i16;
    }

    Wav::from_data(WavSamples::Mono16(data), sample_rate)
        .write(Path::new(&file_name))
        .expect("error writing WAV file");
}
