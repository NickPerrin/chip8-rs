use minifb::{Window, WindowOptions};
use std::fs;
use std::io;
use std::io::Read;
use std::vec::Vec;
pub mod opcode;
pub mod stack;

//const SCREEN_WIDTH: usize = 64;
//const SCREEN_HEIGHT: usize = 32;

#[derive(Debug)]
pub struct RomWindow {
    pub window: minifb::Window,
    pub scale_factor: u8,
    screen_width: usize,
    screen_height: usize,
}

pub trait DisplayWindow {
    fn update(&mut self, buffer: &[u8]);
}

impl RomWindow {
    pub fn new(scale_factor: u8, filename: &str, chip: &Chip) -> RomWindow {
        RomWindow {
            scale_factor,
            screen_width: chip.screen_width,
            screen_height: chip.screen_height,
            window: Window::new(
                filename,
                usize::from(chip.screen_width * scale_factor as usize),
                usize::from(chip.screen_height * scale_factor as usize),
                WindowOptions::default(),
            )
            .expect("Unable to "),
        }
    }

    /// Convert our monochrome bit to a 32
    fn bit_to_u32(bit: u8) -> u32 {
        match bit {
            0 => 0x00101010, // dark gray
            _ => 0x00EEEEEE, // light gray
        }
    }

    /// Expand a single byte to a partial screen buffer
    fn expand_byte(&self, byte: u8) -> Vec<u32> {
        let mut partial_buffer: Vec<u32> =
            Vec::with_capacity(8 * self.scale_factor as usize);
        for i in (0..=7).rev() {
            let bit = (byte >> i) & 0x1;
            for _ in 0..self.scale_factor {
                partial_buffer.push(RomWindow::bit_to_u32(bit));
            }
        }
        partial_buffer
    }

    fn expand_screen_buffer(&self, buffer: &[u8]) -> Vec<u32> {
        let mut screen_buffer: Vec<u32> = Vec::with_capacity(buffer.len() * 8);
        for y in 0..self.screen_height {
            for _ in 0..self.scale_factor {
                for x in 0..(self.screen_width / 8) {
                    screen_buffer.append(
                        &mut self.expand_byte(
                            buffer[x + (y * self.screen_width / 8)],
                        ),
                    );
                }
            }
        }
        screen_buffer
    }
}

impl DisplayWindow for RomWindow {
    fn update(&mut self, buffer: &[u8]) {
        self.window
            .update_with_buffer(&self.expand_screen_buffer(&buffer))
            .expect("Error updating the display\n");
    }
}

/// This represents the state of the chip-8 system including memory,
/// call stack, general purpose registers, program counter and screen buffer
pub struct Chip {
    memory: Vec<u8>,
    stack: stack::Stack<u16>,
    registers: Vec<u8>,
    address: u16,
    program_counter: u16,
    keys: Vec<bool>,
    pub screen_buffer: Vec<u8>,
    screen_width: usize,
    screen_height: usize,
    sound_timer: u8,
    delay_timer: u8,
}

impl Default for Chip {
    fn default() -> Chip {
        Chip::new(64, 32)
    }
}

impl Chip {
    /// Create a new, default initialized Chip struct
    pub fn new(screen_width: usize, screen_height: usize) -> Chip {
        let mut chip = Chip {
            memory: vec![0; 0xFFF],
            stack: stack::Stack::new(16),
            registers: vec![0; 16],
            address: 0,
            program_counter: 0x200,
            keys: vec![false; 16],
            screen_buffer: vec![
                0;
                usize::from(screen_width)
                    * usize::from(screen_height)
                    / 8 // size of u8
            ],
            screen_width,
            screen_height,
            delay_timer: 0,
            sound_timer: 0,
        };
        chip.init_fonts();
        chip
    }

