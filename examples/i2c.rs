//! Do a bus scan of the I2C bus, and read some values.
//!
//! How the I2C bus is taken is a bit odd and symptomatic of the issue of sharing a peripheral
//! between multiple users: It is taken from the PIC abstraction's cold dead hands.
//!
//! The example prints to semihosted stdout (watch your OpenOCD console), and then ends in a loop.
//!
//! Everything outside of main is taken from the cortex-m-quickstart examples.

#![no_main]
#![no_std]

#[macro_use(entry, exception)]
extern crate cortex_m_rt as rt;

extern crate thunderboard_sltb001a;

extern crate embedded_hal;

extern crate panic_semihosting;

extern crate efm32gg_hal;
extern crate cortex_m_semihosting;
use efm32gg_hal::i2c::{ConfiguredI2C0, Error::AddressNack};
use cortex_m_semihosting::hio;
use core::fmt::Write;

use rt::ExceptionFrame;

entry!(main);

fn main() -> ! {
    let board = thunderboard_sltb001a::Board::new();
    let mut pic = board.pic;

    pic.set_env_sensor(true);
    pic.set_ccs(true, true);
    writeln!(hio::hstdout().unwrap(), "Firmware version: {:?}", pic.read_firmware_version()).unwrap();
    writeln!(hio::hstdout().unwrap(), "Interrupts set: {:?}", pic.pending_int()).unwrap();

    let (mut i2c, _) = pic.destroy();

    /// Scan a bus address, report if an Ack came back.
    fn scan(i2c: &mut ConfiguredI2C0, addr: u8) {
        let mut buf = [0u8; 1];
        let result = i2c.read(addr, &mut buf);
        match result {
            Err(AddressNack) => (),
            result => writeln!(hio::hstdout().unwrap(), "From {:#x}: {:?} ({:x?})", addr, result, buf).unwrap()
        }
    }
    for addr in 0..128u8
    {
        scan(&mut i2c, addr);
    }

    // Play with the SL1133. Getting it to do more would involve the choice of what exactly to read
    // from it, and how often, and when to fetch the data. (A one-shot read-everything would be
    // nice, would probably mean forced mode.)
    use embedded_hal::blocking::i2c::{Write, Read};
    i2c.write(0x55, &[0]).unwrap();
    let mut devicedata = [0u8; 3];
    i2c.read(0x55, &mut devicedata).unwrap();
    assert!(devicedata == [0x33, 0x03, 0x10]);


    loop { }
}

// define the hard fault handler
exception!(HardFault, hard_fault);

fn hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

// define the default exception handler
exception!(*, default_handler);

fn default_handler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
