# ComInfo
- Display Com Port info. 
- Continuous mode: Updates display when USB serial devices are added or removed
- Aliases: Devices can be given names through a .json settings file.

## Continuous update mode
![CommInfo](https://github.com/schiltz3/ComInfo/assets/45466247/1abd68ea-c5ed-42fb-a45c-44efa765a0b2)

## Example settings.json
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



### Notes:
- Still in alpha, so UI is a little rough and features like renaming devices are not supported
- Adding the install path to your system `PATH` variable allows you to call `comi.exe` from anywhere
