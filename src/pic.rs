//! Access to the EFM8SB chip on the board, the Power and Interrupt Controller. That chip is
//! basically used as a port expander, and is thus primarily exposed as GPIO output pins.
//!
//! (There are no concrete plans yet as to how its use as of forwarding interrupts can be exposed).

use embedded_hal::blocking::delay::DelayUs;
use efr32xg1 as registers;
use efm32gg_hal::i2c::{I2CExt, ConfiguredI2C0};
use efm32gg_hal::cmu::I2C0Clk;
use efm32gg_hal::gpio::Disabled;
use efm32gg_hal::gpio::pins::{PD10, PC11, PC10};

pub struct PIC/*<D>*/
    /*where D: DelayUs<u16>*/
{
    i2c: ConfiguredI2C0,
//     delay: D // that's gonna be an issue with requiring mut on a delay
}

impl/*<D>*/ PIC/*<D>*/
    /*where D: DelayUs<u16>*/
{
    pub fn new(register: registers::I2C0, delay: &mut DelayUs<u16>, clk: I2C0Clk, pd10: PD10<Disabled>, pc11: PC11<Disabled>, pc10: PC10<Disabled>) -> Self
    {
        use efm32gg_hal::gpio::EFM32Pin;
        use embedded_hal::digital::OutputPin;
        let mut pic_int_wake = pd10.as_opendrain();
        pic_int_wake.set_low();
        delay.delay_us(5u16);

        use efm32gg_hal::i2c::I2CExt;
        let mut pic_i2c = register.with_clock(clk).with_scl(registers::i2c0::routeloc0::SCLLOCW::LOC15, pc11).unwrap().with_sda(registers::i2c0::routeloc0::SDALOCW::LOC15, pc10).unwrap();
        use embedded_hal::blocking::i2c::Write;
        pic_i2c.write(0x90, &[0x04, 0x81]).unwrap(); // Switch RGB LED 1 on

        // interesting fhow contrary to what the documentation says, it is *not* sufficient to just
        // send a pulse and i2c right away (i can't be too slow, can i?)
        pic_int_wake.set_high();

        PIC { i2c: pic_i2c }
    }

    pub fn set_leds(&mut self, led0: bool, led1: bool, led2: bool, led3: bool)
    {
    }
}
