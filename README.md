# Robot control notes

## Flashing

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

## Common issues

An error like:

```bash
  = note: [ldproxy] Running ldproxy
          Error: Linker /home/zorua162/.espressif/tools/xtensa-esp-elf/esp-14.2.0_20251107/xtensa-esp-elf/bin/xtensa-esp32s3-elf-gcc failed: exit status: 1
          STDERR OUTPUT:
          /home/zorua162/.espressif/tools/xtensa-esp-elf/esp-14.2.0_20251107/xtensa-esp-elf/bin/../lib/gcc/xtensa-esp-elf/14.2.0/../../../../xtensa-esp-elf/bin/ld: esp-idf/bluepad32_platform/libbluepad32_platform.a(bluepad32_platform.c.obj):(.literal.bluepad32_platform_run+0x4): undefined reference to btstack_init'
          /home/zorua162/.espressif/tools/xtensa-esp-elf/esp-14.2.0_20251107/xtensa-esp-elf/bin/../lib/gcc/xtensa-esp-elf/14.2.0/../../../../xtensa-esp-elf/bin/ld: esp-idf/bluepad32_platform/libbluepad32_platform.a(bluepad32_platform.c.obj): in function 
```

Was fixed previously by editing sdkconfig.defaults (forgot exactly which line sorry,
but I think its `CONFIG_BLUEPAD32_PLATFORM_CUSTOM=y`, telling it to use a custom platform)
However, note that this gets cached into the target folder, so when making a change its
necessary to then run `cargo clean` first before the change will take
