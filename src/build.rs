// Post build script

use std::env;
use std::process::Command;

macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo::warning={}", format!($($tokens)*))
    }
}

fn main() {
    // Get the current profile from the environment
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let version = env!("CARGO_PKG_VERSION");

    if profile == "release" {
        let output = Command::new("iscc")
            .arg("./InstallerScripts/ComiInstallerScript.iss")
            .arg(format!("/DMyAppVersion={}", version))
            .output()
            .expect("Failed to execute post build installer compiler");

        // Handle the output if the command succeeded
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .for_each(|line| {
                p!("{}", line);
            });

        String::from_utf8_lossy(&output.stderr)
            .lines()
            .for_each(|line| {
                p!("{}", line);
            });
    } else {
        p!("Skipping command execution. Not in release mode.")
    }
}
