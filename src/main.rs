use clap::Parser;
use console::Term;
// use rusb;
use serialport::{available_ports, SerialPortInfo, SerialPortType};
use std::{thread, time};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    /// Continuously monitor COM Ports and update
    #[arg(short, long)]
    continuous: bool,
}

fn main() {
    let term = Term::stdout();
    term.set_title("Serial List");

    let args = Args::parse();

    // println!("Support hotplug?: {}", rusb::has_hotplug());

    if args.continuous {
        continuous_update(&term);
    } else {
        single_update();
    }
}
fn continuous_update(term: &Term) {
    let mut previous_num = usize::MAX;
    loop {
        match available_ports() {
            Ok(ports) => {
                if ports.len() != previous_num {
                    previous_num = ports.len().to_owned();

                    let _ = term.clear_screen();
                    print_ports(ports);
                }
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }
        thread::sleep(time::Duration::from_millis(100));
    }
}

fn single_update() {
    match available_ports() {
        Ok(ports) => {
            if ports.len() == 0 {
                println!("No serial ports found.")
            } else {
                print_ports(ports);
            }
        }
        Err(e) => {
            println!("{:?}", e);
        }
    }
}

fn print_ports(ports: Vec<SerialPortInfo>) {
    let mut serial_port_count: u16 = 0;
    for port in ports {
        match port.port_type {
            SerialPortType::UsbPort(usbinfo) => {
                serial_port_count += 1;
                println!("-------");
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
                println!("-------");
            }
            _ => {}
        }
    }
    if serial_port_count == 0 {
        println!("No COM ports found.")
    }
}
