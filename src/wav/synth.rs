use crate::WavSamples;
use std::f32::consts::TAU;

pub struct Oscilator {
    sample_rate: u32,
    samples_per_period: f32,
    time: f32,

    frecuency: f32,
    volume: u8,
    offset: u8,

    pulse_width: f32,
}

// get_samples
// volume
// volume_offset
// sin_wave, square_wave, sawtooth_wave, triangular_wave, custom
// add
impl Oscilator {
    pub fn new(sample_rate: u32, time:f32, frecuency: f32, volume: u8, offset: u8, pulse_width: f32) -> Self {
        Self {
            sample_rate,
            samples_per_period: sample_rate as f32 / frecuency,
            time,
            frecuency,
            volume,
            offset,
            pulse_width,
        }
    }

    pub fn sin_wave(&self) -> WavSamples {
        let mut data = vec![0; (self.time * self.sample_rate as f32) as usize];

        for (i, sample) in data.iter_mut().enumerate() {
            let t = i as f32 / self.sample_rate as f32;
            *sample = (self.volume as f32 * f32::sin(TAU * self.frecuency * t) + self.offset as f32) as u8;
        }

        WavSamples::Mono8(data)
    }

    pub fn sqr_wave(&self) -> WavSamples {
        let mut data = vec![0; (self.time * self.sample_rate as f32) as usize];

        for (i, sample) in data.iter_mut().enumerate() {
            let x = (i % self.samples_per_period as usize) < (self.pulse_width * self.samples_per_period) as usize;
            *sample = self.volume * x as u8 + self.offset;
        }

        WavSamples::Mono8(data)
    }

    pub fn saw_wave(&self) -> WavSamples {
        let mut data = vec![0; (self.time * self.sample_rate as f32) as usize];

        for (i, sample) in data.iter_mut().enumerate() {
            // x is the porcentage of the wave at the current point
            let x = (i % self.samples_per_period as usize) as f32 / self.samples_per_period as f32;
            *sample = self.offset + self.volume * x as u8;
        }

        WavSamples::Mono8(data)
    }
}

