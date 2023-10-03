//! Provides access to the underlying hardware

pub mod flash;
pub mod init;
pub mod pins;
pub mod usb;

/// Compile-time `const` macro to unwrap an optional
#[macro_export]
macro_rules! const_unwrap {
    ($value:expr, or: $default:expr) => {{
        match $value {
            Some(value) => value,
            None => $default,
        }
    }};
    ($value:expr, $desc:expr) => {{
        match $value {
            Some(value) => value,
            None => panic!($desc),
        }
    }};
}

/// Compile-time `const` macro to parse a `u8` from a `str`
#[macro_export]
macro_rules! const_int_from_compileenv {
    ($name:expr => $type:ty, default: $default:expr) => {{
        // Get environment variable
        let strval = $crate::const_unwrap!(option_env!($name), or: $default);
        let strval = strval.as_bytes();

        // Parse chars
        let mut intval: $type = 0;
        let mut pos = 0;
        while pos < strval.len() {
            // Get the byte
            let byte = strval[pos];
            pos += 1;

            // Shift by `1` in decimal and append next number
            intval = $crate::const_unwrap!(intval.checked_mul(10), "pin number is too large");
            intval = match byte {
                b'0' => $crate::const_unwrap!(intval.checked_add(0), "pin number is too large"),
                b'1' => $crate::const_unwrap!(intval.checked_add(1), "pin number is too large"),
                b'2' => $crate::const_unwrap!(intval.checked_add(2), "pin number is too large"),
                b'3' => $crate::const_unwrap!(intval.checked_add(3), "pin number is too large"),
                b'4' => $crate::const_unwrap!(intval.checked_add(4), "pin number is too large"),
                b'5' => $crate::const_unwrap!(intval.checked_add(5), "pin number is too large"),
                b'6' => $crate::const_unwrap!(intval.checked_add(6), "pin number is too large"),
                b'7' => $crate::const_unwrap!(intval.checked_add(7), "pin number is too large"),
                b'8' => $crate::const_unwrap!(intval.checked_add(8), "pin number is too large"),
                b'9' => $crate::const_unwrap!(intval.checked_add(9), "pin number is too large"),
                _ => panic!("pin number contains non-numeric character"),
            };
        }
        intval
    }};
}
