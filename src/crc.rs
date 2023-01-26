//! A Cyclic redundancy check (CRC) is an error-detecting code. Blocks of data entering these
//! systems get a short check value attached, based on the remainder of a polynomial division of
//! their contents.
//!
//! Specification of a CRC code requires definition of a so-called generator polynomial. This
//! polynomial becomes the divisor in a polynomial long division, which takes the message as the
//! dividend and in which the quotient is discarded and the remainder becomes the result. The
//! important caveat is that the polynomial coefficients are calculated according to the arithmetic
//! of a finite field, so the addition operation can always be performed bitwise-parallel (there is
//! no carry between digits). In practice, all commonly used CRCs employ the Galois field, or more
//! simply a finite field, of two elements, GF(2). The two elements are usually called 0 and 1,
//! comfortably matching computer architecture.
//!
//! A CRC is called an n-bit CRC when its check value is n bits long. For a given n, multiple CRCs
//! are possible, each with a different polynomial. Such a polynomial has highest degree n, which
//! means it has n + 1 terms. In other words, the polynomial has a length of n + 1; its encoding
//! requires n + 1 bits. Note that most polynomial specifications either drop the MSB or LSB, since
//! they are always 1.
//!
//! On the PNG's case, the CRC used is CRC-32, whose polynomial is:
//!
//! x^32 + x^26 + x^23 + x^22 + x^16 + x^12 + x^11 + x^10 + x^8 + x^7 + x^5 + x^4 + x^2 + x + 1
//!
//! Thus the coefficients are (1 - 32, ignoring 32): 1110 1101 1011 1000 1000 0110 0100 0000
//! which is exactly EBD88320 in hex.
//!
//! # Example
//!
//! In this example, we shall encode 14 bits of message with a 3-bit CRC, with a polynomial `x^3 + x
//! + 1` (coefficients 1011). Start with the message to be encoded: `11 0100 1110 1100` and execute
//! a bitwise XOR:
//!
//! ```
//! 11010011101100 000 <--- input right padded by 3 bits
//! 1011               <--- divisor
//! 01100011101100 000 <--- result (note the first four bits are the XOR with the divisor beneath, the rest of the bits are unchanged)
//!  1011              <--- divisor ...
//! 00111011101100 000
//!   1011
//! 00010111101100 000
//!    1011
//! 00000001101100 000 <--- note that the divisor moves over to align with the next 1 in the dividend (since quotient for that step was zero)
//!        1011             (in other words, it doesn't necessarily move one bit per iteration)
//! 00000000110100 000
//!         1011
//! 00000000011000 000
//!          1011
//! 00000000001110 000
//!           1011
//! 00000000000101 000
//!            101 1
//! -----------------
//! 00000000000000 100 <--- remainder (3 bits).  Division algorithm stops here as dividend is equal to zero.
//! ```
//!
//! Now, to check the validity of the message, the operation will be repeated with the remainder
//! instead of zeroes. It should equal zero if there are no detectable errors.
//!
//! ```
//! 11010011101100 100 <--- input with check value
//! 1011               <--- divisor
//! 01100011101100 100 <--- result
//!   ...
//! 00000000000101 100
//!            101 1
//! ------------------
//! 00000000000000 000 <--- remainder
//! ```
//!
//! A practical algorithm for the CRC-32 variant of CRC is the CRCTable, which is a memoization
//! (storage of all the possibilities -- 256) of a calculation that would have to be repeated for
//! each byte of the message.
//!
//! Source (modified): https://en.wikipedia.org/wiki/Cyclic_redundancy_check
//!
//! Translated from the C code avaliable here:
//! http://libpng.org/pub/png/spec/1.2/PNG-CRCAppendix.html

const CRC_MASK: u32 = 0xEDB88320;
const CRC_TABLE_SZ: usize = u8::MAX as usize + 1;

#[derive(Debug, Clone)]
pub struct Crc([u32; CRC_TABLE_SZ]);

impl Crc {
    pub fn new() -> Self {
        let mut table = [0; CRC_TABLE_SZ];

        for (i, table_byte) in table.iter_mut().enumerate() {
            let mut byte = i as u32;
            for _ in 0..8 {
                if (byte & 1) == 1 {
                    byte = CRC_MASK ^ (byte >> 1);
                } else {
                    byte >>= 1;
                }
            }

            *table_byte = byte;
        }

        Crc(table)
    }

    /// Returns the CRC of the bytes on buffer.
    pub fn calculate(&self, buffer: &[u8]) -> u32 {
        let mut crc = 0xFFFF_FFFF_u32;

        for byte in buffer {
            let index = crc as u8 ^ byte;
            crc = (crc >> 8) ^ self.0[index as usize];
        }

        // Invert the bits (1's complement)
        crc ^ 0xFFFF_FFFF_u32
    }
}
