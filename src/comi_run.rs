use std::process::Command;
fn main() {
    Command::new("comi")
        .args(&["-c"])
        .spawn()
        .expect("Failed to find comi.exe");
}
