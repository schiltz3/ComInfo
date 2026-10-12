# ComInfo
- Run from the command line to display all Com Ports.
- Nicknames: Devices can be given nicknames in a settings.json file.
- Continuous mode: Updates display when USB serial devices are added or removed.
- Command Line Substitution: Use the com port's alias with existing command line programs

## Instalation
* Download ComiSetup.exe from [Releases](https://github.com/schiltz3/ComInfo/releases) and run the installer. Administrator is required as the installer adds Comi.exe to the system PATH

## Running
* Run `comi` from the command line
* Alternatively, run `ComiRun` from the Windows Start Menu
* Run `comi -h` to see all available options

## Giving Ports Aliases
* Run `comi --save` to save all current com ports to the settings.json found inside the Comi folder in your documents folder
   * Run `comi -v` to see where the settings.json file is saved
2. Replace the "alias" field with what you want the COM port to be named

## Using comi to substitute names for com ports
Running `comi -a com_alias` will return the com port related to that alias.
ex: Windows Powershell
```PowerShell
plink -serial -sercfg 115200,8,n,1,N $(comi -a "Tool UART 5")
```

## Continuous update mode
`comi -c` opens in continuous mode to update as you plug in or remove com port devices
![CommInfo](https://github.com/schiltz3/ComInfo/assets/45466247/1abd68ea-c5ed-42fb-a45c-44efa765a0b2)

## Example settings.json
* Only `alias`, `product_id`, and `serial_number` are required fields.
* An empty `alias` fields prevent the port from being displayed
* `interface` is optional. Use it to name each COM port of a composite device that exposes several ports under the same `product_id` and `serial_number` (for example an STM32 with two CDC ports, `MI_00` and `MI_02` in Device Manager). `comi` shows the interface number of unnamed ports. `comi --save` records it only for devices that have several COM ports. An entry without `interface` matches every interface of the device. If an older entry covers a device with several COM ports, `comi` suggests running `comi --save`, which splits that entry into one entry per interface and keeps the alias on each so you can rename them. If every interface of that device already has its own entry, `comi --save` removes the older entry instead.
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
    },
    {
      "alias": "Comms Adapter Port A",
      "product_id": 22336,
      "serial_number": "0003",
      "interface": 0
    },
    {
      "alias": "Comms Adapter Port B",
      "product_id": 22336,
      "serial_number": "0003",
      "interface": 2
    }
  ]
}
```

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


