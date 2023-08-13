//! A WS2812B pixel update command

/// A WS2812B pixel update command that can be compressed into 32 bits
///
/// # Important
/// To compress all values into a single `u32`, the format has some constraints, namely:
///  - only 4 strip indices (`0..=3`)
///  - only 512 pixel indices (`0..=511`)
///  - `r`, `g` and `b` are compressed to 7 bits by setting the least-significant bit to `0`
#[derive(Debug, Clone, Copy)]
pub struct Command {
    /// The index of the WS2812B strip to update
    pub strip: usize,
    /// The index of the pixel to update
    pub pixel: usize,
    /// The new RGB value
    pub rgb: (u8, u8, u8),
}
impl Command {
    /// The size of a serial command
    pub const SERIAL_LEN: usize = 16 + 1;

    /// Decodes a serial command
    pub fn from_serial(data: &[u8]) -> Option<Self> {
        /// Decodes a nibble from it's hex representation
        #[inline]
        const fn decode_nibble(nibble: u8) -> Option<u8> {
            #[allow(clippy::identity_op)]
            match nibble {
                b'0'..=b'9' => Some(nibble - (b'0' - 0x0)),
                b'a'..=b'f' => Some(nibble - (b'a' - 0xa)),
                b'A'..=b'F' => Some(nibble - (b'A' - 0xA)),
                _ => None,
            }
        }

        // Validate length and EOL
        let Some(b'\n') = data.get(Self::SERIAL_LEN - 1) else {
            return None;
        };

        // Hex-decode the bytes
        // Note: We use direct indexing here for performance reasons because the decoding routine is a bottle-neck
        let mut binary = [0; 8];
        for index in 0..binary.len() {
            // Decode the hex literal
            let high = decode_nibble(data[index * 2])?;
            let low = decode_nibble(data[(index * 2) + 1])?;
            binary[index] = (high << 4) | (low & 0xF);
        }

        // Cast the bytes
        // Note: This is not pretty, but it's fast and the decoding routine is a bottle-neck
        let strip = ((binary[0] as u16) << 8) | (binary[1] as u16);
        let pixel = ((binary[2] as u16) << 8) | (binary[3] as u16);
        let rgbw = [binary[4], binary[5], binary[6], binary[7]];

        // Validate data
        let strip @ 0..=3 = strip else {
            return None;
        };
        let pixel @ 0..=511 = pixel else {
            return None;
        };
        let [r, g, b, 0] = rgbw else {
            return None;
        };

        // Init self
        Some(Self { strip: strip as usize, pixel: pixel as usize, rgb: (r, g, b) })
    }

    /// Parses the command from it's packed `u32` representation
    pub const fn from_u32(packed: u32) -> Self {
        // Destructure u32 blob
        let strip = (packed >> 30) & 0b11;
        let pixel = (packed >> 21) & 0b1_1111_1111;
        let red = (packed >> 14) & 0b0111_1111;
        let green = (packed >> 7) & 0b0111_1111;
        let blue = packed & 0b0111_1111;

        // Init self
        Self {
            // Convert strip and pixel to usize indices
            strip: strip as usize,
            pixel: pixel as usize,
            // We use the left-shift because we killed the least-significant bit during compression
            rgb: ((red << 1) as u8, (green << 1) as u8, (blue << 1) as u8),
        }
    }
    /// Serializes the command into it's packed `u32` representation
    ///
    /// # Important
    /// If `self.strip` is larger than `STRIP_INDEX_MAX` or if `self.pixel` is larger than `PIXEL_INDEX_MAX`, the values
    /// are silently truncated.
    pub const fn to_u32(self) -> u32 {
        // Compress rgb values
        let (r, g, b) = self.rgb;
        let (r, g, b) = (r >> 1, g >> 1, b >> 1);

        // Pack the struct
        let mut packed = ((self.strip & 0b11) as u32) << 30;
        packed |= ((self.pixel & 0b1_1111_1111) as u32) << 21;
        packed |= ((r & 0b0111_1111) as u32) << 14;
        packed |= ((g & 0b0111_1111) as u32) << 7;
        packed |= (b & 0b0111_1111) as u32;
        packed
    }
}
