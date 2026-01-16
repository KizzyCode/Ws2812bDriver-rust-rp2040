//! Initializes all required hardware modules

use crate::board::hal::clocks::{self, ClocksManager, SystemClock};
use crate::board::hal::gpio::{DynPinId, FunctionSioOutput, Pin, PullDown};
use crate::board::hal::multicore::Multicore;
use crate::board::hal::pio::{PIOExt, UninitStateMachine, PIO, SM0, SM1, SM2, SM3};
use crate::board::hal::sio::SioFifo;
use crate::board::hal::usb::UsbBus;
use crate::board::hal::{Sio, Timer, Watchdog};
use crate::board::pac::{Peripherals, PIO0, PPB, PSM, RESETS};
use crate::board::{Pins, XOSC_CRYSTAL_FREQ};
use crate::hardware::pins::{PinSet, Pio0Pins};

/// The hardware peripherals to start core 1
pub struct Core1 {
    /// The PPB (required for multicore setup)
    ppb: PPB,
    /// The PSM (required for multicore setup)
    psm: PSM,
    /// The inter-core FIFO (required for multicore setup)
    pub sio_fifo: SioFifo,
}
impl Core1 {
    /// Wraps the hardware peripherals to start core 1
    pub const fn new(ppb: PPB, psm: PSM, sio_fifo: SioFifo) -> Self {
        Self { ppb, psm, sio_fifo }
    }

    /// Starts core 1
    pub fn start(&mut self, stack: &'static mut [usize], entry: fn() -> ()) {
        let mut multicore = Multicore::new(&mut self.psm, &mut self.ppb, &mut self.sio_fifo);
        let cores = multicore.cores();
        cores[1].spawn(stack, entry).expect("failed to start core 1");
    }
}

/// The PIO 0 engine
pub struct Pio0 {
    /// The PIO hardware
    pub pio: PIO<PIO0>,
    /// The unitialized state machine 0
    pub sm0: UninitStateMachine<(PIO0, SM0)>,
    /// The unitialized state machine 1
    pub sm1: UninitStateMachine<(PIO0, SM1)>,
    /// The unitialized state machine 2
    pub sm2: UninitStateMachine<(PIO0, SM2)>,
    /// The unitialized state machine 3
    pub sm3: UninitStateMachine<(PIO0, SM3)>,
}
impl Pio0 {
    /// Creates a new PIO0 adapter from the given peripheral
    pub fn new(pio0: PIO0, resets: &mut RESETS) -> Self {
        let (pio, sm0, sm1, sm2, sm3) = pio0.split(resets);
        Self { pio, sm0, sm1, sm2, sm3 }
    }
}

/// The underlying basic hardware
pub struct Hardware {
    /// The system clock
    pub system_clock: SystemClock,
    /// The LED pin
    pub led: Pin<DynPinId, FunctionSioOutput, PullDown>,
    /// The timer peripherals
    pub timer: Timer,
    /// The USB bus
    pub usb_bus: UsbBus,
    /// The hardware peripherals to start core 1
    pub core1: Core1,
    /// The PIO0 peripheral
    pub pio0: Pio0,
    /// The PIO0 associated pins
    pub pio0_pins: Pio0Pins,
}
impl Hardware {
    /// Initializes the required hardware
    pub fn init() -> Option<Self> {
        // Get the required peripherals
        let Peripherals {
            CLOCKS,
            IO_BANK0,
            PADS_BANK0,
            PIO0,
            PLL_SYS,
            PLL_USB,
            PPB,
            PSM,
            mut RESETS,
            SIO,
            TIMER,
            USBCTRL_DPRAM,
            USBCTRL_REGS,
            WATCHDOG,
            XOSC,
            ..
        } = Peripherals::take()?;

        // Create watchdog and init clocks (this is important for all peripherals and should be done always)
        let mut watchdog = Watchdog::new(WATCHDOG);
        let clocks =
            clocks::init_clocks_and_plls(XOSC_CRYSTAL_FREQ, XOSC, CLOCKS, PLL_SYS, PLL_USB, &mut RESETS, &mut watchdog)
                .unwrap_or_else(|_| panic!("Failed to initialize clocks"));

        // Create timer and take system and USB clock
        let timer = Timer::new(TIMER, &mut RESETS, &clocks);
        let ClocksManager { system_clock, usb_clock, .. } = clocks;

        // Create basic IO and get pin set from compile time environment
        let sio = Sio::new(SIO);
        let pins = Pins::new(IO_BANK0, PADS_BANK0, sio.gpio_bank0, &mut RESETS);
        let pin_set = PinSet::from_compile_env(pins);

        // Init self
        Some(Self {
            system_clock,
            led: pin_set.led,
            timer,
            usb_bus: UsbBus::new(USBCTRL_REGS, USBCTRL_DPRAM, usb_clock, true, &mut RESETS),
            core1: Core1::new(PPB, PSM, sio.fifo),
            pio0: Pio0::new(PIO0, &mut RESETS),
            pio0_pins: pin_set.pio0,
        })
    }
}
