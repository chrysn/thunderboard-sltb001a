#![no_std]

extern crate efm32gg990;
extern crate embedded_hal;
extern crate efm32gg_hal;

pub mod led;
pub mod button;

use efm32gg_hal::gpio::GPIOExt;

pub struct Board {
    pub leds: led::LEDs,
    pub buttons: button::Buttons,
}

pub fn init() -> Board {
    let p = efm32gg990::Peripherals::take().unwrap();

    let mut cmu = p.CMU;

    let gpios = p.GPIO.split(&mut cmu);

    let leds = led::LEDs::new(gpios.pe2, gpios.pe3);

    let buttons = button::Buttons::new(gpios.pb9, gpios.pb10);

    Board {
        leds: leds,
        buttons: buttons,
    }
}
