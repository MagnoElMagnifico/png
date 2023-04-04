use std::{fs, io, io::Write, iter::zip, path::Path};

const RIFF: [u8; 4] = [82, 73, 70, 70];
const WAVE: [u8; 4] = [87, 65, 86, 69];
const FMT: [u8; 4] = [102, 109, 116, 32];
const DATA: [u8; 4] = [100, 97, 116, 97];

//  8-bit samples are stored as unsigned bytes, ranging from 0 to 255. 16-bit samples are
//  stored as 2's-complement signed integers, ranging from -32768 to 32767
//
//  For stereo audio, channel 0 is the left channel (.0) and channel 1 is the right (.1).
#[derive(Debug, Clone)]
pub enum WavSamples {
    Stereo16(Vec<(i16, i16)>),
    Stereo8(Vec<(u8, u8)>),
    Mono16(Vec<i16>),
    Mono8(Vec<u8>),
}

#[rustfmt::skip]
impl WavSamples {
    pub fn from_bytes(data: &[u8], stereo: bool, bits_per_sample: u16) -> Self {
        assert!(
            bits_per_sample == 8 || bits_per_sample == 16,
            "allowed bits per sample are 8 or 16, got {bits_per_sample}"
        );

        match (stereo, bits_per_sample) {
            (true, 16) => {
                // Get i16 numbers
                let iter = zip(
                    data.iter().step_by(2), // even bytes
                    data.iter().skip(1).step_by(2)) // odd bytes
                .map(|(a, b)| i16::from_le_bytes([*a, *b])); // map to i16

                // Create pairs
                WavSamples::Stereo16(zip(
                    iter.clone().step_by(2),
                    iter.skip(1).step_by(2)
                ).collect())
            }

            (true, 8) => WavSamples::Stereo8(
                zip(
                    data.iter().step_by(2),
                    data.iter().skip(1).step_by(2))
                .map(|(a, b)| (*a, *b))
                .collect(),
            ),

            (false, 16) => WavSamples::Mono16(
                zip(
                    data.iter().step_by(2),
                    data.iter().skip(1).step_by(2))
                .map(|(a, b)| i16::from_le_bytes([*a, *b]))
                .collect(),
            ),

            (false, 8) => WavSamples::Mono8(data.to_vec()),
            (_, _) => unreachable!(),
        }
    }
}

impl Into<Vec<u8>> for WavSamples {
    fn into(self) -> Vec<u8> {
        match self {
            Self::Stereo16(data) => data
                .iter()
                .flat_map(|(a, b)| [a.to_le_bytes(), b.to_le_bytes()])
                .flatten()
                .collect(),
            Self::Stereo8(data) => data.iter().flat_map(|(a, b)| [*a, *b]).collect(),
            Self::Mono16(data) => data.iter().flat_map(|x| x.to_le_bytes()).collect(),
            Self::Mono8(data) => data,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Wav {
    pub data: WavSamples,
    sample_rate: u32,
}

impl Wav {
    pub fn from_data(data: WavSamples, sample_rate: u32) -> Self {
        Self { data, sample_rate }
    }

    #[rustfmt::skip]
    pub fn read(input_file: &Path) -> io::Result<Self> {
        let file_data = fs::read(input_file)?;
        assert!(file_data.len() > 44, "Incomplete WAV file");

        // RIFF header
        assert_eq!(file_data[..4], RIFF, "`RIFF` signature not found");
        // let _file_length = u32::from_le_bytes(file_data[4..8].try_into().expect("read WAV file length"));
        assert_eq!(file_data[8..12], WAVE, "`WAVE` signature not found");
        assert_eq!(file_data[12..16], FMT, "`fmt ` not found");

        // fmt chunk
        assert_eq!(u32::from_le_bytes(file_data[16..20].try_into().expect("read length of fmt chunk")), 16, "fmt chunk length must be 16 bytes");
        assert_eq!(u16::from_le_bytes(file_data[20..22].try_into().expect("read fmt format tag PCM")), 1, "format tag PCM must be 1");

        let channels         = u16::from_le_bytes(file_data[22..24].try_into().expect("read channels"));
        let sample_rate      = u32::from_le_bytes(file_data[24..28].try_into().expect("read sample rate"));
        let bytes_per_second = u32::from_le_bytes(file_data[28..32].try_into().expect("read bytes/second"));
        let block_align      = u16::from_le_bytes(file_data[32..34].try_into().expect("read block align"));
        let bits_per_sample  = u16::from_le_bytes(file_data[34..36].try_into().expect("read bits/sample"));

        // Logic checks
        assert_eq!(bytes_per_second, sample_rate * block_align as u32);
        assert_eq!(block_align, channels * bits_per_sample / 8);
        assert!(channels == 1 || channels == 2, "allowed channels are 1 or 2, got {channels}");

        // data chunk
        assert_eq!(file_data[36..40], DATA, "`data` signature not found");
        // let _data_length = u32::from_le_bytes(file_data[40..44].try_into().expect("read data length"));

        Ok(Self {
            data: WavSamples::from_bytes(&file_data[44..], channels == 2, bits_per_sample),
            sample_rate,
        })
    }

    pub fn write(self, output_file: &Path) -> io::Result<()> {
        let mut file = fs::File::create(output_file)?;

        // RIFF header
        file.write(&RIFF)?;
        file.write(&WAVE)?;

        // fmt chunk
        let (channels, bits_per_sample) = match self.data {
            WavSamples::Stereo16(_) => (2_u16, 16_u16),
            WavSamples::Stereo8(_) => (2_u16, 8_u16),
            WavSamples::Mono16(_) => (1_u16, 16_u16),
            WavSamples::Mono8(_) => (1_u16, 8_u16),
        };

        let block_align: u16 = channels * bits_per_sample / 8;
        let bytes_per_second: u32 = self.sample_rate * block_align as u32;

        file.write(&FMT)?;
        file.write(&16_u32.to_le_bytes())?; // length of fmt header
        file.write(&1_u16.to_le_bytes())?; // PCM format tag

        file.write(&channels.to_le_bytes())?;
        file.write(&self.sample_rate.to_le_bytes())?;
        file.write(&bytes_per_second.to_le_bytes())?;
        file.write(&block_align.to_le_bytes())?;
        file.write(&bits_per_sample.to_le_bytes())?;

        // data chunk
        let samples_data: Vec<u8> = self.data.into();

        file.write(&DATA)?;
        file.write(&(samples_data.len() as u32).to_le_bytes())?;
        file.write(&samples_data)?;

        Ok(())
    }
}
