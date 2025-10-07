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
* Run `comi -s` to save all current com ports to the settings.json found inside the Comi folder in your documents folder
   * Run `comi -v` to see where the settings.json file is saved
2. Replace the "alias" field with what you want the com port to be named

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


