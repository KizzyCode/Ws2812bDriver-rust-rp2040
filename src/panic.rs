//! Implements the panic handler

use crate::strbuffer::StrBuffer;
use core::{
    fmt::Write,
    hint::black_box,
    panic::PanicInfo,
    sync::atomic::{AtomicI8, Ordering::SeqCst},
};
use cortex_m::asm;
use cortex_m_rt::ExceptionFrame;
use rp_pico::hal::Sio;

/// The index of the core with the last panic (or `-1` in case there is no panic)
pub static LAST_PANIC: AtomicI8 = AtomicI8::new(-1);
/// The static panic buffers for each core
#[no_mangle]
pub static mut PANIC_BUFFER: [StrBuffer<512>; 2] = [StrBuffer::new(), StrBuffer::new()];

/// The panic handler
#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Set the panic flag
    let core = Sio::core();
    LAST_PANIC.store(core as i8, SeqCst);

    // Write the panic info into the buffer
    let buffer = unsafe { &mut PANIC_BUFFER[core as usize] };
    let _write_ok = write!(buffer, "{info}").is_ok();
    black_box(buffer);

    // Trigger a breakpoint and raise a fatal exception
    asm::bkpt();
    asm::udf();
}

#[cortex_m_rt::exception]
#[allow(non_snake_case)]
unsafe fn DefaultHandler(irqn: i16) {
    loop {
        asm::bkpt();
        black_box(irqn);
    }
}

#[cortex_m_rt::exception]
#[allow(non_snake_case)]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    loop {
        asm::bkpt();
        black_box(ef);
    }
}
