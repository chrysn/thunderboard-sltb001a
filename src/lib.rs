//! This is a board support crate for the [EFM32GG-STK3700] Starter Kit.
//!
//! State and quick-start
//! ---------------------
//!
//! Right now, it is very minimal; LEDs and buttons are exposed in a board struct, but that's about
//! it. It will grow as the underlying [HAL implementation] does.
//!
//! Thus, the larger value is in providing runnable examples. With an STK connected via USB and an
//! [OpenOCD] running (typically ``openocd -f board/efm32.cfg``), a blinking example can be run as
//! ``cargo +nightly run --example blink --features="depend-panic-semihosting depend-cortex-m-rt"``.
//!
//! (The features the example needs are actually additional dependencies and are shown in the error
//! message when invoking an example without features).
//!
//! Usage
//! -----
//!
//! See the examples provided (``blink`` is a good start) to get familiar with how the abstract
//! peripherals can be obtained. The usual way is to utilize the board init function, but the main
//! structs of the individual modules can be initialized on their own instead just as well.
//!
//! Noteworthy features
//! -------------------
//!
//! The cargo configuration that enables ``cargo run`` to work is a little more elaborate than that
//! of the [f3] crate that provided me with much guidance: It contains a small gdb-wrapper script
//! in the .cargo directory that detects any usable gdb (might be ``arm-none-eabi-gdb`` or
//! ``gdb-multiarch``), and passes the initial setup commands on the command line rather than using
//! a .gdbinit (because the latter requires [safe-path configuration]).
//!
//! [EFM32GG-STK3700]: https://www.silabs.com/products/development-tools/mcu/32-bit/efm32-giant-gecko-starter-kit
//! [HAL implementation]: https://github.com/chrysn/efm32gg-hal
//! [OpenOCD]: http://openocd.org/
//! [f3]: https://crates.io/crates/f3
//! [safe-path configuration]: https://sourceware.org/gdb/onlinedocs/gdb/Auto_002dloading-safe-path.html#Auto_002dloading-safe-path

#![no_std]

extern crate cortex_m;
extern crate embedded_hal;
extern crate efm32gg_hal;

extern crate efr32xg1;

pub mod led;
pub mod button;
pub mod pic;

use efm32gg_hal::{
    gpio::GPIOExt,
    cmu::CMUExt,
    systick::SystickExt,
};

/// A representation of all the board's peripherals.
///
/// While all its parts can be easily constructed on their own, instanciating the full board takes
/// care of obtaining the low-level peripherals and moving the right pins to the right devices.
pub struct Board {
    pub leds: led::LEDs,
    pub buttons: button::Buttons,
    pub delay: efm32gg_hal::systick::SystickDelay,
    pub pic: pic::PIC,
}

impl Board {
    /// Initialize the board
    ///
    /// This does little configuration, but primarily ``take``s the system and EFM32 peripherals and
    /// distributes them to the the suitable abstractions for the board.
    ///
    /// Peripherals that are not part of the defined board are lost when the structs are taken apart.
    /// The current recommendation for composite devices (ie. "The STK3700 with something actually
    /// connected to the extension header or breakoutp pins") is to not use this function but rather
    /// look at its code, replicate what is needed and add in the composite board's additional devices
    /// in a new board initialization function. The author is open to suggestions as to how that would
    /// be done better.
    pub fn new() -> Self {
        let corep = cortex_m::peripheral::Peripherals::take().unwrap();
        let p = efr32xg1::Peripherals::take().unwrap();

        let cmu = p.CMU.constrain().split();

        let gpios = p.GPIO.split(cmu.gpio);

        let leds = led::LEDs::new(gpios.pd11, gpios.pd12);

        let buttons = button::Buttons::new(gpios.pd14, gpios.pd15);

        let hfcoreclk = cmu.hfcoreclk;
        let syst = corep.SYST.constrain();
        let mut delay = efm32gg_hal::systick::SystickDelay::new(syst, hfcoreclk);

        let pic = pic::PIC::new(p.I2C0, &mut delay, cmu.i2c0, gpios.pd10, gpios.pc11, gpios.pc10);

        Board {
            leds: leds,
            buttons: buttons,
            delay: delay,
            pic: pic,
        }
    }
}
