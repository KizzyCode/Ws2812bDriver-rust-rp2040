//! A main task that reads update commands from the serial interface and applies them

use crate::board::hal::sio::SioFifo;
use crate::board::hal::usb::UsbBus;
use crate::command::Command;
use crate::hardware::usb::UsbSerialDevice;
use crate::strbuffer::StrBuffer;

/// A main task that reads update commands from the serial interface and applies them
pub async fn task(usb_bus: UsbBus, serno: StrBuffer<64>, sio_fifo: &mut SioFifo) {
    // Read incoming commands and forward them to the second core
    let mut serial = UsbSerialDevice::new(usb_bus, serno);
    'message_loop: loop {
        // Try to read the next command line
        let mut buf = [0; Command::SERIAL_LEN];
        serial.read_until(&mut buf, |buf| buf.ends_with(b"\n")).await;

        // Check for bootsel message, reset if appropriate
        #[cfg(feature = "bootsel")]
        if buf == *b"RESET_TO_BOOTSEL\n" {
            // Disconnect USB and reset the pico
            serial.try_reset();
            crate::board::hal::rom_data::reset_to_usb_boot(0, 0);
        }

        // Parse the update or drop the message if the update is invalid
        let Some(update) = Command::from_serial(&buf) else {
            // Restart the message loop
            continue 'message_loop;
        };

        // Wait until the SIO FIFO has some available space
        while !sio_fifo.is_write_ready() {
            // Always yield here to avoid a tight loop
            embedded_runtime_rp2040::spin_once().await;
        }

        // Send the update to the other core and reflect the message to indicate success
        let update = update.to_u32();
        sio_fifo.write(update);
        serial.write_all(&buf).await;
    }
}
