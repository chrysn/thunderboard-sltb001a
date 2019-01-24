//! This example uses no board support crate yet, but is the developer's base line for testing.
//!
//! It does nothing, just idles in an infinite loop.

#![no_main]
#![no_std]

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    loop {}
}

// For any non-minimal demo, and especially during development, you'll likely rather use this
// crate and remove everything below here:
//
// extern crate panic_semihosting;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
