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
sudo apt install git
```

Install rust toolchain
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

You can also install `Remote - SSH` extension to the VScode on your host machine and connect through remote development.

## Run the application

```
git clone https://github.com/jankrejci/max6625-station.git
cd max6625-station
cargo run
```

Results appears on `ip_address:8080/metrics`. Sensors are able to do hotplug so you can add or remove sensors as you wish.