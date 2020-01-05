use chip8_rs;
use chip8_rs::{DisplayWindow, Key, KeyState};
use minifb;
use std::convert::TryFrom;
use std::{env, process, thread, time};

use chip8_rs::debugger::{Chip8Debugger, Debugger};

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
fn map_key(key: minifb::Key) -> Option<usize> {
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
        if let Some(index) = map_key(*key) {
            chip_keys[index].state = KeyState::Pressed;
        }
    }

    chip_keys
}

/// Execute an atomic step through the system. Read user input, execute a cpu instruction, update
/// the display.
fn tick(chip: &mut chip8_rs::Chip, display: &mut chip8_rs::RomWindow) -> Option<()> {
    if let Some(keys) = display.window.get_keys() {
        chip.update_keys(map_keys(keys));
    }

    // Execute the next instruction
    chip.tick();

    // Update the display
    if display.window.is_open() && !display.window.is_key_down(minifb::Key::Escape) {
        display.update(&chip.screen_buffer);
        return Some(());
    }
    None
}

/// Convert a given refresh rate into the corresponding delay between frames. Only 'reasonable'
/// refresh rates will be used. Reasonable is defined as [20, 300]. Anything outside of this range
/// will be set to the default of 60Hz
fn refresh_rate_to_delay_milliseconds(refresh_rate: u16) -> u64 {
    let mut f64_refresh_delay = 60_f64;
    if let Ok(f64_refresh_rate) = f64::try_from(refresh_rate) {
        if f64_refresh_rate >= 20_f64 && f64_refresh_rate <= 300_f64 {
            f64_refresh_delay = (1_f64 / f64_refresh_rate) * 1000_f64;
        }
    }

    // f64_refresh_delay is guarenteed to be positive and smaller than max<u64> so we can truncate
    // and convert to u64 without fear
    f64_refresh_delay as u64
}

/// Call tick at a given refresh rate. This is the default mode of execution for the emulator.
///
/// A note about timing: The refresh rate parameter deterimes the delay applied uniformly after
///                      call to tick(). This means that each call to tick() is assumed to take no
///                      time. So the practical refresh rate will be always be lower than the
///                      confgured refresh reate. This could be fixed by timing the call to tick()
///                      and dynamically adjusting the delay for a fixed refresh rate, assuming
///                      tick() doesn't take too long. I seriously doubt that this will matter, so
///                      I probably won't bother.
fn run(refresh_rate: u16, mut chip: chip8_rs::Chip, mut display: chip8_rs::RomWindow) {
    let refresh_delay =
        time::Duration::from_millis(refresh_rate_to_delay_milliseconds(refresh_rate));
    while let Some(_) = tick(&mut chip, &mut display) {
        thread::sleep(refresh_delay);
    }
}

fn run_debug(mut chip: chip8_rs::Chip, mut display: chip8_rs::RomWindow) {
    // @todo create debugger module with single step, various print modes, restart, and handoff
    // @todo add capability to write to memory and registers. Currently chip is 'read only'

    let debugger = chip8_rs::debugger::Chip8Debugger::new(&chip);
    debugger.welcome();
    loop {
        if let Some(command) = debugger.get_user_input() {
            // TODO
            //  if step, call tick(),
            //  if continue, pass control to run,
            //  if quit, break and return,
            println!("valid command: {:?}", command);
        } else {
            println!("Invalid command. Use help (h) to show valid commands");
        }
    }
}

fn main() {
    if let Some(rom_filename) = get_file_from_cli() {
        let mut chip = chip8_rs::Chip::default();

        match chip.load_rom(&rom_filename) {
            Ok(_) => println!("starting application {}", rom_filename),
            Err(error) => {
                eprintln!("Error loading rom: {}", error);
                process::exit(1);
            }
        }

        let scale_factor = 10_u8;
        let display = chip8_rs::RomWindow::new(scale_factor, &rom_filename, &chip);

        // TODO add better cmdline parsing, add debug option
        run_debug(chip, display);
    //run(60, chip, display);
    } else {
        eprintln!("Unable to parse rom filename");
    }
}
