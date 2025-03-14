use directories::UserDirs;
use path_slash::PathBufExt;
use serde::Deserialize;
use std::{env, fs, path::PathBuf};

#[derive(Deserialize, Debug)]
pub struct ComPort {
    pub alias: String,
    pub product_id: u16,
    pub serial_number: String,
    pub manufacturer: Option<String>,
    pub product_name: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub com_ports: Option<Vec<ComPort>>,
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
