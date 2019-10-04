use std::vec::Vec;
pub mod opcode;
pub mod stack;

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

/// This represents the state of the chip-8 system including memory,
/// call stack, general purpose registers, program counter and screen buffer
#[derive(PartialEq, Debug)]
pub struct Chip {
    memory: Vec<u8>,
    stack: stack::Stack<u16>,
    registers: Vec<u8>,
    address: u16,
    program_counter: u16,
    screen_buffer: Vec<bool>, // @todo  refactor screen buffer into Vec<u8>
}

impl Default for Chip {
    fn default() -> Chip {
        Chip::new()
    }
}

impl Chip {
    /// Create a new, default initialized Chip struct
    pub fn new() -> Chip {
        Chip {
            memory: vec![0; 0xFFF],
            stack: stack::Stack::new(16),
            registers: vec![0; 16],
            address: 0,
            program_counter: 0x200,
            screen_buffer: vec![false; SCREEN_WIDTH * SCREEN_HEIGHT],
        }
    }

    /// Reset the Chip
    pub fn reset(&mut self) {
        *self = Chip::new();
    }

    /// Execute a single instruction
    pub fn tick(&mut self) {
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
        c.screen_buffer[0] = true;

        assert_ne!(c, Chip::new());

        c.reset();
        assert_eq!(c, Chip::new());
    }
}
