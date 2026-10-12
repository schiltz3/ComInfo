use directories::UserDirs;
use path_slash::PathBufExt;
use serde::Deserialize;
use serde::Serialize;
use serialport::UsbPortInfo;
use std::{env, fmt, fs, path::PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ComPort {
    pub alias: String,
    pub product_id: u16,
    pub serial_number: String,
    pub manufacturer: Option<String>,
    pub product_name: Option<String>,
    // USB interface number. Tells apart the COM ports of a composite device that share one PID and serial number
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interface: Option<u8>,
}
pub trait FzyEq {
    fn fuzzy_eq(&self, other: &Self) -> bool;
}

impl fmt::Display for ComPort {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.alias.is_empty() {
            write!(f, "{}", self.alias)?;
        }
        write!(
            f,
            "\tProduct: {}\n",
            self.product_name.clone().unwrap_or_default()
        )?;
        write!(
            f,
            "\tManufacturer: {}\n",
            self.manufacturer.clone().unwrap_or_default()
        )?;
        write!(f, "\tPid: {}\n", self.product_id)?;
        write!(f, "\tSerial Number: {}", self.serial_number.clone())?;
        if let Some(interface) = self.interface {
            write!(f, "\n\tInterface: {}", interface)?;
        }
        // result
        Ok(())
    }
}

impl PartialEq for ComPort {
    fn eq(&self, other: &Self) -> bool {
        let mut eq = true;
        eq = eq && self.alias == other.alias;
        eq = eq && self.product_id == other.product_id;
        eq = eq && self.serial_number == other.serial_number;
        eq = eq && self.manufacturer == other.manufacturer;
        eq = eq && self.product_name == other.product_name;
        if self.product_name.is_some() && other.product_name.is_some() {
            eq = eq && self.product_name.as_ref().unwrap() == other.product_name.as_ref().unwrap();
        }
        eq = eq && self.manufacturer == other.manufacturer;
        if self.manufacturer.is_some() && other.manufacturer.is_some() {
            eq = eq && self.manufacturer.as_ref().unwrap() == other.manufacturer.as_ref().unwrap();
        }
        eq = eq && self.interface == other.interface;
        return eq;
    }
}

// Only match required fields
impl FzyEq for ComPort {
    fn fuzzy_eq(&self, other: &Self) -> bool {
        let mut eq = true;
        eq = eq && self.product_id == other.product_id;
        eq = eq && self.serial_number == other.serial_number;
        eq = eq && self.manufacturer == other.manufacturer;
        eq = eq && self.product_name == other.product_name;
        // A missing interface on either side matches any interface. Settings files written before this field existed have none, and Windows reports none for non-composite devices while Linux reports 0, so a strict compare would break settings shared between the two.
        eq = eq
            && (self.interface.is_none()
                || other.interface.is_none()
                || self.interface == other.interface);
        return eq;
    }
}

// Convert from a UsbPortInfo to a ComPort
impl From<UsbPortInfo> for ComPort {
    fn from(port_info: UsbPortInfo) -> Self {
        ComPort {
            alias: "".to_string(),
            product_id: port_info.pid,
            serial_number: port_info.serial_number.unwrap_or("".to_string()),
            manufacturer: port_info.manufacturer,
            product_name: Some(remove_last_word(
                port_info.product.unwrap_or("".to_string()).as_str(),
            )),
            interface: port_info.interface,
        }
    }
}

fn remove_last_word(input: &str) -> String {
    if let Some(last_space_idx) = input.rfind(' ') {
        let new_string = input[..last_space_idx].to_string();
        return new_string;
    }
    // If there's no space, just return an empty string or the original string.
    input.to_string()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub com_ports: Vec<ComPort>,
}

