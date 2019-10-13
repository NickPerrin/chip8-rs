use chip8_rs;
use std::env;

use piston_window::*;

fn main() {
    if let Some(rom_filename) = get_file_from_cli() {
        let mut chip = chip8_rs::Chip::new();

        match chip.load_rom(&rom_filename) {
            Ok(_) => start_event_loop(&mut chip),
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

fn init_window() -> PistonWindow {
    let title = "Chip8-rs";
    let mut window: PistonWindow = WindowSettings::new(title, [640, 320])
        .exit_on_esc(true)
        .resizable(false)
        .decorated(false)
        .build()
        .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e));
    window.set_lazy(true);
    window
}

fn start_event_loop(chip: &mut chip8_rs::Chip) {
    let mut window: PistonWindow = init_window();
    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g, _| {
            clear([0.1, 0.1, 0.1, 1.0], g);
            rectangle(
                [0.8, 0.8, 0.8, 1.0],
                [50.0, 50.0, 10.0, 10.0],
                c.transform,
                g,
            );
            rectangle(
                [0.5, 0.5, 0.5, 1.0],
                [80.0, 80.0, 10.0, 10.0],
                c.transform,
                g,
            );
        });

        if e.press_args().is_some() {
            println!("key has been pressed");
        }
    }
}
