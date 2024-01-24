use serialport::{available_ports, SerialPortType};
fn main() {
    match available_ports() {
        Ok(ports) => {
            match ports.len() {
                0 => println!("No Ports Found"),
                1 => println!("1 Port FOund"),
                _ => println!("Many ports found"),
            }
            for port in ports {
                println!("{}", port.port_name);
                match port.port_type {
                    SerialPortType::UsbPort(_usbinfo) => {
                        println!(" Usb Type: ");
                    }
                    _ => {}
                }
            }
        }
        Err(e) => {
            println!("{:?}", e);
        }
    }
}
