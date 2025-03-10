use clap::Parser;
use console::Term;
// use rusb;
use serialport::{available_ports, SerialPortInfo, SerialPortType, UsbPortInfo};
use settings::{install_settings_file, read_settings_from_file};
use std::{borrow::Borrow, path::PathBuf, thread, time};
mod settings;

#[derive(Parser, Debug)]
#[command(
    name = "Comi",
    author = "https://github.com/schiltz3",
    version,
    about = "Display list of active COM ports",
    after_help = "Settings file is installed to documents/Comi/settings.json by default"
)]
#[command(
    help_template = "{about-section}\n{usage-heading} {usage}\n\n{all-args} {after-help}\n\nAuthor: {author}"
)]
pub struct Args {
    /// Continuously monitor COM Ports and update
    #[arg(short, long)]
    continuous: bool,

    /// Leave blank to search for settings in exe directory
    #[arg(short, long)]
    settings: Option<PathBuf>,

    /// Print verbose output. ex: The full settings file path
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let term = Term::stdout();
    term.set_title("Serial List");
    // println!("Support hotplug?: {}", rusb::has_hotplug());

    let args = Args::parse();

    // Copy the settings file to user docs if it doesn't already exist
    let install_result = install_settings_file();
    if install_result.is_err() {
        eprintln!("{}", install_result.unwrap_err());
    }

    // Get path we think we should use for settings.json
    let settings_file_path: Option<PathBuf> = settings::find_settings_path(&args.settings, args.verbose);

    // Open path and extract settings
    let default_settings = read_settings_from_file(&settings_file_path)
        .unwrap_or(settings::Settings { com_ports: None });

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
// Checks if a Com port alias entry equals a com port
fn alias_com_port_eq(serial_port_info: &UsbPortInfo, com_port: &settings::ComPort) -> bool {
    let mut matched_element = 0;
    let mut matched = true;
    if serial_port_info.pid == com_port.product_id {
        matched = matched && true;
        matched_element += 1;
    } else {
        matched = false;
    }

    match serial_port_info.serial_number.borrow() {
        Some(s) => {
            if s == &com_port.serial_number {
                matched = matched && true;
                matched_element += 1;
            } else {
                matched = false;
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
                } else {
                    matched = false;
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
                } else {
                    matched = false;
                }
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

fn continuous_update(term: &Term, settings: settings::Settings) {
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

fn single_update(settings: settings::Settings) {
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

fn print_ports(ports: Vec<SerialPortInfo>, settings: &settings::Settings) {
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
                }
            }
            _ => {}
        }
    }
    if serial_port_count == 0 {
        println!("No COM ports found.")
    } else {
        println!("-------");
    }
}
