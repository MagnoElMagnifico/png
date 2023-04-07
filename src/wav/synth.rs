use crate::{Wav, WavSamples};
use std::f32::consts::TAU;

pub trait Oscillator {
    fn get_samples(&self, time: u32) -> WavSamples;
    fn get_sample_rate(&self) -> u32;

    fn to_wav(&self, time: u32) -> Wav {
        Wav::from_data(self.get_samples(time), self.get_sample_rate())
    }

    fn weighted_sum(self, other: Self) -> Self;
}

pub enum BasicWaveForm {
    Sin,
    Sqr(f32),
    Saw,
    Custom(fn(usize, &BasicOscillator) -> u8),
}

pub struct BasicOscillator {
    wave: BasicWaveForm,
    sample_rate: u32,
    pub frecuency: f32,
    pub volume: u8,
    pub offset: u8,
}

impl BasicOscillator {
    pub fn new(sample_rate: u32, frecuency: f32, wave: BasicWaveForm, volume: u8, offset: u8) -> Self {
        Self {
            sample_rate,
            frecuency,
            wave,
            volume,
            offset,
        }
    }

    pub fn get_spp(&self) -> f32 {
        self.sample_rate as f32 / self.frecuency
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
        let spp = self.get_spp() as usize;
        let high_samples = (pulse_width * self.get_spp()) as usize;

        for (i, sample) in data.iter_mut().enumerate() {
            let x = i % spp < high_samples;
            *sample = self.offset + self.volume * x as u8;
        }
    }

    fn saw_wave(&self, data: &mut [u8]) {
        for (i, sample) in data.iter_mut().enumerate() {
            // x is the porcentage of the wave at the current point
            let x = (i % self.get_spp() as usize) as f32 / self.get_spp() as f32;
            *sample = self.offset + self.volume * x as u8;
        }
    }

    fn custom_wave(&self, data: &mut [u8], callback: fn(usize, &BasicOscillator) -> u8) {
        for (i, sample) in data.iter_mut().enumerate() {
            *sample = callback(i, self);
        }
    }
}

impl Oscillator for BasicOscillator {
    fn get_samples(&self, time: u32) -> WavSamples {
        let mut data = vec![0; (time * self.sample_rate / 1000) as usize];

        match self.wave {
            BasicWaveForm::Sin => self.sin_wave(&mut data),
            BasicWaveForm::Sqr(pulse_width) => self.sqr_wave(&mut data, pulse_width),
            BasicWaveForm::Saw => self.saw_wave(&mut data),
            BasicWaveForm::Custom(callback) => self.custom_wave(&mut data, callback),
        }

        WavSamples::Mono8(data)
    }

    fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn weighted_sum(self, _other: Self) -> Self {
        todo!();
    }
}
