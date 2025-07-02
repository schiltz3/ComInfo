use directories::UserDirs;
use path_slash::PathBufExt;
use serde::Deserialize;
use serialport::UsbPortInfo;
use std::{env, fs, path::PathBuf};

#[derive(Deserialize, Debug)]
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

// TODO: Clean up implementation. IDK what I was on when I wrote it
// TODO: Write exact match, and a fuzzy match
impl FzyEq for ComPort {
    fn fuzzy_eq(&self, other: &Self) -> bool {
        let mut matched_element = 0;
        let mut matched = true;
        if other.product_id == self.product_id {
            matched = matched && true;
            matched_element += 1;
        } else {
            matched = false;
        }

        if other.serial_number == self.serial_number {
            matched = matched && true;
            matched_element += 1;
        } else {
            matched = false;
        }

        match other.manufacturer.as_ref() {
            Some(m) => match self.manufacturer.as_ref() {
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

        match other.product_name.as_ref() {
            Some(p) => match self.product_name.as_ref() {
                Some(pn) => {
                    if pn == p {
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
    // If there's no space, just return an empty string or the original string as per your requirement.
    input.to_string()
}

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub com_ports: Option<Vec<ComPort>>,
    pub verbose: bool,
}

pub fn validate_settings(settings: &Settings) -> Result<(), String> {
    return match settings.com_ports {
        Some(ref com_ports) => {
            for (i, com_port) in com_ports.iter().enumerate() {
                if com_port.serial_number.is_empty() {
                    return Err(format!("Serial number cannot be empty\n{:#?}", com_port));
                }
                for com_port_to_compare in com_ports.iter().skip(i + 1) {
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
                }
            }
            Ok(())
        }
        None => Ok(()),
    };
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
