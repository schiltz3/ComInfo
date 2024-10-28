// Post build script

use std::env;
use std::process::Command;

fn main() {
    // Get the current profile from the environment
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    if profile == "release" {
        let output = Command::new("iscc")
            .arg("./InstallerScripts/ComiInstallerScript.iss")
            .output()
            .expect("Failed to execute post installer compiler");

        // Optionally, handle the output
        println!("Output: {:?}", String::from_utf8_lossy(&output.stdout));
        println!("Error: {:?}", String::from_utf8_lossy(&output.stderr));
    } else {
        println!("Skipping command execution. Not in release mode.")
    }
    println!("Skipping command execution. Not in release mode.")
}
