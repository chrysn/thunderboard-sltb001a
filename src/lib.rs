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
pub struct Board<D1, D2>
    where D1: embedded_hal::blocking::delay::DelayMs<u16>,
          D2: embedded_hal::blocking::delay::DelayUs<u16>,
{
    pub leds: led::LEDs,
    pub buttons: button::Buttons,
    pub delay: D1,
    pub pic: pic::PIC<D2>,
}

impl Board<VeryBadDelay, efm32gg_hal::systick::SystickDelay> {
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
        let delay = efm32gg_hal::systick::SystickDelay::new(syst, hfcoreclk);

        // At board initialization, it makes sense to clear the LEDs because the EFM8 is not reset
        // along with the EFR32. (Would make sense to clear everything else too once enabled, or to
        // find a SYS_CMD that resets the chip as a whole, see
        // <https://www.silabs.com/community/thunderboard/forum.topic.html/thunderboard_reset-6Agl>).
        let mut pic = pic::PIC::new(p.I2C0, delay, cmu.i2c0, gpios.pd10, gpios.pc11, gpios.pc10);
        pic.set_leds(false, false, false, false);
        let id = pic.read_device_id();
        assert!(&id == &[0x49, 0x4f, 0x58, 0x50], "PIC device ID unexpected");

        // If we wanted to pass out the good delay, we could do this and not pass out the PIC.
        // let (i2c, delay) = pic.destroy();

        Board {
            leds: leds,
            buttons: buttons,
            delay: VeryBadDelay { },
            pic: pic,
        }
    }
}

use embedded_hal::blocking::delay::{DelayUs, DelayMs};
pub struct VeryBadDelay {
}

impl DelayUs<u16> for VeryBadDelay {
    // This is only accurate to an order of magnitude, and only in release builds and for
    // unmodified clock settings...
    fn delay_us(&mut self, us: u16) {
        for _i in 1..us/10 {
            cortex_m::asm::nop();
        }
    }
}

impl DelayMs<u16> for VeryBadDelay {
    fn delay_ms(&mut self, ms: u16) {
        for _i in 1..ms {
            self.delay_us(1000)
        }
    }
}
