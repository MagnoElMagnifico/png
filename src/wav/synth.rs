use crate::{Wav, WavSamples};
use std::f32::consts::TAU;

pub enum WaveForm {
    Sin,
    Sqr(f32),
    Saw,
    Custom(fn(usize, &Oscilator) -> u8),
}

pub struct Oscilator {
    wave: WaveForm,
    pub sample_rate: u32,
    pub spp: f32,
    pub frecuency: f32,
    pub volume: u8,
    pub offset: u8,
}

// sin_wave, square_wave, sawtooth_wave, triangular_wave, custom
// add
impl Oscilator {
    pub fn new(sample_rate: u32, frecuency: f32, wave: WaveForm, volume: u8, offset: u8) -> Self {
        Self {
            sample_rate,
            spp: sample_rate as f32 / frecuency,
            frecuency,
            wave,
            volume,
            offset,
        }
    }

    pub fn get_samples(&self, time: u32) -> WavSamples {
        let mut data = vec![0; (time * self.sample_rate / 1000) as usize];

        match self.wave {
            WaveForm::Sin => self.sin_wave(&mut data),
            WaveForm::Sqr(pulse_width) => self.sqr_wave(&mut data, pulse_width),
            WaveForm::Saw => self.saw_wave(&mut data),
            WaveForm::Custom(callback) => self.custom_wave(&mut data, callback),
        }

        WavSamples::Mono8(data)
    }

    pub fn to_wav(&self, time: u32) -> Wav {
        Wav::from_data(self.get_samples(time), self.sample_rate)
    }

    fn sin_wave(&self, data: &mut [u8]) {
        let sample_rate = self.sample_rate as f32;
        let volume = self.volume as f32;
        let offset = self.offset as f32;

        for (i, sample) in data.iter_mut().enumerate() {
            let t = i as f32 / sample_rate;
            *sample = (volume * f32::sin(TAU * self.frecuency * t) + offset) as u8;
        }
    }

    fn sqr_wave(&self, data: &mut [u8], pulse_width: f32) {
        let spp = self.spp as usize;
        let high_samples = (pulse_width * self.spp) as usize;

        for (i, sample) in data.iter_mut().enumerate() {
            let x = i % spp < high_samples;
            *sample = self.offset + self.volume * x as u8;
        }
    }

    fn saw_wave(&self, data: &mut [u8]) {
        for (i, sample) in data.iter_mut().enumerate() {
            // x is the porcentage of the wave at the current point
            let x = (i % self.spp as usize) as f32 / self.spp as f32;
            *sample = self.offset + self.volume * x as u8;
        }
    }

    fn custom_wave(&self, data: &mut [u8], callback: fn(usize, &Oscilator) -> u8) {
        for (i, sample) in data.iter_mut().enumerate() {
            *sample = callback(i, self);
        }
    }
}
