use std::process::Command;

fn main() {
    // run srcipts/asm_replace.py
    Command::new("python")
        .arg("scripts/asm_replace.py")
        .status()
        .expect("Failed to run asm_replace.py");
}