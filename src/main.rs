#![no_std]
#![no_main]

mod command;
mod hardware;
mod panic;
mod strbuffer;
mod tasks;
mod ws2812b;

// Select the appropriate board
#[cfg(feature = "raspberrypi-pico")]
use rp_pico as board;
#[cfg(feature = "seeduino-xiao")]
use seeeduino_xiao_rp2040 as board;
#[cfg(not(any(feature = "raspberrypi-pico", feature = "seeduino-xiao")))]
compile_error!(concat! {
    "Please select a board using the appropriate crate feature. Supported boards are:\n",
    " - Raspberry Pi Pico: `raspberrypi-pico`\n",
    " - Seeed Studio XIAO RP2040: `seeduino-xiao`\n"
});

use crate::hardware::flash;
use crate::hardware::init::{Core1, Hardware};
use crate::strbuffer::StrBuffer;
use crate::tasks::{heartbeat, serial};
use crate::ws2812b::{Core1Hardware, CORE1_HARDWARE};
use core::fmt::Write;

#[board::entry]
fn main() -> ! {
    // Get the flash UID before doing anything else and build the serial number
    let (jedec_id, flash_uid) = critical_section::with(|_| unsafe { flash::uid() });
    let mut serno: StrBuffer<64> = StrBuffer::new();
    write!(&mut serno, "WS2812B-0001-{jedec_id:08X}-{flash_uid:016X}").expect("failed to build serial number");

    // Initalize the hardware
    let mut hardware = Hardware::init().expect("failed to initialize hardware");
    let core1_hardware =
        Core1Hardware { system_clock: hardware.system_clock, pio0: hardware.pio0, pio0_pins: hardware.pio0_pins };

    // Start core 1
    critical_section::with(|cs| CORE1_HARDWARE.replace(cs, Some(core1_hardware)));
    hardware.core1.start(ws2812b::stack_core1(), ws2812b::write_pio_core1);

    // Get the required peripherals for our main tasks
    let Hardware { timer, usb_bus, mut led, core1, .. } = hardware;
    let Core1 { mut sio_fifo, .. } = core1;

    // Run our tasks
    let result = embedded_runtime_rp2040::run! {
        // Heartbeat task
        heartbeat::task(&mut led, &timer),
        // The main control task
        serial::task(usb_bus, serno, &mut sio_fifo)
    };
    panic!("the executor failed ({result:?})");
}
