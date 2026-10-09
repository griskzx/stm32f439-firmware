//! Host-side entry point for future flashing and OTA packaging tasks.

fn main() {
    match std::env::args().nth(1).as_deref() {
        None | Some("help" | "--help" | "-h") => {
            println!("Usage: cargo xtask [help]");
            println!("Flashing and OTA packaging commands are not implemented yet.");
        }
        Some(command) => {
            eprintln!("Unknown command: {command}");
            eprintln!("Run `cargo xtask help` for usage.");
            std::process::exit(2);
        }
    }
}
