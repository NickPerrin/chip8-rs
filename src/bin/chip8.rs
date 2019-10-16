use chip8_rs;
use std::env;

fn main() {
    if let Some(rom_filename) = get_file_from_cli() {
        let mut chip = chip8_rs::Chip::new();

        match chip.load_rom(&rom_filename) {
            Ok(_) => println!("starting application"),
            Err(error) => eprintln!("Error loading rom: {}", error),
        }
    } else {
        eprintln!("Unable to parse rom filename");
    }
}

/// Get the filename from the command line.
/// Fragile implementation, either use clap or some kind of ui to choose a rom
fn get_file_from_cli() -> Option<String> {
    let mut args: Vec<String> = env::args().collect();
    if args.get(1).is_none() {
        None
    } else {
        Some(args.swap_remove(1))
    }
}


