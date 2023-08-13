//! Flash-related peripheral access

use rp2040_flash::flash;

/// Gets the flash UID
///
/// # Safety
/// Nothing must access flash while this is running. Usually this means:
/// - interrupts must be disabled
/// - 2nd core must be running code from RAM or ROM with interrupts disabled
/// - DMA must not access flash memory
pub unsafe fn uid() -> (u32, u64) {
    // Get the JEDEC ID and prepare buffer
    let jedec_id = flash::flash_jedec_id(false);
    let mut flash_uid = [0; 8];

    // Get the UID if the flash chip is from a well-known manufactor with support for UIDs
    #[allow(clippy::single_match)]
    match jedec_id {
        0xEF7015 => flash::flash_unique_id(&mut flash_uid, false),
        0xEF4015 => flash::flash_unique_id(&mut flash_uid, false),
        _ => (/* do nothing */),
    }

    // Combine JEDEC and board UID
    let flash_uid = u64::from_ne_bytes(flash_uid);
    (jedec_id, flash_uid)
}
