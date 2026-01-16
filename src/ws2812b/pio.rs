//! The PIO assembly for WS2812B

use crate::{
    board::{
        hal::{
            clocks::SystemClock,
            pio::{PIOBuilder, PinDir, ShiftDirection, Tx, SM0, SM1, SM2, SM3},
            Clock,
        },
        pac::PIO0,
    },
    hardware::{init::Pio0, pins::Pio0Pins},
};
use pio::{Program, RP2040_MAX_PROGRAM_SIZE};

/// A PIO TX pin
pub trait PioTx {
    /// Writes a value to the TX FIFO handle
    fn write(&mut self, value: u32) -> bool;
}
/// Implements `PioTx` for the given type
macro_rules! impl_pio_tx {
    ($type:ty) => {
        impl PioTx for $type {
            fn write(&mut self, value: u32) -> bool {
                self.write(value)
            }
        }
    };
}
impl_pio_tx!(Tx<(PIO0, SM0)>);
impl_pio_tx!(Tx<(PIO0, SM1)>);
impl_pio_tx!(Tx<(PIO0, SM2)>);
impl_pio_tx!(Tx<(PIO0, SM3)>);

/// Deploys the assembly code to the PIO
///
/// # Safety
/// The deployed program must not be uninstalled until all state machines have been stopped manually.
#[allow(clippy::type_complexity)]
pub fn setup(
    pio0: Pio0,
    pio0_pins: Pio0Pins,
    system_clock: &SystemClock,
) -> (Tx<(PIO0, SM0)>, Tx<(PIO0, SM1)>, Tx<(PIO0, SM2)>, Tx<(PIO0, SM3)>) {
    /// The WS2812B frequency (800 kHz)
    const WS2812B_FREQUENCY: u32 = 800_000;
    /// The amount of PIO clock cycles per control bit
    const CYCLES_PER_CONTROL_BIT: u32 = 10;

    // Compute clock frequency
    let clock_frequency = system_clock.freq().to_Hz();
    let target_frequency = WS2812B_FREQUENCY * CYCLES_PER_CONTROL_BIT;
    let (frequency_int, frequency_frac) = frequency(clock_frequency, target_frequency);

    // Install the programm
    let Pio0 { mut pio, sm0, sm1, sm2, sm3 } = pio0;
    let program = program();
    let installed = pio.install(&program).expect("failed to install program");

    // Setup the state machines
    macro_rules! setup_statemachine {
        ($sm:expr => $pin:expr) => {{
            // Setup state machine
            let (mut sm, _, tx) = PIOBuilder::from_installed_program(unsafe { installed.share() })
                .side_set_pin_base($pin.id().num)
                .out_shift_direction(ShiftDirection::Left)
                .autopull(true)
                .pull_threshold(24)
                .clock_divisor_fixed_point(frequency_int, frequency_frac)
                .build($sm);

            // Set pin direction
            sm.set_pindirs([($pin.id().num, PinDir::Output)]);
            sm.start();
            tx
        }};
    }

    // Create the state machine tuple
    (
        setup_statemachine!(sm0 => pio0_pins.pin_a),
        setup_statemachine!(sm1 => pio0_pins.pin_b),
        setup_statemachine!(sm2 => pio0_pins.pin_c),
        setup_statemachine!(sm3 => pio0_pins.pin_d),
    )
}

/// Computes the clock frequency
const fn frequency(clock_frequency: u32, target_frequency: u32) -> (u16, u8) {
    // Compute frequency
    let int = clock_frequency / target_frequency;
    let rem = clock_frequency - (int * target_frequency);
    let frac = (rem * 256) / target_frequency;

    // 65536.0 is represented as 0 in the PIO's clock divider
    match int {
        65536 => (0, frac as u8),
        _ => (int as u16, frac as u8),
    }
}

/// The WS2812B program
fn program() -> Program<{ RP2040_MAX_PROGRAM_SIZE }> {
    let compiled = pio_proc::pio_asm! {
        ".side_set 1",
        ".wrap_target",
        // Bitloop
        "bitloop:",
        "   out x  1       side 0 [3 - 1]",
        "   jmp !x do_zero side 1 [2 - 1]",
        "   jmp    bitloop side 1 [5 - 1]",
        // Do zero
        "do_zero:",
        "   nop            side 0 [5 - 1]",
        ".wrap"
    };
    compiled.program
}
