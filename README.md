[![License BSD-2-Clause](https://img.shields.io/badge/License-BSD--2--Clause-blue.svg)](https://opensource.org/licenses/BSD-2-Clause)
[![License MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)


# `ws2812b-driver`
Welcome to `ws2812b-driver` 🎉

This crate provides a RPi Pico driver firmware to control up to 4 WS2812B LED strips simultaneously. It communicates via
USB-serial (CDC), and controls the WS2812B strips via the RP2040's PIOs to ensure accurate timings.


## Serial Communication
The USB-serial communication is a simple request-response format, where the host sends a single command at a time to the
driver, and if the command was processed successfully, the driver sends it back to the server as-is.

### Command Format
Each command consists of a big-endian hex-encoded 64 bit integer, terminated by a newline (indices are bit offsets):
- `[0, 16)`: The index of the LED strip; must be a number within `[0, 4)` (`strip as u16 << 48`)
- `[16, 32)`: The index of the LED pixel within the strip; must be a number within `[0, 512)` (`pixel as u16 << 32`)
- `[32, 40)`: The RGBW red value; must be a number within `[0, 256)` (`red as u8 << 24`)
- `[40, 48)`: The RGBW green value; must be a number within `[0, 256)` (`green as u8 << 16`)
- `[48, 56)`: The RGBW blue value; must be a number within `[0, 256)` (`blue as u8 << 8`)
- `[56, 64)`: The RGBW white value; this value is currently unsupported and must be `0`

#### Rust Example
```rust
// This example creates a message to set strip 2, LED 17 to RGB 255,255,255
let message = format!("{:04x}{:04x}{:02x}{:02x}{:02x}00\n", 2, 17, 255, 255, 255);
assert_eq!(message, "00020011ffffff00\n");
```

#### Shell Example
```sh
# This example creates a message to set strip 2, LED 17 to RGB 255,255,255
printf "%04x%04x%02x%02x%02x00\n" 2 17 255 255 255
```

#### `RESET_TO_BOOTSEL\n` command
The `bootsel`-feature (enabled by default) allows you to reboot the Pico into USB bootloader mode by sending the serial
command `RESET_TO_BOOTSEL\n`. This will reboot the Pico into UF2 update mode so that you can flash another firmware
without the need for user interaction/physical presence.


## GPIO pins
The currently preselected GPIO pins are `10`, `11`, `12`, `13`. To adjust the GPIO pins, you can change them in
`src/hardware/init`.

**IMPORTANT**: Please keep in mind that most WS2812B LED strips require 5V, whereas the Pico's GPIOs are 3V3, which may
not be enough or can cause weird errors.
