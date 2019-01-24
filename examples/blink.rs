//! Blink an LED while running the other one continuously, while monitoring the buttons to change
//! the behavior.
//!
//! The example runs in a tight loop and only occasionally reads out the buttons (rather than
//! sleeping, using interrupts and timers, as one would preferably do to be responsive and
//! efficient), but that's what the current HAL implementation can give.

#![no_main]
#![no_std]

extern crate panic_semihosting;

use cortex_m_rt::entry;
use embedded_hal::blocking::delay::DelayMs;

#[entry]
fn main() -> ! {
    let board = thunderboard_sltb001a::Board::new();
    let mut leds = board.leds;
    let buttons = board.buttons;
    let mut delay = board.delay;
    let mut pic = board.pic;

    let mut count = 1;

    loop {
        if buttons.button1_pressed() {
            leds.led0_off();
        } else {
            leds.led0_on();
        }

        delay.delay_ms(500u16);
        if !buttons.button0_pressed() {
            leds.led1_on();
        }
        delay.delay_ms(500u16);
        leds.led1_off();

        count = (count + 1) % 4;
        pic.set_leds(count == 0, count == 1, count == 2, count == 3);
    }
}
