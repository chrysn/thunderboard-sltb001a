//! Access to the EFM8SB chip on the board, the Power and Interrupt Controller. That chip is
//! basically used as a port expander, and is thus primarily exposed as GPIO output pins.
//!
//! (There are no concrete plans yet as to how its use as of forwarding interrupts can be exposed).

use embedded_hal::blocking::delay::DelayUs;
use efr32xg1 as registers;
use efm32gg_hal::i2c::ConfiguredI2C0;
use efm32gg_hal::cmu::I2C0Clk;
use efm32gg_hal::gpio::{Disabled, Output};
use efm32gg_hal::gpio::pins::{PD10, PC11, PC10};

use embedded_hal::digital::OutputPin;
use embedded_hal::blocking::i2c::{Write, Read};

pub struct PIC<D>
{
    i2c: ConfiguredI2C0,
    delay: D,
    int_wake: PD10<Output>,
}

impl<D> PIC<D>
    where D: DelayUs<u16>
{
    pub fn new(register: registers::I2C0, delay: D, clk: I2C0Clk, pd10: PD10<Disabled>, pc11: PC11<Disabled>, pc10: PC10<Disabled>) -> Self
    {
        use efm32gg_hal::gpio::EFM32Pin;
        use efm32gg_hal::i2c::I2CExt;

        let i2c = register.with_clock(clk).with_scl(registers::i2c0::routeloc0::SCLLOCW::LOC15, pc11).unwrap().with_sda(registers::i2c0::routeloc0::SDALOCW::LOC15, pc10).unwrap();

        let mut int_wake = pd10.as_opendrain();
        int_wake.set_high();

        PIC { i2c: i2c, delay: delay, int_wake: int_wake }
    }

    fn acquiring<T>(&mut self, inner: impl FnOnce(&mut ConfiguredI2C0) -> T) -> T
    {
        self.int_wake.set_low();
        self.delay.delay_us(5u16);

        let result = inner(&mut self.i2c);

        // interesting fhow contrary to what the documentation says, it is *not* sufficient to just
        // send a pulse and i2c right away (i can't be too slow, can i?)
        self.int_wake.set_high();

        result
    }

    pub fn set_leds(&mut self, led0: bool, led1: bool, led2: bool, led3: bool)
    {
        self.acquiring(|i2c| {
            let led_config: u8 = ((led0 as u8) << 7) |
                                 ((led1 as u8) << 6) |
                                 ((led2 as u8) << 5) |
                                 ((led3 as u8) << 4) |
                                 ((led0 || led1 || led2 || led3) as u8);
            i2c.write(0x90, &[0x04, led_config]).unwrap();
        })
    }

    pub fn read_device_id(&mut self) -> [u8; 4]
    {
        self.acquiring(|i2c| {
            let mut result = [0xff; 4];
            for i in 0..4 {
                i2c.write(0x90, &[0xf8 + i]).unwrap();
                i2c.read(0x90, &mut result[(i as usize)..((i+1) as usize)]).unwrap();
            }
            result
        })
    }

    pub fn destroy(self) -> (ConfiguredI2C0, D)
    {
        (self.i2c, self.delay)
    }
}
