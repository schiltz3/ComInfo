use clap::Parser;
use console::Term;
use serde::Deserialize;
// use rusb;
use serialport::{available_ports, SerialPortInfo, SerialPortType, UsbPortInfo};
use std::{borrow::Borrow, fs, path::PathBuf, thread, time};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    /// Continuously monitor COM Ports and update
    #[arg(short, long)]
    continuous: bool,

    #[arg(short, long)]
    settings: Option<PathBuf>,
}

#[derive(Deserialize, Debug)]
struct ComPort {
    alias: String,
    product_id: u16,
    serial_number: String,
    manufacturer: Option<String>,
    product_name: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Settings {
    com_ports: Option<Vec<ComPort>>,
}

fn main() {
    let term = Term::stdout();
    term.set_title("Serial List");
    // println!("Support hotplug?: {}", rusb::has_hotplug());

    let args = Args::parse();
    let mut settings: Option<Settings> = None;

    match args.settings {
        Some(settings_path) => {
            if settings_path.exists() {
                let config_file = fs::read_to_string(&settings_path);
                match config_file {
                    Ok(config_json) => {
                        settings = serde_json::from_str(&config_json).unwrap_or(None);
                        if settings.is_none() {
                            println!(
                                "Error parsing settings from: {}",
                                settings_path
                                    .to_str()
                                    .expect("Unable to convert path too string")
                            );
                        }
                    }
                    _ => {
                        println!(
                            "Path does not exist: {}",
                            settings_path
                                .to_str()
                                .expect("Unable to convert path too string")
                        );
                    }
                }
            }
        }
        None => {}
    }

    let default_settings = settings.unwrap_or(Settings { com_ports: None });

    if args.continuous {
        continuous_update(&term, default_settings);
    } else {
        single_update(default_settings);
    }
}

// enum ComAlias {
//     String,
//     SerialPortInfo,
// }

fn remove_last_word(input: &str) -> String {
    if let Some(last_space_idx) = input.rfind(' ') {
        let new_string = input[..last_space_idx].to_string();
        return new_string;
    }
    // If there's no space, just return an empty string or the original string as per your requirement.
    input.to_string()
}
fn alias_com_port_eq(serial_port_info: &UsbPortInfo, com_port: &ComPort) -> bool {
    let mut matched_element = 0;
    let mut matched = true;
    if serial_port_info.pid == com_port.product_id {
        matched = matched && true;
        matched_element += 1;
    }

    match serial_port_info.serial_number.borrow() {
        Some(s) => {
            if s == &com_port.serial_number {
                matched = matched && true;
                matched_element += 1;
            }
        }
        None => {}
    }

    match serial_port_info.manufacturer.borrow() {
        Some(m) => match com_port.manufacturer.borrow() {
            Some(mn) => {
                if m == mn {
                    matched = matched && true;
                    matched_element += 1;
                }
            }
            None => {}
        },
        None => {}
    }

    match serial_port_info.product.borrow() {
        Some(p) => match com_port.product_name.borrow() {
            Some(pn) => {
                if pn == &remove_last_word(p) {
                    matched = matched && true;
                    matched_element += 1;
                };
            }
            None => {}
        },
        None => {}
    }
    return matched && (matched_element > 0);
}

// fn filter(ports: &Vec<SerialPortInfo>, aliases: &Vec<ComPort>) -> Vec<ComAlias> {
//     ports.into_iter().map(|com_info | -> {
//         for alias in  aliases{

//         }

//     })
// }

fn continuous_update(term: &Term, settings: Settings) {
    let mut previous_num = usize::MAX;
    loop {
        match available_ports() {
            Ok(ports) => {
                if ports.len() != previous_num {
                    previous_num = ports.len().to_owned();

                    let _ = term.clear_screen();
                    print_ports(ports, &settings);
                }
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }
        thread::sleep(time::Duration::from_millis(100));
    }
}

fn single_update(settings: Settings) {
    match available_ports() {
        Ok(ports) => {
            if ports.len() == 0 {
                println!("No serial ports found.")
            } else {
                print_ports(ports, &settings);
            }
        }
        Err(e) => {
            println!("{:?}", e);
        }
    }
}

fn print_ports(ports: Vec<SerialPortInfo>, settings: &Settings) {
    let mut serial_port_count: u16 = 0;
    for port in ports {
        match port.port_type {
            SerialPortType::UsbPort(usbinfo) => {
                let mut skip_printing = false;
                serial_port_count += 1;

                match settings.com_ports.borrow() {
                    Some(com_port_aliases) => {
                        for com_port_alias in com_port_aliases {
                            if alias_com_port_eq(&usbinfo, com_port_alias) {
                                skip_printing = true;
                                println!("-------");
                                println!("{} {}", port.port_name, com_port_alias.alias);
                                println!("-------");
                            }
                        }
                    }
                    None => {}
                }
                if !skip_printing {
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
            }
            _ => {}
        }
    }
    if serial_port_count == 0 {
        println!("No COM ports found.")
    }
}
