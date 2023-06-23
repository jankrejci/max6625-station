#!/usr/bin/python

import spidev
import logging as log
log.basicConfig(level=log.DEBUG)

import time

from MAX6675 import MAX6675
import RPi.GPIO as GPIO

GPIO.setmode(GPIO.BCM)

# Raspberry Pi hardware SPI configuration.
SPI_PORT   = 0
SPI_DEVICE = 0

spi_device = spidev.SpiDev()
spi_device.open(SPI_PORT, SPI_DEVICE)
spi_device.max_speed_hz=500000
spi_device.mode = 0
spi_device.lsbfirst = False

# CS = [14,15,18,27,23,20,1,7,25,24]
CS = [23,24]

sensors = []
for cs in CS:
    sensors.append(
        MAX6675(spi_device, cs)
    )


while True:
    try:
        for id, sensor in enumerate(sensors):
            temp = sensor.read_temp()
            log.info(f"ID {id}, temp {temp:0.3F}°C")
        time.sleep(1.0)
    except KeyboardInterrupt:
        break

GPIO.cleanup()