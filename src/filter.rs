//! Algorithms that prepare the image data for optimum compression, because it can significantly
//! reduce the resultant size.
//!
//! PNG filter method 0 (described by IHDR, the only one as for PNG 1.2) defines five basic filter
//! types:
//!
//! ```
//! 0   None
//! 1   Sub
//! 2   Up
//! 3   Average
//! 4   Paeth
//! ```

/// Transmits the difference between each byte and the value of the corresponding byte of the prior
/// pixel.
///
/// Formula for each byte (being x a byte):
///
///     Sub(x) = Raw(x) - Raw(x - bpp)
///
/// Unsigned arithmetic modulo 256 is used, so both inputs and outputs fit into into bytes.
///
/// Where
///
/// - `Raw(pos)`    raw data byte in position pos (if x < 0, asume Raw(x) = 0)
/// - `bpp`         number of bytes per pixel
///
/// For example:
///
/// - For color type 2 with a bit depth of 16, `bpp` is equal to 6 (three samples, two bytes per
///   sample)
/// - For color type 0 with a bit depth of 2, `bpp` is equal to 1 (rounding up)
/// - For color type 4 with a bit depth of 16, `bpp` is equal to 4 (two-byte greyscale sample, plus
///   two-byte alpha sample).
pub fn _sub(data: &mut [u8]) {
    let _bpp = 0;
    for (_x, _raw) in data.iter_mut().enumerate() {
        // *raw -= *raw - data.get(x - bpp).unwrap_or(&0);
    }
}

/// The inverse of the sub filter: Sub(x) + Raw(x - bpp)

pub fn _sub_inv(data: &mut [u8]) {
    let _bpp = 0;
    for (_x, _raw) in data.iter_mut().enumerate() {
        // *raw += data.get(x - bpp).unwrap_or(&0);
    }
}
