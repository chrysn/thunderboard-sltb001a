//! The same behavior as blink, but implemented using interrupts. Don't worry that it looks quite a
//! bit different -- try shaking the device to see that it does blink. Having a lower blink rate
//! would require using a timer that's currently not implemented in the HAL crate.
//!
//! This requires nightly as it uses a statically initialized queue (would be needlessly
//! verbose without const-fn).

#![no_main]
#![no_std]
#![feature(const_fn)]

extern crate panic_semihosting;

use efr32xg1::interrupt;

use cortex_m_rt::entry;
use embedded_hal::blocking::delay::DelayMs;

#[entry]
fn main() -> ! {
    let board = thunderboard_sltb001a::Board::new();
    let mut leds = board.leds;
    let buttons = board.buttons;
    let mut delay = board.delay;
    let mut pic = board.pic;
    let mut nvic = board.nvic;

    nvic.enable(efr32xg1::Interrupt::TIMER1);

    // Show that nothing bad happens even if we call this too early
    cortex_m::peripheral::NVIC::pend(efr32xg1::Interrupt::TIMER1);

    // unsafe: The SPSC documentation does it too.
    let mut for_timer1 = unsafe { FOR_TIMER1.split().0 };
    for_timer1.enqueue((buttons, leds, pic)).ok().unwrap();

    // Matter of taste: I rather make sure all the initialization work is done before we really
    // start spinning
    cortex_m::peripheral::NVIC::pend(efr32xg1::Interrupt::TIMER1);

    let mut timer1 = board.timer1;

    timer1.enable_outputcompare(0);
    timer1.interrupt_enable(efm32gg_hal::timer::InterruptFlag::CC0);
    timer1.start();

    loop {
    }
}

use heapless::spsc::Queue;
use heapless::consts::U1;
use thunderboard_sltb001a::{button::Buttons, pic::PIC, RefCellDelay};
#[cfg(feature = "led-pwm")]
use thunderboard_sltb001a::led_pwm::LEDs;
#[cfg(not(feature = "led-pwm"))]
use thunderboard_sltb001a::led::LEDs;


// Queue along which peripherals are moved into the timer.
// See https://github.com/rust-embedded/wg/issues/294 for future safe directions.
// It would feel a tad more safe to .split() this right away, but the signature 'd get ugly.
static mut FOR_TIMER1: Queue<(Buttons, LEDs, PIC<RefCellDelay>), U1> = Queue::new();

#[interrupt]
fn TIMER1() {
    static mut stuff: Option<(Buttons, LEDs, PIC<RefCellDelay>)> = None;
    static mut halfcount: i32 = 0;

    efm32gg_hal::timer::Timer0::interrupt_unpend(efm32gg_hal::timer::InterruptFlag::CC0);

    if let Some((buttons, leds, pic)) = stuff {

        let count = *halfcount / 2;
        let phase = *halfcount % 2;
        match phase {
            0 => {
                pic.set_leds(count == 0, count == 1, count == 2, count == 3);
                leds.led1_off();
                if buttons.button1_pressed() {
                    leds.led0_off();
                } else {
                    leds.led0_on();
                }
            },
            1 => {
                if !buttons.button0_pressed() {
                    leds.led1_on();
                }
            },
            _ => unreachable!(),
        }

        *halfcount = (*halfcount + 1) % 8;
    } else {
        // unsafe: The SPSC documentation does it too.
        let mut for_timer1 = unsafe { FOR_TIMER1.split().1 };
        *stuff = for_timer1.dequeue();
    }
}
