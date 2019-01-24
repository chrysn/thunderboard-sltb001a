use efm32gg_hal::{gpio, timer};
use embedded_hal::PwmPin;
use efm32gg_hal::gpio::EFM32Pin;

/// A representation of the two user LEDs on the STK3700
pub struct LEDs {
    led0: efm32gg_hal::timer::RoutedTimerChannel<efm32gg_hal::timer::Timer0, efm32gg_hal::timer::Channel0, efm32gg_hal::gpio::pins::PD11<efm32gg_hal::gpio::Output>>,
    led1: efm32gg_hal::timer::RoutedTimerChannel<efm32gg_hal::timer::Timer0, efm32gg_hal::timer::Channel1, efm32gg_hal::gpio::pins::PD12<efm32gg_hal::gpio::Output>>,
}

impl LEDs {
    pub fn new(pd11: gpio::pins::PD11<gpio::Disabled>, pd12: gpio::pins::PD12<gpio::Disabled>, mut pwmtimer: timer::Timer0) -> Self {
        pwmtimer.start();
        let timer::Channels { channel0, channel1, channel2 } = pwmtimer.split();
        let mut led0 = channel0.route(pd11.as_output());
        let mut led1 = channel1.route(pd12.as_output());
        led0.enable();
        led1.enable();
        LEDs { led0, led1 }
    }

    pub fn led0_on(&mut self)
    {
        self.led0.set_duty(500);
    }

    pub fn led0_off(&mut self)
    {
        self.led0.set_duty(0);
    }

    pub fn led1_on(&mut self)
    {
        self.led1.set_duty(1000);
    }

    pub fn led1_off(&mut self)
    {
        self.led1.set_duty(0);
    }
}
