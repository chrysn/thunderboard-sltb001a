use efm32gg_hal::gpio;
use embedded_hal::digital::OutputPin;
use efm32gg_hal::gpio::EFM32Pin;

/// A representation of the two user LEDs on the STK3700
pub struct LEDs {
    led0: gpio::pins::PD11<gpio::Output>,
    led1: gpio::pins::PD12<gpio::Output>,
}

impl LEDs {
    pub fn new(pd11: gpio::pins::PD11<gpio::Disabled>, pd12: gpio::pins::PD12<gpio::Disabled>) -> Self {
        LEDs { led0: pd11.as_output(), led1: pd12.as_output() }
    }

    pub fn led0_on(&mut self)
    {
        self.led0.set_high();
    }

    pub fn led0_off(&mut self)
    {
        self.led0.set_low();
    }

    pub fn led1_on(&mut self)
    {
        self.led1.set_high();
    }

    pub fn led1_off(&mut self)
    {
        self.led1.set_low();
    }
}
