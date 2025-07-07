use clap::Parser;
use console::Term;
// use rusb;
use serialport::{available_ports, SerialPortType};
use settings::{install_settings_file, read_settings_from_file, validate_settings};
use std::{path::PathBuf, thread, time};

use crate::settings::FzyEq;
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

    /// return a COM port if it exists given the alias
    #[arg(short, long)]
    alias: Option<String>,
}

pub struct ApplicationSettings {
    pub file_settings: settings::Settings,
    pub verbose: bool,
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
    let file_settings =
        read_settings_from_file(&settings_file_path).unwrap_or(settings::Settings {
            com_ports: Vec::new(),
        });

    let valid_settings = validate_settings(&file_settings);

    if valid_settings.is_err() {
        eprintln!("{}", valid_settings.unwrap_err())
    }

    let application_settings = ApplicationSettings {
        file_settings,
        verbose: args.verbose,
    };
    if args.alias.is_some() {
        let alias = args.alias.unwrap();
        print_com(&alias.trim().to_string(), &application_settings);
        return;
    } else if args.continuous {
        continuous_update(&term, application_settings);
    } else {
        single_update(application_settings);
    }
}

fn continuous_update(term: &Term, settings: ApplicationSettings) {
    let mut previous_num = usize::MAX;
    loop {
        let ports = get_usb_ports();
        if ports.len() != previous_num {
            previous_num = ports.len().to_owned();

            let _ = term.clear_screen();
            print_ports(&ports, &settings);
        }
        thread::sleep(time::Duration::from_millis(100));
    }
}

fn single_update(settings: ApplicationSettings) {
    let ports = get_usb_ports();
    print_ports(&ports, &settings);
}

type UsbPortVec = Vec<(String, settings::ComPort)>;

fn get_usb_ports() -> UsbPortVec {
    available_ports()
        .unwrap_or_default()
        .iter()
        .filter_map(|port| {
            if let SerialPortType::UsbPort(usbinfo) = &port.port_type {
                Some((
                    port.port_name.to_owned(),
                    settings::ComPort::from(usbinfo.to_owned()),
                ))
            } else {
                None
            }
        })
        .collect()
}

fn print_ports(ports: &UsbPortVec, settings: &ApplicationSettings) {
    if ports.len() == 0 {
        println!("No usb COM ports found.");
        return;
    }

    let mut serial_port_count: u16 = 0;
    ports.iter().for_each(|(port_name, com_port_info)| {
        let mut found_match = false;
        settings
            .file_settings
            .com_ports
            .iter()
            .for_each(|com_port_alias| {
                if settings.verbose {
                    println!("Checking\n{:?}\n{:?}\n", com_port_info, com_port_alias);
                }
                if com_port_info.fuzzy_eq(com_port_alias) {
                    found_match = true;
                    if !com_port_alias.alias.is_empty() {
                        println!("-------");
                        println!("{} {}", port_name, com_port_alias.alias);
                        serial_port_count += 1;
                    }
                }
            });

        if !found_match {
            println!("-------");
            println!("{}", port_name);
            println!("{}", com_port_info);
        }
    });

    if serial_port_count == 0 {
        println!("No COM ports found.")
    } else {
        println!("-------");
    }
}

fn print_com(alias: &String, settings: &ApplicationSettings) {
    let ports = get_usb_ports();
    let mut serial_port_count: u16 = 0;
    for (port_name, port) in ports {
        serial_port_count += 1;
        for com_port_alias in &settings.file_settings.com_ports {
            // Check if the alias matches the one we are looking for and return the COM number if it does
            if port.fuzzy_eq(com_port_alias) {
                if !com_port_alias.alias.is_empty() && &com_port_alias.alias == alias {
                    print!("{}", port_name);
                    return;
                }
            }
        }
    }
    if serial_port_count == 0 {
        println!("No COM ports found.")
    } else {
        println!("No COM port found with alias: {}", alias);
    }
}
