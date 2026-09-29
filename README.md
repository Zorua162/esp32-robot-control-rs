# Flashing

To flash from Windows:

`usbipd list`

```powershell
BUSID  VID:PID    DEVICE                                                        STATE
1-2    303a:1001  USB Serial Device (COM12), USB JTAG/serial debug unit         Not shared
1-7    0a12:0001  Generic Bluetooth Radio                                       Not shared
1-8    03f0:0b92  HyperX Virtual Surround Sound, USB Input Device               Not shared
1-10   1038:1702  USB Input Device, SteelSeries Rival 100, SteelSeries GG C...  Not shared
3-2    3282:0001  USB Input Device                                              Not shared
```

RUN AS ADMINISTRATOR
`usbipd bind --busid 1-2`

where the busid is found from the above list (the USB serial one)

Then run in a regular use sesh
`usbipd attach --wsl --busid 1-2 --host-ip 172.20.240.1`
