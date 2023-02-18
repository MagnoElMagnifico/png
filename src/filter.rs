#![allow(dead_code)]
//! Algorithms that prepare the image data for optimum compression, because it can significantly
//! reduce the resultant size.
//!
//! PNG filter method 0 (described by IHDR, the only one as for PNG 1.2) defines five basic filter
//! types:
//!
//!   0. None
//!   1. Sub
//!   2. Up
//!   3. Average
//!   4. Paeth
//!
//! These are applied to scanlines: a 1-pixel-high sequence starting at the far left and ending at
//! the far right of the image.
//!
//! Note that these functions also add the filter-type byte of the method. The inverse functions
//! also take in this byte and remove it from the output.
//!
//! `Raw(pos)`: unfiltered data byte in position pos (if `x < 0`, asume `Raw(x) = 0`)
//!
//! Therefore, to filter the whole image, go from the bottom to the top. To decode the image, from
//! top to bottom.
//!
//! Unsigned arithmetic modulo 256 is used, so both inputs and outputs fit into into bytes.

/// bpp stands for bytes per complete pixel, rounding up to 1. It depends on the bit depth and
/// color type set on the IHDR chunk.
///
/// Examples:
///
/// - Color type 2, bit depth 16 => `bpp` is 6 (three samples, two bytes per sample)
/// - Color type 0, bit depth 2  => `bpp` is 1 (rounding up)
/// - Color type 4, bit depth 16 => `bpp` is 4 (two-byte greyscale sample, plus two-byte alpha sample).
pub fn bytes_per_pixel(color_type: u8, bit_depth: u8) -> u8 {
    let mut n_samples = 1; // Grayscale or index: 1 sample
    n_samples += color_type & (1 << 1); // RGB: +2 samples (not shift back, it is multiplied by 2)
    n_samples += (color_type & (1 << 2)) >> 2; // Add 1 sample for alpha

    // Bytes per sample
    let bps = ((bit_depth & (1 << 4)) >> 4) + 1; // If 16, 2 bytes. 1 byte otherwise.
    n_samples * bps
}

/// Transmits the difference between each byte and the value of the corresponding byte of the prior
/// pixel.
///
/// Formula for each byte (being x a byte):
///
/// ```
/// Sub(x) = Raw(x) - Raw(x - bpp)
/// ```
pub fn sub(scanline: &[u8], bpp: u8) -> Vec<u8> {
    let bpp = bpp as usize;
    let mut filtered = scanline.to_vec();

    for (i, byte) in scanline.iter().enumerate() {
        let previous_byte = if i < bpp { 0 } else { scanline[i - bpp] };
        filtered[i] = byte.wrapping_sub(previous_byte);
    }

    filtered.insert(0, 1); // Add filter-type byte method for sub
    filtered
}

/// The inverse of the `sub` filter:
///
/// ```
/// Sub(x) + Raw(x - bpp)
/// ```
pub fn sub_inv(filtered: &[u8], bpp: u8) -> Vec<u8> {
    let bpp = bpp as usize;
    let mut original = filtered[1..].to_vec(); // Ignore filter-type byte

    for (i, byte) in filtered.iter().skip(1).enumerate() {
        let previous_byte = if i < bpp { 0 } else { original[i - bpp] };
        original[i] = byte.wrapping_add(previous_byte);
    }

    original
}

/// Similar to the `Sub()` filter, except that the pixel immediately above the current one, rather
/// than just to its left, is used. Note that this scanline should be unfiltered.
///
/// Formula for each byte (being x a byte):
///
/// ```
/// Up(x) = Raw(x) - Prior(x)
/// ```
///
/// If a prior scanline cannot be found, 0 will be assumed.
pub fn up(scanline: &[u8], prior_scanline: &[u8]) -> Vec<u8> {
    let mut filtered = scanline.to_vec();

    for (i, byte) in scanline.iter().enumerate() {
        let prior_byte = prior_scanline.get(i).unwrap_or(&0);
        filtered[i] = byte.wrapping_sub(*prior_byte);
    }

    filtered.insert(0, 2);  // Add filter-type byte method for up
    filtered

}