    pub fn load_rom(&mut self, file: &str) -> Result<(), io::Error> {
        static MAX_ROM_SIZE: usize = 0x400;
        let mut f = fs::File::open(&file)?;

        let bytes_read = f.read(&mut self.memory[0x200..])?;
        if bytes_read > MAX_ROM_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Rom file too large",
            ));
        }
        Ok(())
    }

    /// Initialize fonts
    fn init_fonts(&mut self) {
        let fonts: [u8; 16 * 5] = [
            0xF0, 0x90, 0x90, 0x90, 0xf0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0x10, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];

        self.memory[0..16 * 5].copy_from_slice(&fonts);
    }

    /// Reset the Chip
    pub fn reset(&mut self) {
        *self = Chip::new(self.screen_width, self.screen_height);
    }

    /// Execute a single instruction
    pub fn tick(&mut self) {
        self.delay_timer = self.delay_timer.checked_sub(1).unwrap_or(0);
        self.sound_timer = self.sound_timer.checked_sub(1).unwrap_or(0);

        let opcode = self.get_next_opcode();
        opcode.decode_execute(self);
    }

    /// Read the next opcode from memory
    fn get_next_opcode(&mut self) -> opcode::Opcode {
        // @todo bounds checking
        // @todo figure out how to handle out of bounds

        let ms_byte = self.memory[usize::from(self.program_counter)];
        let ls_byte = self.memory[usize::from(self.program_counter + 1)];
        let opcode: u16 = u16::from(ms_byte) << 8;
        opcode::Opcode::new(opcode | u16::from(ls_byte))
    }

    /// Point the program counter to the next instruction.
    /// Each instruction is 2 bytes
    fn increment_program_counter(&mut self, step: Option<u16>) {
        match step {
            Some(i) => self.program_counter += 2 * i,
            None => self.program_counter += 2,
        }
    }

    /// Get the first pressed key from the keyboard
    fn get_pressed_key(&self) -> Option<usize> {
        for key in 0..self.keys.len() {
            if self.keys[key] {
                return Some(key);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn bit_to_u32() {
        for i in 0..=255 {
            if i == 0 {
                assert_eq!(0x00101010, RomWindow::bit_to_u32(i));
            } else {
                assert_eq!(0x00EEEEEE, RomWindow::bit_to_u32(i));
            }
        }
    }

    #[test]
    fn expand_byte() {
        let byte = 0;
        let expected: Vec<u32> = vec![
            0x00101010, 0x00101010, 0x00101010, 0x00101010, 0x00101010,
            0x00101010, 0x00101010, 0x00101010,
        ];
        assert_eq!(
            expected,
            RomWindow::new(1, "", &Chip::default()).expand_byte(byte)
        );

        let byte = 0b10000000;
        let expected: Vec<u32> = vec![
            0x00EEEEEE, 0x00101010, 0x00101010, 0x00101010, 0x00101010,
            0x00101010, 0x00101010, 0x00101010,
        ];

        assert_eq!(
            expected,
            RomWindow::new(1, "", &Chip::default()).expand_byte(byte)
        );

        let byte = 0b00000001;
        let expected: Vec<u32> = vec![
            0x00101010, 0x00101010, 0x00101010, 0x00101010, 0x00101010,
            0x00101010, 0x00101010, 0x00EEEEEE,
        ];

        assert_eq!(
            expected,
            RomWindow::new(1, "", &Chip::default()).expand_byte(byte)
        );
    }

    #[test]
    fn expand_screen_buffer() {
        let chip_buffer: Vec<u8> = vec![0, 0b10000000, 0b00000001];
        let expected = vec![
            0x00101010, 0x00101010, 0x00101010, 0x00101010, 0x00101010,
            0x00101010, 0x00101010, 0x00101010, 0x00EEEEEE, 0x00101010,
            0x00101010, 0x00101010, 0x00101010, 0x00101010, 0x00101010,
            0x00101010, 0x00101010, 0x00101010, 0x00101010, 0x00101010,
            0x00101010, 0x00101010, 0x00101010, 0x00EEEEEE,
        ];

        let window = RomWindow::new(1, "", &Chip::new(24, 1));

        assert_eq!(expected, window.expand_screen_buffer(&chip_buffer));
    }

    #[test]
    #[should_panic]
    fn load_rom_too_big() {
        let mut c = Chip::default();
        let filename = "./test.c8";
        let mut f = File::create(filename).unwrap();
        let data = vec![1; 0x401];

        f.write(&data[..]).unwrap();
        c.load_rom(filename).unwrap();
    }

    #[test]
    fn init_fonts() {
        let c = Chip::default();
        assert_eq!(c.memory[0], 0xF0);
    }
}
