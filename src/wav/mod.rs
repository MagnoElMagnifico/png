pub mod synth;

use std::{fs, io, io::Write, iter::zip, path::Path};

/// Text `RIFF` encoded in ASCII
const RIFF: [u8; 4] = [82, 73, 70, 70];
/// Text `WAVE` enconded in ASCII
const WAVE: [u8; 4] = [87, 65, 86, 69];
/// Text `fmt ` encoded in ASCII
const FMT: [u8; 4] = [102, 109, 116, 32];
/// Text `data` encoded in ASCII
const DATA: [u8; 4] = [100, 97, 116, 97];

/// Data Structure representing WAV samples.
///
/// Bits per sample:
///
/// - 8-bit samples: stored as unsigned bytes, ranging from 0 to 255.
/// - 16-bit samples: stores as 2's-complement signed integers, ranging from -32768 to 32767.
///
/// Channels:
///
/// - Stereo: 2 channels
/// - Mono: 1 channel
///
/// Fo multi-channel data, samples are interleaved between channels:
///
/// ```
/// sample 0 for channel 0
/// sample 0 for channel 1
/// sample 1 for channel 0
/// sample 1 for channel 1
/// ```
///
/// Where, for stereo audio, channel 0 is left and 1 is right.
#[derive(Debug, Clone)]
pub enum WavSamples {
    Stereo16(Vec<(i16, i16)>),
    Stereo8(Vec<(u8, u8)>),
    Mono16(Vec<i16>),
    Mono8(Vec<u8>),
}

#[rustfmt::skip]
impl WavSamples {
    /// Converts a buffer into its corresponding WavSamples.
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

/// Data Structure representing a WAV file.
///
/// # WAV file format
///
/// The canonical WAVE format starts with the RIFF header:
///
/// ```
///  Offset  Length   Contents
///  0       4 bytes  'RIFF' = 0x52494646
///  4       4 bytes  <file length - 8>
///  8       4 bytes  'WAVE' = 0x57415645
/// ```
///
/// The `8` on the second entry is the length of the first two, thus the second entry is the number
/// of bytes that follow in the file.
///
/// Next a `fmt` chunk that describes the sample format:
///
/// ```
///  Offset  Length   Contents
///  12      4 bytes  'fmt ' = 0x666d7420
///  16      4 bytes  0x00000010            // Length of the fmt data (16 bytes)
///  20      2 bytes  0x0001                // Format tag: 1 = PCM
///  22      2 bytes  <channels>            // Channels: 1 = mono, 2 = stereo
///  24      4 bytes  <sample rate>         // Samples per second: e.g., 44100
///  28      4 bytes  <bytes/second>        // sample rate * block align
///  32      2 bytes  <block align>         // channels * bits/sample / 8
///  34      2 bytes  <bits/sample>         // 8 or 16
///  ```
///
///  Finally, the `data` chunk conteining the sample data:
///
/// Finally, the data chunk contains the sample data:
///
/// ```
///  Offset  Length   Contents
///  36      4 bytes  'data' = 0x64617461
///  40      4 bytes  <length of the data block> = <file length - 36>
///  44      -        <sample data>
/// ```
///
/// (Source)[http://www.lightlink.com/tjweber/StripWav/Canon.html]
#[derive(Debug, Clone)]
pub struct Wav {
    pub data: WavSamples,
    sample_rate: u32,
}

impl Wav {
    /// Creates a new Wav given the samples and the sample rate.
    pub fn from_data(data: WavSamples, sample_rate: u32) -> Self {
        Self { data, sample_rate }
    }

    /// Reads the file given and converts its contents into WavSamples.
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

        // The entire function can be simplified to:
        // Ok(Self {
        //      data: WavSamples::from_bytes(
        //          &file_data[44..],
        //          u16::from_le_bytes(file_data[22..24].try_into().expect("read channels") == 2
        //          u16::from_le_bytes(file_data[34..36].try_into().expect("read bits/sample")
        //      ),
        //      sample_rate: u32::from_le_bytes(file_data[24..28].try_into().expect("read sample rate")),
        // })
    }

    /// Writes to the filepath given the WAV file.
    pub fn write(self, output_file: &Path) -> io::Result<()> {
        let (channels, bits_per_sample) = match self.data {
            WavSamples::Stereo16(_) => (2_u16, 16_u16),
            WavSamples::Stereo8(_) => (2_u16, 8_u16),
            WavSamples::Mono16(_) => (1_u16, 16_u16),
            WavSamples::Mono8(_) => (1_u16, 8_u16),
        };
        let block_align: u16 = channels * bits_per_sample / 8;
        let bytes_per_second: u32 = self.sample_rate * block_align as u32;

        let samples_data: Vec<u8> = self.data.into();
        let file_length: u32 = samples_data.len() as u32 + 36;

        let mut file = fs::File::create(output_file)?;

        // RIFF header
        file.write(&RIFF)?;
        file.write(&file_length.to_le_bytes())?;
        file.write(&WAVE)?;

        // fmt chunk
        file.write(&FMT)?;
        file.write(&16_u32.to_le_bytes())?; // length of fmt header
        file.write(&1_u16.to_le_bytes())?; // PCM format tag

        file.write(&channels.to_le_bytes())?;
        file.write(&self.sample_rate.to_le_bytes())?;
        file.write(&bytes_per_second.to_le_bytes())?;
        file.write(&block_align.to_le_bytes())?;
        file.write(&bits_per_sample.to_le_bytes())?;

        // data chunk
        file.write(&DATA)?;
        file.write(&(samples_data.len() as u32).to_le_bytes())?;
        file.write(&samples_data)?;

        Ok(())
    }
}
