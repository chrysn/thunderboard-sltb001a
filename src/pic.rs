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

const ADDR: u8 = 0x48;

#[derive(Debug)]
pub struct InterruptSet {
    pub ccs: bool,
    pub imu: bool,
    pub uv: bool,
}

impl InterruptSet {
    fn to_bits(&self) -> u8 {
        ((self.ccs as u8) << 0) | ((self.imu as u8) << 1) | ((self.uv as u8) << 2)
    }

    fn from_bits(byte: u8) -> Self {
        InterruptSet {
            ccs: (byte & 0x01) != 0,
            imu: (byte & 0x02) != 0,
            uv: (byte & 0x04) != 0,
        }
    }
}

/// Configuration options for the PIC's interrupt line
#[derive(Debug)]
pub enum InterruptConfiguration {
    /// Pull low once when an interrupt arrives (default)
    SinglePulse,
    /// Pull low as long as an interrupt is set
    Latched,
    /// Pull low in periodic intervals as long aas an interrupt is set
    Periodic(u8),
}

impl InterruptConfiguration {
    fn to_bits(&self) -> u8
    {
        match *self {
            InterruptConfiguration::SinglePulse => 0u8,
            InterruptConfiguration::Latched => 0x10,
            InterruptConfiguration::Periodic(i) if i < 0x8u8 => i | 0x8,
            _ => panic!("Invalid period time"),
        }
    }
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

    fn set_register(&mut self, reg: u8, value: u8)
    {
        self.acquiring(|i2c| {
            i2c.write(ADDR, &[reg, value]).unwrap();
        })
    }

    /// Enable or disable (ie. set power and connect SPI) the inertial sensor
    pub fn set_imu(&mut self, enable: bool)
    {
        self.set_register(0x01, enable as u8);
    }

    /// Enable or disable (ie. set power and connect I2C) the environmental sensor group
    pub fn set_env_sensor(&mut self, enable: bool)
    {
        self.set_register(0x01, enable as u8);
    }

    /// Enable or disable the microphone
    pub fn set_mic(&mut self, enable: bool)
    {
        self.set_register(0x02, enable as u8);
    }

    /// Enable or disable the indoor air quality sensor (ie. set power and connect I2C at 0x5a), and set its wake state
    pub fn set_ccs(&mut self, enable: bool, wake: bool)
    {
        let state = (enable as u8) | ((wake as u8) << 1);
        self.set_register(0x03, state);
    }

    /// Enable or disable the individual RGB LEDs.
    ///
    /// This only provides power to the LEDs; a color still needs to be set using the LED pins of
    /// the main MCU.
    pub fn set_leds(&mut self, led0: bool, led1: bool, led2: bool, led3: bool)
    {
        let led_config: u8 = ((led0 as u8) << 7) |
                             ((led1 as u8) << 6) |
                             ((led2 as u8) << 5) |
                             ((led3 as u8) << 4) |
                             ((led0 || led1 || led2 || led3) as u8);
        self.set_register(0x04, led_config);
    }

    /// Select which interrupts are active
    pub fn set_int(&mut self, enable: InterruptSet)
    {
        self.set_register(0x05, enable.to_bits());
    }

    /// Selectively clear the pending interrupts
    pub fn clear_int(&mut self, clear: InterruptSet)
    {
        self.set_register(0x06, clear.to_bits());
    }

    /// Query which interrupts are active (CCS, IMU or UV)
    ///
    /// Note that this method, like all others, includes a several microsecond delay to wake up the
    /// EFM8SB chip. As long as its wake state is not tracked (in which case it might be possible
    /// to directly read after an interrupt), it might be faster to just check the individual
    /// active devices for any pending interrupt causes.
    pub fn pending_int(&mut self) -> InterruptSet
    {
        self.acquiring(|i2c| {
            let mut result = [0xff; 1];
            i2c.write(ADDR, &[0x07]).unwrap();
            i2c.read(ADDR, &mut result).unwrap();
            InterruptSet::from_bits(result[0])
        })
    }

    /// Configure the interrupt controller settings
    pub fn set_int_mode(&mut self, mode: &InterruptConfiguration)
    {
        self.set_register(0x08, mode.to_bits());
    }

    /// Read the (major, minor, patch) version components of the PIC firmware version
    pub fn read_firmware_version(&mut self) -> [u8; 3]
    {
        self.acquiring(|i2c| {
            let mut result = [0xf1; 3];
            for i in 0..3 {
                i2c.write(ADDR, &[0xf8 + i]).unwrap();
                i2c.read(ADDR, &mut result[(i as usize)..((i+1) as usize)]).unwrap();
            }
            result
        })
    }

    /// Read the 4-byte device identification number
    pub fn read_device_id(&mut self) -> [u8; 4]
    {
        self.acquiring(|i2c| {
            let mut result = [0xff; 4];
            for i in 0..4 {
                i2c.write(ADDR, &[0xf8 + i]).unwrap();
                i2c.read(ADDR, &mut result[(i as usize)..((i+1) as usize)]).unwrap();
            }
            result
        })
    }

    pub fn destroy(self) -> (ConfiguredI2C0, D)
    {
        (self.i2c, self.delay)
    }
}
