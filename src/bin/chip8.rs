use chip8_rs;
use chip8_rs::DisplayWindow;
use minifb;
use std::env;

fn main() {
    if let Some(rom_filename) = get_file_from_cli() {
        let mut chip = chip8_rs::Chip::default();

        match chip.load_rom(&rom_filename) {
            Ok(_) => println!("starting application {}", rom_filename),
            Err(error) => eprintln!("Error loading rom: {}", error),
        }

        // @todo remove
        let scale_factor = 10_u8;
        let mut display =
            chip8_rs::RomWindow::new(scale_factor, &rom_filename, &chip);
        chip.screen_buffer[0] = 0xF0;

        while display.window.is_open()
            && !display.window.is_key_down(minifb::Key::Escape)
        {
            display.update(&chip.screen_buffer);
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
