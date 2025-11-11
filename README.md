# Remember/forget the serial

```sh
vim ~/.config/espflash/espflash_ports.toml
```

# Can't connect

Error:
> [2025-11-04T20:41:43Z INFO ] Connecting...
> Error:   × Failed to open serial port /dev/ttyACM0
>   ╰─▶ Error while connecting to device

Solution ([source](https://github.com/esphome/issues/issues/4525#issuecomment-2676978792)):
```sh
sudo setfacl -m u:USERNAME:rw /dev/ttyACM0 
```

# linker `xtensa-esp32-elf-gcc` not found

```sh
cargo install espup --locked
espup install

# Run once
. ~/export-esp.sh

# Or put in Fish profile
echo ". $HOME/export-esp.sh" >> $HOME/.fish_profile
```
