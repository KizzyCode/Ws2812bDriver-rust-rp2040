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


## Configuration
To adjust the GPIO pins, you can set the following environment variables during compilation to the appropriate pin
numbers:
 - `WS2812B_PIO0_PIN0`: The control pin for the first LED strip (defaults to `10`)
 - `WS2812B_PIO0_PIN1`: The control pin for the second LED strip (defaults to `11`)
 - `WS2812B_PIO0_PIN2`: The control pin for the third LED strip (defaults to `12`)
 - `WS2812B_PIO0_PIN3`: The control pin for the fourth LED strip (defaults to `13`)
 - `WS2812B_GPIO_LED`: The control pin for the status LED (defaults to `25`)
  
To adjust the USB serial number, you can set the following environment variables during compilation:
 - `WS2812B_UID_VENDOR`: The vendor ID (defaults to the JEDEC vendor ID of the connected flash chip)
 - `WS2812B_UID_ID`: The vendor ID (defaults to the maybe-unique fabrication ID of the connected flash chip)

**IMPORTANT**: Please keep in mind that most WS2812B LED strips require 5V, whereas the RP2040's GPIOs are 3V3, which may
not be enough or can cause weird errors.

## TODO:
 - [ ] Batch command format to improve state-change performance
 - [ ] Maybe batch/ring IPC between core 0 and 1 instead of SIO FIFO
 - [ ] Update only changed strips/skip strips without change
 - [ ] Select board constant instead of LED PIN
   - [ ] Disable unused LEDs on Seeduino XIAO RP2040