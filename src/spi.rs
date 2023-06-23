use anyhow::Result;
use spidev::{SpiModeFlags, Spidev, SpidevOptions};
use std::io::Read;

pub struct Spi {
    inner: Spidev,
}

impl Spi {
    pub fn open() -> Self {
        let mut inner = Spidev::open("/dev/spidev0.0").expect("BUG: Failed to open SPI device");
        let options = SpidevOptions::new()
            .bits_per_word(8)
            .max_speed_hz(500_000)
            .mode(SpiModeFlags::SPI_MODE_0)
            .build();
        inner
            .configure(&options)
            .expect("BUG: failed to configure spi");

        Self { inner }
    }

    pub fn read(&mut self) -> Result<[u8; 2]> {
        let mut rx_buf = [0_u8; 2];
        self.inner.read(&mut rx_buf)?;
        Ok(rx_buf)
    }
}
