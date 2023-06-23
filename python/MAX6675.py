import logging as log
import RPi.GPIO as GPIO
import time

class MAX6675(object):
    """Class to represent a MAX6675 thermocouple temperature measurement board.
    """

    def __init__(self, spi_device, cs):
        """Initialize MAX6675 device with hardware SPI and custom CS.
        """
        log.debug(f"Using hardware SPI with CS {cs}")

        self._spi = spi_device
        self._cs = cs
        GPIO.setup(cs, GPIO.OUT, initial = GPIO.LOW)


    def read_temp(self):
        """Return the thermocouple temperature value in degrees celsius."""
        v = self._read16()
        # Check for error reading value.
        if v & 0x4:
            return float('NaN')
        # Check if signed bit is set.
        if v & 0x80000000:
            # Negative value, take 2's compliment. Compute this with subtraction
            # because python is a little odd about handling signed/unsigned.
            v >>= 3 # only need the 12 MSB
            v -= 4096
        else:
            # Positive value, just shift the bits to get the value.
            v >>= 3 # only need the 12 MSB
        # Scale by 0.25 degrees C per bit and return value.
        return v * 0.25

    def _read16(self):
        GPIO.output(self._cs, GPIO.LOW)

        # Read 16 bits from the SPI bus.
        raw = self._spi.readbytes(2)
        if raw is None or len(raw) != 2:
            raise RuntimeError('Did not read expected number of bytes from device!')
        value = raw[0] << 8 | raw[1]
        log.debug('Raw value: 0x{0:08X}'.format(value & 0xFFFFFFFF))

        GPIO.output(self._cs, GPIO.HIGH)

        return value