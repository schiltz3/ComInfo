use clap::Parser;
use console::Term;
// use rusb;
use serialport::{available_ports, SerialPortInfo, SerialPortType, UsbPortInfo};
use settings::{install_settings_file, read_settings_from_file, validate_settings};
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
    let settings_file_path: Option<PathBuf> =
        settings::find_settings_path(&args.settings, args.verbose);

    // Open path and extract settings
    let default_settings = read_settings_from_file(&settings_file_path)
        .unwrap_or(settings::Settings { com_ports: None });

    let valid_settings = validate_settings(&default_settings);

    if valid_settings.is_err() {
        eprintln!("{}", valid_settings.unwrap_err());
    } else {
        if args.continuous {
            continuous_update(&term, default_settings);
        } else {
            single_update(default_settings);
        }
    }
}

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
                let com_port_info: &settings::ComPort = &usbinfo.into();
                let mut skip_printing = false;
                serial_port_count += 1;

                match settings.com_ports.borrow() {
                    Some(com_port_aliases) => {
                        for com_port_alias in com_port_aliases {
                            if com_port_info == com_port_alias {
                                skip_printing = true;
                                if com_port_alias.alias.is_empty() {
                                    // Decrement the count if we want to hide this port
                                    serial_port_count -= 1;
                                } else {
                                    //TODO: Pull out to print alias function
                                    println!("-------");
                                    println!("{} {}", port.port_name, com_port_alias.alias);
                                }
                            }
                        }
                    }
                    None => {}
                }
                if !skip_printing {
                    //TODO: Pull out to print port function
                    println!("-------");
                    println!("{}", port.port_name);
                    println!(
                        "\tProduct: {}",
                        com_port_info.product_name.clone().unwrap_or_default()
                    );
                    println!(
                        "\tManufacturer: {}",
                        com_port_info.manufacturer.clone().unwrap_or_default()
                    );
                    println!("\tPid: {}", com_port_info.product_id);
                    println!("\tSerial Number: {}", com_port_info.serial_number.clone());
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
