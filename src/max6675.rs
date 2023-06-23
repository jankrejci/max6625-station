use crate::spi::Spi;
use anyhow::{anyhow, Result};
use rppal::gpio::{Gpio, OutputPin};
use std::sync::{Arc, Mutex};

pub struct MAX6675 {
    spi: Arc<Mutex<Spi>>,
    cs: OutputPin,
    pub id: usize,
}

impl MAX6675 {
    pub fn new(spi: Arc<Mutex<Spi>>, cs_pin: u8, id: usize) -> Self {
        let mut cs = Gpio::new()
            .expect("Failed to create new GPIO pin instance")
            .get(cs_pin)
            .expect("Failed to get pin")
            .into_output();

        cs.set_high();

        Self { spi, cs, id }
    }

    pub fn read_temp(&mut self) -> Result<f64> {
        let mut spi = self.spi.lock().expect("BUG: Failed to acquire SPI device");

        // Custom CS implementation
        self.cs.set_low();
        let value = spi.read()?;
        self.cs.set_high();

        let mut value: usize = (value[0] as usize) << 8 | value[1] as usize;

        // If there are no data on the bus, there are still SCK present,
        // therefore 0x0 is read.
        if value == 0 {
            return Err(anyhow!("Sensor is not present"));
        }

        if value & 0x4 == 0x4 {
            return Err(anyhow!("Temerature is NaN"));
        }

        // First 3 bits are just status flags
        value >>= 3;
        // Negative value, take 2's compliment. Compute this with subtraction.
        if value & 0x10000000 == 0x10000000 {
            value -= 4096
        }

        Ok(value as f64 * 0.25)
    }
}
