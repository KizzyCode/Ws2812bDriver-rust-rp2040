//! A heartbeat task that monitors the application for errors and blinks the LEDs

use crate::{
    board::hal::{
        gpio::{DynPinId, FunctionSioOutput, Pin, PullDown},
        Timer,
    },
    panic::LAST_PANIC,
};
use core::sync::atomic::Ordering::SeqCst;
use embedded_hal::digital::{OutputPin, StatefulOutputPin};
use fugit::MicrosDurationU32;

/// The heartbeat interval
const INTERVAL: MicrosDurationU32 = MicrosDurationU32::millis(500);

/// The heartbeat task
pub async fn task(led: &mut Pin<DynPinId, FunctionSioOutput, PullDown>, timer: &Timer) {
    // Create and await an alarm
    let mut last_blink = timer.get_counter();
    loop {
        // Always yield here to avoid a tight loop
        embedded_runtime_rp2040::spin_once().await;

        // Check if we have a panic
        let last_panic = LAST_PANIC.load(SeqCst);
        if last_panic >= 0 {
            panic!("panic on core {last_panic}");
        }

        // Toggle the LED state if appropriate
        let now = timer.get_counter();
        if now > last_blink + INTERVAL {
            // Blink LED
            match led.is_set_high().expect("failed to get LED state") {
                true => led.set_low().expect("failed to set LED to low"),
                false => led.set_high().expect("failed to set LED to high"),
            };

            // Update the last-blink time
            last_blink = now;
        }
    }
}
