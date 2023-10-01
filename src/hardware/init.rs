//! Initializes all required hardware modules

use rp_pico::{
    hal::{
        clocks::{self, ClocksManager, SystemClock},
        gpio::{bank0::*, FunctionPio0, Output, Pin, PushPull},
        multicore::Multicore,
        pio::{PIOExt, UninitStateMachine, PIO, SM0, SM1, SM2, SM3},
        sio::SioFifo,
        usb::UsbBus,
        Sio, Timer, Watchdog,
    },
    pac::{Peripherals, PIO0, PPB, PSM, RESETS},
    Pins, XOSC_CRYSTAL_FREQ,
};

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
    pub fn start(&mut self, stack: &'static mut [usize], entry: fn() -> !) {
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

/// The PIO pins
pub struct Pio0Pins {
    /// PIO pin A
    pub pin_a: Pin<Gpio10, FunctionPio0>,
    /// PIO pin B
    pub pin_b: Pin<Gpio11, FunctionPio0>,
    /// PIO pin C
    pub pin_c: Pin<Gpio12, FunctionPio0>,
    /// PIO pin D
    pub pin_d: Pin<Gpio13, FunctionPio0>,
}

/// The underlying basic hardware
pub struct Hardware {
    /// The watchdog peripheral
    pub watchdog: Watchdog,
    /// The system clock
    pub system_clock: SystemClock,
    /// The LED pin
    pub led: Pin<Gpio25, Output<PushPull>>,
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
        let ClocksManager { system_clock, usb_clock, .. } =
            clocks::init_clocks_and_plls(XOSC_CRYSTAL_FREQ, XOSC, CLOCKS, PLL_SYS, PLL_USB, &mut RESETS, &mut watchdog)
                .unwrap_or_else(|_| panic!("Failed to initialize clocks"));

        // Create basic IO
        let sio = Sio::new(SIO);
        let gpio_bank0 = sio.gpio_bank0;
        let pins = Pins::new(IO_BANK0, PADS_BANK0, gpio_bank0, &mut RESETS);
        let led = pins.led.into_push_pull_output();

        // Create PIO pins
        let pio0_pins = Pio0Pins {
            pin_a: pins.gpio10.into_mode(),
            pin_b: pins.gpio11.into_mode(),
            pin_c: pins.gpio12.into_mode(),
            pin_d: pins.gpio13.into_mode(),
        };

        // Init self
        Some(Self {
            watchdog,
            system_clock,
            led,
            timer: Timer::new(TIMER, &mut RESETS),
            usb_bus: UsbBus::new(USBCTRL_REGS, USBCTRL_DPRAM, usb_clock, true, &mut RESETS),
            core1: Core1::new(PPB, PSM, sio.fifo),
            pio0: Pio0::new(PIO0, &mut RESETS),
            pio0_pins,
        })
    }
}