pub fn validate_settings(settings: &Settings) -> Result<(), String> {
    for (i, com_port) in settings.com_ports.iter().enumerate() {
        if com_port.serial_number.is_empty() {
            return Err(format!("Serial number cannot be empty\n{:#?}", com_port));
        }
        for com_port_to_compare in settings.com_ports.iter().skip(i + 1) {
            if com_port_to_compare.serial_number.is_empty() {
                return Err(format!(
                    "Serial number cannot be empty\n{:#?}",
                    com_port_to_compare
                ));
            }
            if com_port_to_compare == com_port {
                return Err(format!(
                    "Duplicate ComPort found \n\"{:#?}\" \n.\n.\n.\n\"{:#?}\"",
                    com_port, com_port_to_compare
                ));
            }
            if com_port_to_compare.fuzzy_eq(com_port) {
                return Err(format!(
                    "Similar potentially conflicting ComPort found \n\"{:#?}\" \n.\n.\n.\n\"{:#?}\"",
                    com_port, com_port_to_compare
                ));
            }
        }
    }
    Ok(())
}

// Search for settings file in where the user specified, or in the default location
pub fn find_settings_path(args: &Option<PathBuf>, verbose: bool) -> Option<PathBuf> {
    return match args.clone() {
        Some(settings_path) => {
            if settings_path.exists() {
                if verbose {
                    println!("Using \"{}\"", settings_path.to_slash().unwrap());
                }
                Some(settings_path)
            } else {
                None
            }
        }
        None => {
            // Look in default location
            let path = get_settings_path()?;
            if path.exists() {
                if verbose {
                    println!("Using \"{}\"", path.to_slash().unwrap());
                }
                Some(path)
            } else {
                None
            }
        }
    };
}

// Get the json from a settings file as a string
pub fn read_settings_from_file(settings_file_path: &Option<PathBuf>) -> Option<Settings> {
    return match settings_file_path {
        Some(settings_path) => {
            let string = fs::read_to_string(&settings_path).ok()?;
            return serde_json::from_str::<Settings>(string.as_str()).ok();
        }

        _ => None,
    };
}

pub fn write_setting_to_file(
    settings_file_path: &Option<PathBuf>,
    com_ports: Vec<ComPort>,
) -> Result<i32, ()> {
    if settings_file_path.is_none() {
        print!("Cannot save settings file because no valid path was found");
        return Err(());
    }
    let mut settings_written = 0;

    // Open settings file, read it, add com ports if they don't already exist, and write it back
    let mut settings =
        read_settings_from_file(settings_file_path).unwrap_or(Settings { com_ports: vec![] });

    let com_ports = drop_unneeded_interfaces(&com_ports);

    // Split before adding new ports so the ports of a split device aren't also added as new hidden entries
    let splits = find_entries_to_split(&settings, &com_ports);
    for (index, interfaces) in &splits {
        let alias = &settings.com_ports[*index].alias;
        if interfaces.is_empty() {
            println!("Removed \"{}\" because every COM port of that device already has its own entry.", alias);
        } else {
            println!(
                "\"{}\" has several COM ports, so its entry was split into one entry per interface {:?}. Edit their aliases in the settings file to tell them apart.",
                alias, interfaces
            );
        }
    }
    split_entries(&mut settings, &splits);

    for port in com_ports {
        if !settings.com_ports.iter().any(|p| p.fuzzy_eq(&port)) {
            settings.com_ports.push(port);
            settings_written += 1;
        }
    }
    let json = serde_json::to_string_pretty(&settings).unwrap();
    fs::write(settings_file_path.as_ref().unwrap(), json).unwrap();
    return Ok(settings_written);
}

// Only composite devices, which expose several COM ports under one PID and serial number, need an interface to tell their ports apart. Dropping it everywhere else keeps settings files the same on Windows and Linux, since Linux reports interface 0 for single-port devices.
fn drop_unneeded_interfaces(ports: &[ComPort]) -> Vec<ComPort> {
    ports
        .iter()
        .map(|port| {
            let ports_on_device = ports
                .iter()
                .filter(|p| p.product_id == port.product_id && p.serial_number == port.serial_number)
                .count();
            let mut port = port.clone();
            if ports_on_device < 2 {
                port.interface = None;
            }
            port
        })
        .collect()
}

