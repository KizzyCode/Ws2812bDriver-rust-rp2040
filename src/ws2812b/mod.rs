//! Implements the PIO logic on core 1

mod pio;

use crate::{
    command::Command,
    hardware::init::{Pio0, Pio0Pins},
    ws2812b::pio::PioTx,
};
use core::{cell::RefCell, hint};
use critical_section::Mutex;
use rp_pico::{
    hal::{clocks::SystemClock, multicore::Stack, Sio},
    pac::Peripherals,
};

/// The state matrix of an LED strip
type StripState<const SIZE: usize> = [Option<(u8, u8, u8)>; SIZE];

/// The required hardware for core 1
pub struct Core1Hardware {
    /// The system clock
    pub system_clock: SystemClock,
    /// The PIO0 peripheral
    pub pio0: Pio0,
    /// The PIO0 associated pins
    pub pio0_pins: Pio0Pins,
}
/// The hardware required by core 1
pub static CORE1_HARDWARE: Mutex<RefCell<Option<Core1Hardware>>> = Mutex::new(RefCell::new(None));

/// Returns the stack for core 1
pub fn stack_core1() -> &'static mut [usize; 1024 * 12] {
    /// The stack for the core 1 (48 KiB)
    static mut STACK: Stack<{ 1024 * 12 }> = Stack::new();
    unsafe { &mut STACK.mem }
}

/// A tight runloop that checks the inter-core FIFO for pixel changes and syncs the new state to the PIO
///
/// # Important
/// This runloop is blocking and designed to run on another core exclusively (i.e. core 1)
pub fn write_pio_core1() -> ! {
    // Steal the SIO FIFO and get the shared hardware
    // This should hopefully be safe since the SIO FIFO is explicitely designed for inter-core communication
    let Peripherals { SIO, .. } = unsafe { Peripherals::steal() };
    let Sio { mut fifo, .. } = Sio::new(SIO);
    let Core1Hardware { system_clock, pio0, pio0_pins } =
        critical_section::with(|cs| CORE1_HARDWARE.take(cs)).expect("missing required hardware instances for core 1");

    // Init states and setup state machines
    let mut states: [StripState<512>; 4] = [[None; 512]; 4];
    let (mut _tx0, mut _tx1, mut _tx2, mut _tx3) = pio::setup(pio0, pio0_pins, &system_clock);
    let pio_tx: [&mut dyn PioTx; 4] = [&mut _tx0, &mut _tx1, &mut _tx2, &mut _tx3];

    // Loop forever to process the incoming state
    loop {
        // Wait until we receive an update
        while !fifo.is_read_ready() {
            hint::spin_loop();
        }

        // Update the state (the FIFO can store at max 8 entries, so we limit ourself to 8 to avoid stalling the PIO)
        'read_fifo: for _ in 0..8 {
            // Read the next packed update
            let Some(packed) = fifo.read() else {
                break 'read_fifo;
            };

            // Update the state
            let Command { strip, pixel, rgb } = Command::from_u32(packed);
            states[strip][pixel] = Some(rgb);
        }

        // Sync to all PIOs
        'write_strip: for strip in 0..states.len() {
            for pixel in 0..states[strip].len() {
                // Skip the strip if the pixel index is beyond end-of-strip
                let Some((r, g, b)) = states[strip][pixel] else {
                    continue 'write_strip;
                };

                // Write the pixel
                // Note: This "weird" encoding is intentional since the LED strip/state machine requires this order
                let grb = ((g as u32) << 24) | ((r as u32) << 16) | ((b as u32) << 8);
                while !pio_tx[strip].write(grb) {
                    hint::spin_loop();
                }
            }
        }
    }
}
