//! Dynamic pin selection for PIO GPIO pins

use crate::board::{
    hal::gpio::{DynPinId, Function, FunctionNull, FunctionPio0, FunctionSioOutput, Pin, PullDown},
    Pins,
};

/// `const` macro to unwrap an optional
macro_rules! unwrap {
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

/// `const` macro to parse a `u8` from a `str`
macro_rules! u8_from_compileenv {
    ($name:expr, default: $default:expr) => {{
        // Get environment variable
        let strval = unwrap!(option_env!($name), or: $default);
        let strval = strval.as_bytes();

        // Parse chars
        let mut intval: u8 = 0;
        let mut pos = 0;
        while pos < strval.len() {
            // Get the byte
            let byte = strval[pos];
            pos += 1;

            // Shift by `1` in decimal and append next number
            intval = unwrap!(intval.checked_mul(10), "pin number is too large");
            intval = match byte {
                b'0' => unwrap!(intval.checked_add(0), "pin number is too large"),
                b'1' => unwrap!(intval.checked_add(1), "pin number is too large"),
                b'2' => unwrap!(intval.checked_add(2), "pin number is too large"),
                b'3' => unwrap!(intval.checked_add(3), "pin number is too large"),
                b'4' => unwrap!(intval.checked_add(4), "pin number is too large"),
                b'5' => unwrap!(intval.checked_add(5), "pin number is too large"),
                b'6' => unwrap!(intval.checked_add(6), "pin number is too large"),
                b'7' => unwrap!(intval.checked_add(7), "pin number is too large"),
                b'8' => unwrap!(intval.checked_add(8), "pin number is too large"),
                b'9' => unwrap!(intval.checked_add(9), "pin number is too large"),
                _ => panic!("pin number contains non-numeric character"),
            };
        }
        intval
    }};
}

/// The PIO pins
pub struct Pio0Pins {
    /// PIO pin A
    pub pin_a: Pin<DynPinId, FunctionPio0, PullDown>,
    /// PIO pin B
    pub pin_b: Pin<DynPinId, FunctionPio0, PullDown>,
    /// PIO pin C
    pub pin_c: Pin<DynPinId, FunctionPio0, PullDown>,
    /// PIO pin D
    pub pin_d: Pin<DynPinId, FunctionPio0, PullDown>,
}

/// The compile-time specified pin set
pub struct PinSet {
    /// The LED pin
    pub led: Pin<DynPinId, FunctionSioOutput, PullDown>,
    /// Pins for PIO 0
    pub pio0: Pio0Pins,
}
impl PinSet {
    /// Gets the pin set from environment
    pub fn from_compile_env(pins: Pins) -> Self {
        /// PIO0 pin 0
        const PIO0_PIN0: u8 = u8_from_compileenv!("WS2812B_PIO0_PIN0", default: "10");
        /// PIO0 pin 1
        const PIO0_PIN1: u8 = u8_from_compileenv!("WS2812B_PIO0_PIN1", default: "11");
        /// PIO0 pin 2
        const PIO0_PIN2: u8 = u8_from_compileenv!("WS2812B_PIO0_PIN2", default: "12");
        /// PIO0 pin 3
        const PIO0_PIN3: u8 = u8_from_compileenv!("WS2812B_PIO0_PIN3", default: "13");
        /// The LED pin
        const GPIO_LED: u8 = u8_from_compileenv!("WS2812B_GPIO_LED", default: "25");

        /// Helper function to configure a dynamic pin
        fn get_pin<T>(pin: &mut Option<Pin<DynPinId, FunctionNull, PullDown>>) -> Pin<DynPinId, T, PullDown>
        where
            T: Function,
        {
            // Configure pin
            let pin = pin.take().expect("pin is already in use");
            pin.try_into_function().unwrap_or_else(|_| panic!("failed to configure pin"))
        }

        // Init self
        let mut pins = Self::index_set(pins);
        let pio0 = Pio0Pins {
            pin_a: get_pin(pins.as_mut().get_mut(PIO0_PIN0 as usize).expect("invalid pin number")),
            pin_b: get_pin(pins.as_mut().get_mut(PIO0_PIN1 as usize).expect("invalid pin number")),
            pin_c: get_pin(pins.as_mut().get_mut(PIO0_PIN2 as usize).expect("invalid pin number")),
            pin_d: get_pin(pins.as_mut().get_mut(PIO0_PIN3 as usize).expect("invalid pin number")),
        };
        Self { led: get_pin(pins.as_mut().get_mut(GPIO_LED as usize).expect("invalid pin number")), pio0 }
    }

    /// Creates an indexed set from the GPIO pins
    #[cfg(feature = "raspberrypi-pico")]
    fn index_set(pins: Pins) -> impl AsMut<[Option<Pin<DynPinId, FunctionNull, PullDown>>]> {
        [
            Some(pins.gpio0.into_dyn_pin()),
            Some(pins.gpio1.into_dyn_pin()),
            Some(pins.gpio2.into_dyn_pin()),
            Some(pins.gpio3.into_dyn_pin()),
            Some(pins.gpio4.into_dyn_pin()),
            Some(pins.gpio5.into_dyn_pin()),
            Some(pins.gpio6.into_dyn_pin()),
            Some(pins.gpio7.into_dyn_pin()),
            Some(pins.gpio8.into_dyn_pin()),
            Some(pins.gpio9.into_dyn_pin()),
            Some(pins.gpio10.into_dyn_pin()),
            Some(pins.gpio11.into_dyn_pin()),
            Some(pins.gpio12.into_dyn_pin()),
            Some(pins.gpio13.into_dyn_pin()),
            Some(pins.gpio14.into_dyn_pin()),
            Some(pins.gpio15.into_dyn_pin()),
            Some(pins.gpio16.into_dyn_pin()),
            Some(pins.gpio17.into_dyn_pin()),
            Some(pins.gpio18.into_dyn_pin()),
            Some(pins.gpio19.into_dyn_pin()),
            Some(pins.gpio20.into_dyn_pin()),
            Some(pins.gpio21.into_dyn_pin()),
            Some(pins.gpio22.into_dyn_pin()),
            None,
            None,
            Some(pins.led.into_dyn_pin()), // 25
            Some(pins.gpio26.into_dyn_pin()),
            Some(pins.gpio27.into_dyn_pin()),
            Some(pins.gpio28.into_dyn_pin()),
            None,
        ]
    }

    /// Creates an indexed set from the GPIO pins
    #[cfg(feature = "seeduino-xiao")]
    fn index_set(pins: Pins) -> impl AsMut<[Option<Pin<DynPinId, FunctionNull, PullDown>>]> {
        [
            Some(pins.a0.into_dyn_pin()),   // 0
            Some(pins.a1.into_dyn_pin()),   // 1
            Some(pins.a2.into_dyn_pin()),   // 2
            Some(pins.a3.into_dyn_pin()),   // 3
            Some(pins.sda.into_dyn_pin()),  // 4
            Some(pins.scl.into_dyn_pin()),  // 5
            Some(pins.tx.into_dyn_pin()),   // 6
            Some(pins.rx.into_dyn_pin()),   // 7
            Some(pins.sck.into_dyn_pin()),  // 8
            Some(pins.miso.into_dyn_pin()), // 9
            Some(pins.mosi.into_dyn_pin()), // 10
            None,
            None,
            None,
            None,
            None,
            Some(pins.led_green.into_dyn_pin()), // 16
            Some(pins.led_red.into_dyn_pin()),   // 17
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(pins.led_blue.into_dyn_pin()), // 25
        ]
    }
}
