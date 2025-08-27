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
    let mut dir = env::current_dir().ok()?;
    dir.push("settings.json");
    return Some(dir);
}

// Get the settings file path in user documents
fn get_settings_path() -> Option<PathBuf> {
    let mut path = UserDirs::new()?.document_dir()?.to_path_buf();
    path.push("Comi\\settings.json");
    return Some(path);
}