// Find settings entries without an interface that match the ports of a composite device. Returns each entry's index with the interfaces it should be split into. An empty list means every interface already has its own entry, so the old entry should be removed. `ports` must already have gone through drop_unneeded_interfaces, otherwise a single-port device on Linux looks composite.
fn find_entries_to_split(settings: &Settings, ports: &[ComPort]) -> Vec<(usize, Vec<u8>)> {
    let mut splits = Vec::new();
    for (index, entry) in settings.com_ports.iter().enumerate() {
        if entry.interface.is_some() {
            continue;
        }
        let mut device_interfaces: Vec<u8> = ports
            .iter()
            .filter(|port| entry.fuzzy_eq(port))
            .filter_map(|port| port.interface)
            .collect();
        if device_interfaces.is_empty() {
            continue;
        }
        device_interfaces.sort();
        device_interfaces.dedup();
        // An interface the user already named in another entry doesn't need a copy of this one
        let unnamed_interfaces: Vec<u8> = device_interfaces
            .into_iter()
            .filter(|interface| {
                !settings
                    .com_ports
                    .iter()
                    .any(|other| other.interface == Some(*interface) && other.fuzzy_eq(entry))
            })
            .collect();
        splits.push((index, unnamed_interfaces));
    }
    splits
}

// Replace each entry with one copy per interface, or remove it if the list is empty. The alias is kept on every copy so the ports keep their current name until the user edits them.
fn split_entries(settings: &mut Settings, splits: &[(usize, Vec<u8>)]) {
    // Reverse so earlier indices stay valid while splicing
    for (index, interfaces) in splits.iter().rev() {
        let entry = settings.com_ports[*index].clone();
        let copies: Vec<ComPort> = interfaces
            .iter()
            .map(|interface| ComPort {
                interface: Some(*interface),
                ..entry.clone()
            })
            .collect();
        settings.com_ports.splice(*index..*index + 1, copies);
    }
}

// True if `comi --save` would split or remove a settings entry, so the user can be told to run it
pub fn needs_interface_split(settings: &Settings, ports: &[ComPort]) -> bool {
    !find_entries_to_split(settings, &drop_unneeded_interfaces(ports)).is_empty()
}

// Check if settings file exists and create it if it does not
pub fn install_settings_file() -> Result<u64, String> {
    match get_default_settings_path() {
        Some(default_settings) => match get_settings_path() {
            Some(instal_path) => {
                if !instal_path.exists() {
                    // Create the Comi directory
                    fs::create_dir_all(
                        &instal_path
                            .parent()
                            .expect("default settings file has no parent"),
                    )
                    .or_else(|e| {
                        let fmt_err =
                            format!("Error creating dir for settings file {}", e.to_string());
                        return Err(fmt_err);
                    })?;

                    // Create the new file
                    fs::File::create_new(&instal_path).or_else(|e| {
                        let fmt_err = format!("Error creating File {}", e.to_string());
                        return Err(fmt_err);
                    })?;

                    // Copy the file contents
                    return std::fs::copy(&default_settings, &instal_path).or_else(|err| {
                        let fmt_err = format!(
                            "Failed to copy {} to {} with error {}",
                            default_settings
                                .to_str()
                                .unwrap_or("Unable to convert path to string"),
                            instal_path
                                .to_str()
                                .unwrap_or("Unable to convert path to string"),
                            err.to_string()
                        );
                        return Err(fmt_err);
                    });
                } else {
                    // Return an ok because the file already exists
                    //TODO Could check scheme here and update if needed
                    return Ok(0);
                }
            }

            None => {
                return Err("Failed to determine if Docs directory".to_string());
            }
        },

        None => {
            return Err("No settings.json in comi install dir. Cannot install settings.json to Docs/Comi/settings.json".to_string());
        }
    };
}

