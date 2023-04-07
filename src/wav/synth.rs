use crate::{Wav, WavSamples};
use std::f32::consts::TAU;

pub trait Oscillator {
    fn sample(&self, x: usize) -> u8;
    fn get_sample_rate(&self) -> u32;

    fn get_samples(&self, time: u32) -> WavSamples {
        let mut data = vec![0; (time * self.get_sample_rate() / 1000) as usize];

        for (i, sample) in data.iter_mut().enumerate() {
            *sample = self.sample(i);
        }

        WavSamples::Mono8(data)
    }

    fn to_wav(&self, time: u32) -> Wav {
        Wav::from_data(self.get_samples(time), self.get_sample_rate())
    }
}

pub struct SinOsc {
    sample_rate: u32,
    pub frecuency: f32,
    pub volume: u8,
    pub offset: u8,
}

impl SinOsc {
    pub fn new(sample_rate: u32, frecuency: f32, volume: u8, offset: u8) -> Self {
        Self {
            sample_rate,
            frecuency,
            volume,
            offset,
        }
    }
}

impl Oscillator for SinOsc {
    fn sample(&self, x: usize) -> u8 {
        let t = x as f32 / self.sample_rate as f32;
        (self.volume as f32 * f32::sin(TAU * self.frecuency * t) + self.offset as f32) as u8
    }

    fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

pub struct SqrOsc {
    sample_rate: u32,
    pub frecuency: f32,
    pub pulse_width: f32,
    pub volume: u8,
    pub offset: u8,
}

impl SqrOsc {
    pub fn new(sample_rate: u32, frecuency: f32, pulse_width: f32, volume: u8, offset: u8) -> Self {
        Self {
            sample_rate,
            frecuency,
            pulse_width,
            volume,
            offset,
        }
    }
}

impl Oscillator for SqrOsc {
    fn sample(&self, x: usize) -> u8 {
        let spp = self.sample_rate as f32 / self.frecuency;
        let high_samples = (self.pulse_width * spp) as usize;

        let x = x % (spp as usize) < high_samples;
        self.offset + self.volume * x as u8
    }

    fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }
}


pub struct SawOsc {
    sample_rate: u32,
    pub frecuency: f32,
    pub volume: u8,
    pub offset: u8,
}

impl SawOsc {
    pub fn new(sample_rate: u32, frecuency: f32, volume: u8, offset: u8) -> Self {
        Self {
            sample_rate,
            frecuency,
            volume,
            offset,
        }
    }
}

impl Oscillator for SawOsc {
    fn sample(&self, x: usize) -> u8 {
        let spp = self.sample_rate as f32 / self.frecuency;
        let x = (x % spp as usize) as f32 / spp as f32;
        self.offset + self.volume * x as u8
    }

    fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }
}


pub struct CustomOsc {
    sample_rate: u32,
    sample_fn: fn(usize) -> u8, // TODO: add options
}

impl CustomOsc {
    pub fn new(sample_rate: u32, sample_fn: fn(usize) -> u8) -> Self {
        Self {
            sample_rate,
            sample_fn,
        }
    }
}

impl Oscillator for CustomOsc {
    fn sample(&self, x: usize) -> u8 {
        (self.sample_fn)(x)
    }

    fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }
}
