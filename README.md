# ComInfo
- Display Com Port info. 
- Continuous mode: Updates display when USB serial devices are added or removed.
- Aliases: Devices can be given names through a .json settings file found in the install directory.

## Continuous update mode
![CommInfo](https://github.com/schiltz3/ComInfo/assets/45466247/1abd68ea-c5ed-42fb-a45c-44efa765a0b2)

## Example settings.json
Only `alias`, `product_id`, and `serial_number` are required fields.
```json
{
  "com_ports": [
    {
      "alias": "WLED",
      "product_id": 60000,
      "serial_number": "0001",
      "manufacturer": "Silicon Labs",
      "product_name": "Silicon Labs CP210x USB to UART Bridge"
    }
  ]
}
```

## How to Run
* Download ComiSetup.exe from [Releases](https://github.com/schiltz3/ComInfo/releases) and run the installer. Administrator is required as the installer adds Comi.exe to the system PATH
* By default, ComInfo does not open in continuous mode, so running the exe will cause a terminal to flash on the screen briefly.
* To run ComInfo, call it from the command line by using `cmd` or `powershell`, navigating to the install directory, and running `comi` or `comi -h` for more info.


## Building
https://jrsoftware.org/isdl.php to build the installer

