use directories::UserDirs;
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
pub fn find_settings_path(args: &Option<PathBuf>) -> Option<PathBuf> {
    return match args.clone() {
        Some(settings_path) => {
            if settings_path.exists() {
                println!("Using {}", settings_path.to_str().unwrap());
                Some(settings_path)
            } else {
                None
            }
        }
        None => {
            // Look in default location
            let path = get_settings_path()?;
            if path.exists() {
                println!("Using {}", path.to_str().unwrap());
                Some(path)
            } else {
                None
            }
        }
    };
}

pub fn read_settings_from_file(settings_file_path: &Option<PathBuf>) -> Option<Settings> {
    return match settings_file_path {
        Some(settings_path) => {
            let string = fs::read_to_string(&settings_path).ok()?;
            return serde_json::from_str::<Settings>(string.as_str()).ok();
        }

        _ => None,
    };
}

pub fn install_settings_file() {
    match get_default_settings_path() {
        Some(default_settings) => {
            match get_settings_path() {
                Some(instal_path) => {
                    if !instal_path.exists() {
                        std::fs::copy(default_settings, instal_path).or_else(
                            // | err| println!( "Failed to copy {} to {}", default_settings .to_str() .unwrap_or("Unable to convert to string"), instal_path .to_str() .unwrap_or("Unable to convert to string"))
                        }
                        // Copy default_settings to instal_path
                    }
                
            
                None => {
                    println!("Failed to determine if Docs directory")
                }
            }
            }
        
        None => {
            println!("No settings.json in comi install dir. Cannot install settings.json to Docs/Comi/settings.json")
        }
    }
}
fn get_default_settings_path() -> Option<PathBuf> {
    let mut dir = env::current_dir().ok()?;
    dir.push("settings.json");
    return Some(dir);
}

fn get_settings_path() -> Option<PathBuf> {
    let mut path = UserDirs::new()?.document_dir()?.to_path_buf();
    path.push("Comi/settings.json");
    return Some(path);
}
