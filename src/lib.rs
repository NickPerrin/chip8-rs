use std::vec::Vec;
pub mod opcode;
pub mod stack;

const SCREEN_WIDTH: u8 = 64;
const SCREEN_HEIGHT: u8 = 32;

/// This represents the state of the chip-8 system including memory,
/// call stack, general purpose registers, program counter and screen buffer
#[derive(PartialEq, Debug)]
pub struct Chip {
    memory: Vec<u8>,
    stack: stack::Stack<u16>,
    registers: Vec<u8>,
    address: u16,
    program_counter: u16,
    keys: Vec<bool>,
    screen_buffer: Vec<u8>,
    screen_width: u8,
    screen_height: u8,
    sound_timer: u8,
    delay_timer: u8,
}

impl Default for Chip {
    fn default() -> Chip {
        Chip::new()
    }
}

impl Chip {
    /// Create a new, default initialized Chip struct
    pub fn new() -> Chip {
        let mut chip = Chip {
            memory: vec![0; 0xFFF],
            stack: stack::Stack::new(16),
            registers: vec![0; 16],
            address: 0,
            program_counter: 0x200,
            keys: vec![false; 16],
            screen_buffer: vec![
                0;
                usize::from(SCREEN_WIDTH)
                    * usize::from(SCREEN_HEIGHT)
            ],
            screen_width: SCREEN_WIDTH,
            screen_height: SCREEN_HEIGHT,
            delay_timer: 0,
            sound_timer: 0,
        };
        chip.init_fonts();
        chip
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
        *self = Chip::new();
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

    #[test]
    fn reset_chip() {
        let mut c = Chip::new();
        c.memory[1] = 2;
        c.stack.push(2).unwrap();
        c.registers[0xf] = 2;
        c.address = 100;
        c.program_counter = 1;
        c.screen_buffer[0] = 0;

        assert_ne!(c, Chip::new());

        c.reset();
        assert_eq!(c, Chip::new());
    }

    #[test]
    fn init_fonts() {
        let c = Chip::new();
        assert_eq!(c.memory[0], 0xF0);
    }
}
