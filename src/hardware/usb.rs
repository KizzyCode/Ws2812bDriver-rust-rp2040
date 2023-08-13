//! Implements USB-serial futures

use crate::strbuffer::StrBuffer;
use rp_pico::hal::usb::UsbBus;
use usb_device::{
    class_prelude::UsbBusAllocator,
    prelude::{UsbDevice, UsbDeviceBuilder, UsbVidPid},
    UsbError::WouldBlock,
};
use usbd_serial::SerialPort;

/// The USB vendor ID
const VID: (u16, u16) = (0x16c0, 0x27dd);
/// The USB manufacturer
const MANUFACTURER: &str = "KizzyCode Software Labs./Keziah Biermann";
/// The USB product
const PRODUCT: &str = "WS2812B LED Driver";
/// The USB device class
const CLASS: u8 = 2;

/// The USB allocator
/// This is ugly but sadly necessary since `SerialPort` and `UsbDeviceBuilder` both require a reference
static mut USB_ALLOCATOR: Option<UsbBusAllocator<UsbBus>> = None;
/// The serial number
/// This is ugly but sadly necessary since `UsbDeviceBuilder` requires a reference
static mut SERNO: Option<StrBuffer<64>> = None;

/// A USB serial device
pub struct UsbSerialDevice {
    /// The USB device itself
    device: UsbDevice<'static, UsbBus>,
    /// The USB device as serial device
    serial: SerialPort<'static, UsbBus>,
}
impl UsbSerialDevice {
    /// Creates a new USB serial device on the given USB bus
    pub fn new(usb_bus: UsbBus, serno: StrBuffer<64>) -> Self {
        // Move the allocator and serial number into a static var and reference it
        // Note: the entire block should be sound since this function can only be called once since `usb_bus` is a
        // singleton
        let allocator = UsbBusAllocator::new(usb_bus);
        let allocator = unsafe {
            USB_ALLOCATOR = Some(allocator);
            USB_ALLOCATOR.as_ref().expect("failed to access USB allocator")
        };
        let serno = unsafe {
            SERNO = Some(serno);
            SERNO.as_ref().expect("failed to access serial number")
        };

        // Initialize the USB device
        let vid_pid = UsbVidPid(VID.0, VID.1);
        let serial = SerialPort::new(allocator);
        let device = UsbDeviceBuilder::new(allocator, vid_pid)
            .serial_number(serno)
            .manufacturer(MANUFACTURER)
            .product(PRODUCT)
            .device_class(CLASS)
            .build();

        Self { device, serial }
    }

    /// Polls the USB devices
    ///
    /// # Important
    /// To be standard-compliant, the bus must be polled at least every 10ms.
    pub fn poll(&mut self) {
        // Note: We don't propagate the result of `device.poll` since it is not reliable and may return false even if some
        // progress can be made`
        self.device.poll(&mut [&mut self.serial]);
    }

    /// Reads some data from the USB bus
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        match self.serial.read(buf) {
            Ok(len) => len,
            Err(WouldBlock) => 0,
            Err(e) => panic!("failed to read from USB device ({e:?})"),
        }
    }
    /// Reads one byte from the USB bus
    pub fn read_one(&mut self) -> Option<u8> {
        let mut buf = [0];
        match self.read(&mut buf) {
            1 => Some(buf[0]),
            _ => None,
        }
    }

    /// Writes some data to the USB bus
    pub fn write(&mut self, data: &[u8]) -> usize {
        match self.serial.write(data) {
            Ok(len) => len,
            Err(WouldBlock) => 0,
            Err(e) => panic!("failed to write to USB device ({e:?})"),
        }
    }

    /// Performs an opportunistic USB device reset
    #[cfg(feature = "bootsel")]
    pub fn try_reset(&mut self) {
        let _ = self.device.force_reset();
    }

    /// Reads byte per byte until the `condition` evaluates to true
    pub async fn read_until<F>(&mut self, buf: &mut [u8], mut condition: F)
    where
        F: FnMut(&[u8]) -> bool,
    {
        'read_loop: while !condition(buf) {
            // Always yield here to avoid a tight loop
            embedded_runtime_rp2040::yield_now().await;
            self.poll();

            // Read the next byte if available, otherwise try again
            let Some(next) = self.read_one() else {
                continue 'read_loop;
            };

            // Append byte
            buf[0] = next;
            buf.rotate_left(1);
        }
    }

    /// Writes the entire buffer
    pub async fn write_all(&mut self, buf: &[u8]) {
        // Write the entire buffer
        let mut buf_pos = 0;
        while buf_pos < buf.len() {
            // Always yield here to avoid a tight loop
            embedded_runtime_rp2040::yield_now().await;
            self.poll();

            // Write the next data
            buf_pos += self.write(&buf[buf_pos..]);
        }
    }
}
