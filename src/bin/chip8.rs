use chip8_rs;
use chip8_rs::{DisplayWindow, Key, KeyState};
use minifb;
use std::env;

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

/// Map a minifb key to the corresponding chip8 key
/// _________________
/// | 1 / 2 / 3 / c |
/// | 4 / 5 / 6 / D |
/// | 7 / 8 / 9 / E |
/// | A / 0 / B / F |
/// -----------------
/// Optionally returns the chip8 index of the key pressed
fn map_key(key: &minifb::Key) -> Option<usize> {
    match key {
        minifb::Key::Key1 => Some(0x1_usize),
        minifb::Key::Key2 => Some(0x2_usize),
        minifb::Key::Key3 => Some(0x3_usize),
        minifb::Key::Key4 => Some(0xc_usize),

        minifb::Key::Q => Some(0x4_usize),
        minifb::Key::W => Some(0x5_usize),
        minifb::Key::E => Some(0x6_usize),
        minifb::Key::R => Some(0xd_usize),

        minifb::Key::A => Some(0x7_usize),
        minifb::Key::S => Some(0x8_usize),
        minifb::Key::D => Some(0x9_usize),
        minifb::Key::F => Some(0xe_usize),

        minifb::Key::Z => Some(0xa_usize),
        minifb::Key::X => Some(0x0_usize),
        minifb::Key::C => Some(0xb_usize),
        minifb::Key::V => Some(0xf_usize),

        _ => None,
    }
}

// @todo figure out a way to remap keys inside the app
/// This maps a list of minifb::Keys to Chip::keys
fn map_keys(keys: Vec<minifb::Key>) -> Vec<Key> {
    let mut chip_keys = vec![
        Key {
            state: KeyState::NotPressed,
        };
        16
    ];

    for key in keys.iter() {
        if let Some(index) = map_key(key) {
            chip_keys[index].state = KeyState::Pressed;
        }
    }

    chip_keys
}

/// Execute an atomic step through the system. Read user input, execute a cpu instruction, update
/// the display.
fn tick(chip: &mut chip8_rs::Chip, display: &mut chip8_rs::RomWindow) {
    if let Some(keys) = display.window.get_keys() {
        chip.update_keys(map_keys(keys));
    }

    println!("{:?}", chip.keys);
}

fn main() {
    if let Some(rom_filename) = get_file_from_cli() {
        let mut chip = chip8_rs::Chip::default();

        match chip.load_rom(&rom_filename) {
            Ok(_) => println!("starting application {}", rom_filename),
            Err(error) => eprintln!("Error loading rom: {}", error),
        }

        // @todo remove
        let scale_factor = 10_u8;
        let mut display = chip8_rs::RomWindow::new(scale_factor, &rom_filename, &chip);
        chip.screen_buffer[0] = 0xF0;
        while display.window.is_open() && !display.window.is_key_down(minifb::Key::Escape) {
            display.update(&chip.screen_buffer);
        }
        tick(&mut chip, &mut display);
    } else {
        eprintln!("Unable to parse rom filename");
    }
}
