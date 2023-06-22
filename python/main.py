#!/usr/bin/python

import logging as log
log.basicConfig(level=log.INFO)

import time

import Adafruit_GPIO.SPI as SPI
from MAX6675 import MAX6675

# Raspberry Pi software SPI configuration.
CLK = 11
DO  = 9

# CS = [14,15,18,27,23,20,1,7,25,24]
CS = [23,24]

sensors = []
for cs in CS:
    sensors.append((cs, MAX6675(CLK, cs, DO)))

while True:
    for id, (cs, spi) in enumerate(sensors):
        temp = spi.read_temp()
        log.info(f"ID {id}, CS {cs}, temp {temp:0.3F}°C")
    time.sleep(1.0)