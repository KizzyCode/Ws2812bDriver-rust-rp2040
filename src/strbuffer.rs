//! A stack-allocated buffer that implements `core::fmt::Write`

use core::fmt::{Display, Formatter, Write};
use core::ops::Deref;

/// A stack-allocated buffer that implements `core::fmt::Write`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct StrBuffer<const SIZE: usize> {
    /// The string bytes
    bytes: [u8; SIZE],
    /// The size of the string in bytes
    len: usize,
}
impl<const SIZE: usize> StrBuffer<SIZE> {
    /// Creates a new empty panic buffer
    pub const fn new() -> Self {
        Self { bytes: [0; SIZE], len: 0 }
    }
}
impl<const SIZE: usize> Write for StrBuffer<SIZE> {
    #[inline(never)]
    fn write_str(&mut self, str_: &str) -> core::fmt::Result {
        // Get the target subbuffer
        let bytes = &mut self.bytes[self.len..];
        let to_copy = core::cmp::min(str_.len(), bytes.len());
        self.len += to_copy;

        // Copy the string using a volatile write to ensure this is not optimized away
        let mut dest = bytes.as_mut_ptr();
        for source in str_.bytes().take(to_copy) {
            unsafe { dest.write_volatile(source) };
            unsafe { dest = dest.add(1) };
        }
        Ok(())
    }
}
impl<const SIZE: usize> Display for StrBuffer<SIZE> {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        // Write the message
        for byte in self.bytes.iter().take(self.len) {
            // Escape the byte if necessary
            match byte.is_ascii_graphic() | byte.is_ascii_whitespace() {
                true => write!(f, "{}", *byte as char)?,
                false => write!(f, r#"\x{:02x}"#, byte)?,
            };
        }
        Ok(())
    }
}
impl<const SIZE: usize> Deref for StrBuffer<SIZE> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        core::str::from_utf8(&self.bytes[..self.len]).expect("string is not UTF-8")
    }
}
