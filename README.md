# ComInfo
- Display Com Ports.
- Nicknames: Devices can be given nicknames in a settings.json file.
- Continuous mode: Updates display when USB serial devices are added or removed.


## Continuous update mode
![CommInfo](https://github.com/schiltz3/ComInfo/assets/45466247/1abd68ea-c5ed-42fb-a45c-44efa765a0b2)

## Example settings.json
* Only `alias`, `product_id`, and `serial_number` are required fields.
* An empty `alias` fields prevent the port from being displayed
```json
{
  "com_ports": [
    {
      "alias": "WLED",
      "product_id": 60000,
      "serial_number": "0001",
      "manufacturer": "Silicon Labs",
      "product_name": "Silicon Labs CP210x USB to UART Bridge"
    },
    {
      "alias": "",
      "product_id": 61254,
      "serial_number": "0002"
    }
  ]
}
```

## Instalation
* Download ComiSetup.exe from [Releases](https://github.com/schiltz3/ComInfo/releases) and run the installer. Administrator is required as the installer adds Comi.exe to the system PATH

## Running
* Run `comi` from the command line
* Alternatively, run `ComiRun` from the Windows Start Menu


## Building
## Debug
```powershell
cargo build
```

### Release
https://jrsoftware.org/isdl.php is required to build the installer
```powershell
winget install "inno setup"
pip install -r requirements.txt
python .\build.py release --installer
```


