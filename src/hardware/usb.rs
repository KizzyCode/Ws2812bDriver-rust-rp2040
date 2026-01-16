//! Implements USB-serial futures

use crate::board::hal::usb::UsbBus;
use crate::strbuffer::StrBuffer;
use core::cell::OnceCell;
use core::marker::PhantomData;
use static_cell::StaticCell;
use usb_device::class_prelude::UsbBusAllocator;
use usb_device::device::StringDescriptors;
use usb_device::prelude::{UsbDevice, UsbDeviceBuilder, UsbVidPid};
use usb_device::LangID;
use usb_device::UsbError::WouldBlock;
use usbd_serial::SerialPort;

/// The USB vendor ID
const VID: (u16, u16) = (0x16c0, 0x27dd);
/// The USB manufacturer
const MANUFACTURER: &str = "KizzyCode Software Labs./Keziah Biermann";
/// The USB product
const PRODUCT: &str = "WS2812B LED Driver";
/// The USB device class
const CLASS: u8 = 2;

/// A USB serial device
pub struct UsbSerialDevice {
    /// The USB device itself
    device: UsbDevice<'static, UsbBus>,
    /// The USB device as serial device
    serial: SerialPort<'static, UsbBus>,
    /// Deny send and sync
    _nosendsync: PhantomData<*const OnceCell<(UsbBusAllocator<UsbBus>, StrBuffer<64>)>>,
}
impl UsbSerialDevice {
    /// Creates a new USB serial device on the given USB bus
    pub fn new(usb_bus: UsbBus, serno: StrBuffer<64>) -> Self {
        /// The USB allocator
        ///
        /// # Safety
        /// This entire block should be sound since this function is implicitly once because the function argument
        /// `usb_bus` is a singleton, so this function cannot be called simultaneously from another core or interrupt
        /// handler. Furthermore, the resulting object is `!Send` and `!Sync`, so there should be no inter-core race
        /// conditions when reading the static reference.
        static USB_ALLOCATOR: StaticCell<UsbBusAllocator<UsbBus>> = StaticCell::new();
        let allocator = UsbBusAllocator::new(usb_bus);
        let allocator = USB_ALLOCATOR.init(allocator);

        /// The serial number
        ///
        /// # Safety
        /// This entire block should be sound since this function is implicitly once because the function argument
        /// `usb_bus` is a singleton, so this function cannot be called simultaneously from another core or interrupt
        /// handler. Furthermore, the resulting object is `!Send` and `!Sync`, so there should be no inter-core race
        /// conditions when reading the static reference.
        static SERNO: StaticCell<StrBuffer<64>> = StaticCell::new();
        let serno = SERNO.init(serno);

        // Initialize the USB device
        let vid_pid = UsbVidPid(VID.0, VID.1);
        let serial = SerialPort::new(allocator);
        let descriptors =
            StringDescriptors::new(LangID::DE).serial_number(serno).manufacturer(MANUFACTURER).product(PRODUCT);
        let device = UsbDeviceBuilder::new(allocator, vid_pid)
            // Set identifiers
            .strings(&[descriptors]).expect("failed to set descriptors")
            // Mark as serial device
            .device_class(CLASS)
            .build();

        Self { device, serial, _nosendsync: PhantomData }
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
            embedded_runtime_rp2040::spin_once().await;
            self.poll();

            // Read the next byte if available, otherwise try again
            match self.serial.read(&mut buf[..1]) {
                Ok(_) => buf.rotate_left(1),
                Err(WouldBlock) => continue 'read_loop,
                Err(e) => panic!("failed to read from USB device ({e:?})"),
            }
        }
    }

    /// Writes the entire buffer
    pub async fn write_all(&mut self, buf: &[u8]) {
        // Write the entire buffer
        let mut buf_pos = 0;
        while buf_pos < buf.len() {
            // Always yield here to avoid a tight loop
            embedded_runtime_rp2040::spin_once().await;
            self.poll();

            // Write the next data
            buf_pos += match self.serial.write(&buf[buf_pos..]) {
                Ok(len) => len,
                Err(WouldBlock) => 0,
                Err(e) => panic!("failed to write to USB device ({e:?})"),
            };
        }
    }
}
