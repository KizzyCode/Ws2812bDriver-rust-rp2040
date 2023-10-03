//! Flash-related peripheral access

// FIXME: the `rp2040_flash` is currently incompatible with the current HAL version
// use rp2040_flash::flash;
//
// /// Gets the flash UID
// ///
// /// # Safety
// /// Nothing must access flash while this is running. Usually this means:
// /// - interrupts must be disabled
// /// - 2nd core must be running code from RAM or ROM with interrupts disabled
// /// - DMA must not access flash memory
// pub unsafe fn uid() -> (u32, u64) {
//     /// The vendor ID
//     const VENDOR: u32 = const_int_from_compileenv!("WS2812B_UID_VENDOR" => u32, default: "51966"); // 0xCAFE
//     /// The device ID
//     const ID: u64 = const_int_from_compileenv!("WS2812B_UID_ID" => u32, default: "3735928559"); // 0xDEADBEEF
//
//     // Use compile-time vendor and ID if given
//     if VENDOR != 0xCAFE || ID != 0xDEADBEEF {
//         return (VENDOR, ID);
//     }
//
//     // Get the JEDEC ID and prepare buffer
//     let jedec_id = flash::flash_jedec_id(false);
//     let mut flash_uid = [0; 8];
//
//     // Get the UID if the flash chip is from a well-known manufactor with support for UIDs
//     #[allow(clippy::single_match)]
//     match jedec_id {
//         0xEF7015 => flash::flash_unique_id(&mut flash_uid, false),
//         0xEF4015 => flash::flash_unique_id(&mut flash_uid, false),
//         _ => (/* do nothing */),
//     }
//
//     // Combine JEDEC and board UID
//     let flash_uid = u64::from_ne_bytes(flash_uid);
//     (jedec_id, flash_uid)
// }

use crate::const_int_from_compileenv;

/// Gets the flash UID
///
/// # Safety
/// Nothing must access flash while this is running. Usually this means:
/// - interrupts must be disabled
/// - 2nd core must be running code from RAM or ROM with interrupts disabled
/// - DMA must not access flash memory
pub unsafe fn uid() -> (u32, u64) {
    /// The vendor ID
    const VENDOR: u32 = const_int_from_compileenv!("WS2812B_UID_VENDOR" => u32, default: "51966"); // 0xCAFE
    /// The device ID
    const ID: u64 = const_int_from_compileenv!("WS2812B_UID_ID" => u64, default: "3735928559"); // 0xDEADBEEF

    // FIXME: WORKAROUND: This is a no-op fallback since the `rp2040_flash` is currently incompatible with the current HAL
    // version
    (VENDOR, ID)
}
