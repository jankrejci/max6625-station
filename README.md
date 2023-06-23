# MAX6675 Station

## Prepare SD card
For the raspbian installation use the RPi imager tool
https://github.com/raspberrypi/rpi-imager
```
sudo apt install rpi-imager
```
* choose RPi OS Lite 64-bit
* go to advanced options
  * setup wifi credentials
  * enable ssh
  * change hostname to `max6675`
  * change username and password
* start flashing and wait till done

## Connect to the RPi and prepare it to work

Power on raspberry with flashed SD card and connect it to the network.
Preferably via cable, otherwies it can be difficult to find it as the hostname is not shown over wifi. 
```
nmap -p 22 --open -oG - xx.xx.xx.0/24 |grep max6675
```

When the IP address is known, connect through SSH.
```
ssh pi@xx.xx.xx.xx
```

Install basic tools
```
sudo apt install git python3-pip python3-venv python3-dev python3-smbus build-essential
```

Install rust toolchain
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Install Python dependencies
```
pip3 install RPi.GPIO spidev
```



You can also install `Remote - SSH` extension to the VScode on your host machine and connect through remote development.