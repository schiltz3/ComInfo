use console::Term;
use rusb;
use serialport::{available_ports, SerialPortInfo, SerialPortType};
use std::{thread, time};
fn main() {
    let term = Term::stdout();
    term.set_title("Serial List");

    println!("Support hotplug?: {}", rusb::has_hotplug());

    let mut previous_num = 0;
    loop {
        match available_ports() {
            Ok(ports) => {
                if ports.len() != previous_num {
                    previous_num = ports.len();
                    print_ports(ports, &term);
                }
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }
        thread::sleep(time::Duration::from_millis(100));
    }
}

fn print_ports(ports: Vec<SerialPortInfo>, term: &Term) {
    let _ = term.clear_screen();
    println!("-------");
    for port in ports {
        match port.port_type {
            SerialPortType::UsbPort(usbinfo) => {
                println!("{}", port.port_name);
                println!("\tProduct: {}", usbinfo.product.clone().unwrap_or_default());
                println!(
                    "\tManufacturer: {}",
                    usbinfo.manufacturer.clone().unwrap_or_default()
                );
                println!("\tPid: {}", usbinfo.pid);
                println!(
                    "\tSerial Number: {}",
                    usbinfo.serial_number.clone().unwrap_or_default()
                );
            }
            _ => {}
        }
        println!("-------");
    }
}