// Get the template settings file
fn get_default_settings_path() -> Option<PathBuf> {
    let mut dir = env::current_exe().ok()?;
    dir = dir.parent()?.to_path_buf();
    dir.push("settings.json");
    return Some(dir);
}

// Get the settings file path in user documents
fn get_settings_path() -> Option<PathBuf> {
    let mut path = UserDirs::new()?.document_dir()?.to_path_buf();
    path.push("Comi\\settings.json");
    return Some(path);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn port(interface: Option<u8>) -> ComPort {
        ComPort {
            alias: "".to_string(),
            product_id: 22336,
            serial_number: "20643975524E".to_string(),
            manufacturer: Some("Microsoft".to_string()),
            product_name: Some("USB Serial Device".to_string()),
            interface,
        }
    }

    #[test]
    fn same_interface_matches() {
        assert!(port(Some(2)).fuzzy_eq(&port(Some(2))));
    }

    #[test]
    fn different_interface_does_not_match() {
        assert!(!port(Some(0)).fuzzy_eq(&port(Some(2))));
        assert!(!port(Some(2)).fuzzy_eq(&port(Some(0))));
    }

    #[test]
    fn missing_interface_matches_any_interface() {
        // Both directions matter. Detected ports are checked against settings and settings entries against each other.
        assert!(port(None).fuzzy_eq(&port(Some(0))));
        assert!(port(Some(0)).fuzzy_eq(&port(None)));
        assert!(port(None).fuzzy_eq(&port(None)));
    }

    #[test]
    fn interface_does_not_override_other_fields() {
        let mut other_serial = port(Some(2));
        other_serial.serial_number = "OTHER".to_string();
        assert!(!port(Some(2)).fuzzy_eq(&other_serial));
        assert!(!port(None).fuzzy_eq(&other_serial));
    }

    #[test]
    fn exact_eq_compares_interface() {
        assert!(port(Some(2)) == port(Some(2)));
        assert!(port(Some(0)) != port(Some(2)));
        assert!(port(None) != port(Some(0)));
    }

    #[test]
    fn display_shows_interface_when_present() {
        assert!(port(Some(2)).to_string().ends_with("\n\tInterface: 2"));
    }

    #[test]
    fn display_omits_interface_when_absent() {
        assert!(!port(None).to_string().contains("Interface"));
    }

    #[test]
    fn settings_entry_without_interface_deserializes_as_none() {
        let json = r#"{"alias": "A", "product_id": 1, "serial_number": "1", "manufacturer": null, "product_name": null}"#;
        let parsed: ComPort = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.interface, None);
    }

    #[test]
    fn interface_round_trips_and_is_omitted_when_none() {
        let with = serde_json::to_string(&port(Some(2))).unwrap();
        assert!(with.contains("\"interface\":2"));
        assert_eq!(serde_json::from_str::<ComPort>(&with).unwrap().interface, Some(2));

        let without = serde_json::to_string(&port(None)).unwrap();
        assert!(!without.contains("interface"));
    }

    fn named(alias: &str, interface: Option<u8>) -> ComPort {
        ComPort {
            alias: alias.to_string(),
            ..port(interface)
        }
    }

    #[test]
    fn single_port_device_drops_interface() {
        // Linux reports interface 0 for a plain single-port adapter
        let cleaned = drop_unneeded_interfaces(&[port(Some(0))]);
        assert_eq!(cleaned[0].interface, None);
    }

    #[test]
    fn composite_device_keeps_interfaces() {
        let cleaned = drop_unneeded_interfaces(&[port(Some(0)), port(Some(2))]);
        assert_eq!(cleaned[0].interface, Some(0));
        assert_eq!(cleaned[1].interface, Some(2));
    }

    #[test]
    fn legacy_entry_for_composite_device_is_split() {
        let mut settings = Settings {
            com_ports: vec![named("Before", None), named("M18 Comms Adapter", None), named("After", Some(9))],
        };
        // "Before" and "After" are other devices
        settings.com_ports[0].serial_number = "OTHER1".to_string();
        settings.com_ports[2].serial_number = "OTHER2".to_string();
        let detected = drop_unneeded_interfaces(&[port(Some(2)), port(Some(0))]);

        let splits = find_entries_to_split(&settings, &detected);
        assert_eq!(splits, vec![(1, vec![0, 2])]);

        split_entries(&mut settings, &splits);
        let aliases: Vec<(&str, Option<u8>)> = settings.com_ports.iter().map(|p| (p.alias.as_str(), p.interface)).collect();
        assert_eq!(
            aliases,
            vec![("Before", None), ("M18 Comms Adapter", Some(0)), ("M18 Comms Adapter", Some(2)), ("After", Some(9))]
        );
    }

    #[test]
    fn legacy_entry_for_single_port_device_is_not_split() {
        let settings = Settings {
            com_ports: vec![named("FTDI", None)],
        };
        assert!(!needs_interface_split(&settings, &[port(Some(0))]));
    }

    #[test]
    fn interface_named_by_another_entry_is_not_copied() {
        let settings = Settings {
            com_ports: vec![named("Whole device", None), named("Port B", Some(2))],
        };
        let detected = drop_unneeded_interfaces(&[port(Some(0)), port(Some(2))]);
        assert_eq!(find_entries_to_split(&settings, &detected), vec![(0, vec![0])]);
    }

    #[test]
    fn entries_with_interfaces_need_no_split() {
        let settings = Settings {
            com_ports: vec![named("Port A", Some(0)), named("Port B", Some(2))],
        };
        assert!(!needs_interface_split(&settings, &[port(Some(0)), port(Some(2))]));
    }

    #[test]
    fn split_happens_only_once() {
        let mut settings = Settings {
            com_ports: vec![named("M18 Comms Adapter", None)],
        };
        let detected = [port(Some(0)), port(Some(2))];
        let splits = find_entries_to_split(&settings, &drop_unneeded_interfaces(&detected));
        split_entries(&mut settings, &splits);

        // A second `comi --save` must leave the split entries alone
        assert!(!needs_interface_split(&settings, &detected));
        assert_eq!(settings.com_ports.len(), 2);
    }

    #[test]
    fn legacy_entry_with_every_interface_named_is_removed() {
        let mut settings = Settings {
            com_ports: vec![named("Whole device", None), named("Port A", Some(0)), named("Port B", Some(2))],
        };
        let detected = [port(Some(0)), port(Some(2))];
        assert!(needs_interface_split(&settings, &detected));

        let splits = find_entries_to_split(&settings, &drop_unneeded_interfaces(&detected));
        assert_eq!(splits, vec![(0, Vec::<u8>::new())]);

        split_entries(&mut settings, &splits);
        let aliases: Vec<&str> = settings.com_ports.iter().map(|p| p.alias.as_str()).collect();
        assert_eq!(aliases, vec!["Port A", "Port B"]);
        assert!(!needs_interface_split(&settings, &detected));
    }

    #[test]
    fn legacy_entry_for_other_device_is_not_removed() {
        let mut other_device = named("Other device", None);
        other_device.serial_number = "OTHER".to_string();
        let settings = Settings {
            com_ports: vec![other_device, named("Port A", Some(0)), named("Port B", Some(2))],
        };
        assert!(!needs_interface_split(&settings, &[port(Some(0)), port(Some(2))]));
    }

    #[test]
    fn disconnected_composite_device_needs_no_split() {
        let settings = Settings {
            com_ports: vec![named("M18 Comms Adapter", None)],
        };
        assert!(!needs_interface_split(&settings, &[]));
    }
}
