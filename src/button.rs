use efm32gg_hal::gpio;
use embedded_hal::digital::InputPin;
use efm32gg_hal::gpio::EFM32Pin;

pub struct Buttons {
    button0: gpio::pins::PD14<gpio::Input>,
    button1: gpio::pins::PD15<gpio::Input>,
}

/// A representation of the two user buttons on the STK3700
impl Buttons {
    pub fn new(pd14: gpio::pins::PD14<gpio::Disabled>, pd15: gpio::pins::PD15<gpio::Disabled>) -> Self {
        Buttons { button0: pd14.as_input(), button1: pd15.as_input() }
    }

    /// Return true if PB0 in depressed state at the time of the invocation.
    pub fn button0_pressed(&self) -> bool
    {
        self.button0.is_low()
    }

    /// Return true if PB1 in depressed state at the time of the invocation.
    pub fn button1_pressed(&self) -> bool
    {
        self.button1.is_low()
    }
}