/// The inverse of the `up` filter: `Up(x) + Prior(x)`
/// NOTE: `Prior()` are decoded bytes
pub fn up_inv(filtered: &[u8], prior_scanline: &[u8]) -> Vec<u8> {
    let mut original = filtered[1..].to_vec(); // Ignore filter-type byte

    for (i, byte) in filtered.iter().skip(1).enumerate() {
        let prior_byte = prior_scanline.get(i).unwrap_or(&0);
        original[i] = byte.wrapping_add(*prior_byte);
    }

    original
}

/// Mix of the methods `Sub()` and `Up()`: takes the average of the left and above pixel.
///
/// ```
/// Average(x) = Raw(x) - floor( (Raw(x - bpp) + Prior(x)) / 2)
/// ```
///
/// However, the sum Raw(x-bpp)+Prior(x) must be formed without overflow (using at least nine-bit arithmetic).
/// floor() could be integer division or >> 2
pub fn average(scanline: &[u8], prior_scanline: &[u8], bpp: u8) -> Vec<u8> {
    let bpp = bpp as usize;
    let mut filtered = scanline.to_vec();

    for (i, byte) in scanline.iter().enumerate() {
        let previous_byte = if i < bpp { 0 } else { scanline[i - bpp] };
        let prior_byte = prior_scanline.get(i).unwrap_or(&0);
        let floor = (previous_byte as u16 + *prior_byte as u16) >> 2;

        filtered[i] = byte.wrapping_sub(floor as u8);
    }

    filtered.insert(0, 3);
    filtered
}

/// ```
/// Average(x) + floor((Raw(x-bpp)+Prior(x))/2)
/// ```
pub fn average_inv(filtered: &[u8], prior_scanline: &[u8], bpp: u8) -> Vec<u8> {
    let bpp = bpp as usize;
    let mut original = filtered[1..].to_vec();

    for (i, byte) in filtered.iter().skip(1).enumerate() {
        let previous_byte = if i < bpp { 0 } else { original[i - bpp] };
        let prior_byte = prior_scanline.get(i).unwrap_or(&0);
        let floor = (previous_byte as u16 + *prior_byte as u16) >> 2;

        original[i] = byte.wrapping_add(floor as u8);
    }

    original
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_per_pixel_test() {
        // RGB => 3 samples, 2 bytes per sample
        assert_eq!(bytes_per_pixel(2, 16), 6);
        // Greyscale => 1 sample, 1 byte per sample
        assert_eq!(bytes_per_pixel(0, 2), 1);
        // Greyscale with alpha => 2 samples, 2 bytes per sample
        assert_eq!(bytes_per_pixel(4, 16), 4);
    }

    #[test]
    fn sub_test() {
        let random_scanline = vec![4, 5, 6, 7, 8, 9, 10, 11, 12];
        let bpp = 1;

        // With this example, you can clearly see the power of this filter.
        //
        // For a sequence of values that change with a regular pattern, they can be stored as
        // distances, therefore similar values and easier to compress. Moreover, it is a reversible
        // operation.
        //
        // In this case, the filtered scanline should be [1, 4, 1, 1, 1, 1, 1, 1, 1, 1].

        let filtered = sub(&random_scanline, bpp);
        let inverse = sub_inv(&filtered, bpp);
        assert_eq!(random_scanline, inverse);
    }

    #[test]
    fn up_test() {
        let prior_scanline     = vec![41, 123, 1, 54, 127, 230, 69];
        let scanline_to_filter = vec![42, 124, 2, 55, 128, 231, 70];

        let filtered = up(&scanline_to_filter, &prior_scanline);
        let inverse  = up_inv(&filtered, &prior_scanline);
        assert_eq!(scanline_to_filter, inverse);

        // Now test if the scanline were the first
        let filtered = up(&scanline_to_filter, &[]);
        let inverse  = up_inv(&filtered, &[]);
        assert_eq!(scanline_to_filter, inverse);
    }

    #[test]
    fn average_test() {
        let prior_scanline = vec![1, 2, 3, 4, 5, 6, 8, 9];
        let scanline       = vec![6, 10, 7, 9, 9, 12, 2, 14];
        let bpp = 1;

        let filtered = average(&scanline, &prior_scanline, bpp);
        let inverse  = average_inv(&filtered, &prior_scanline, bpp);
        assert_eq!(scanline, inverse);

        // Now test if the scanline were the first
        let filtered = up(&scanline, &[]);
        let inverse  = up_inv(&filtered, &[]);
        assert_eq!(scanline, inverse);
    }
}
