use std::vec::Vec;
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
            registers: vec![0; 24],
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
        // fetch opcode
        // decode opcode
        // execute instruction

        let opcode = self.get_next_opcode();
        self.decode_execute(opcode);
    }

    /// Read the next opcode from memory
    fn get_next_opcode(&mut self) -> Opcode {
        // @todo bounds checking
        // @todo figure out how to handle out of bounds

        let ms_byte = self.memory[usize::from(self.program_counter)];
        let ls_byte = self.memory[usize::from(self.program_counter + 1)];
        let opcode: u16 = u16::from(ms_byte) << 8;
        self.program_counter += 2;
        Opcode::new(opcode | u16::from(ls_byte))
    }

    /// Decode and execute a single instruction
    ///
    /// # Arguments
    ///
    /// opcode The opcode to be executed
    fn decode_execute(&mut self, opcode: Opcode) {}
}

/// Represents a single 2 byte opcode and provides convenient access to each
/// nibble
pub struct Opcode {
    opcode: u16,
}

impl Opcode {
    pub fn new(opcode: u16) -> Opcode {
        Opcode { opcode }
    }

    pub fn n1(&self) -> u16 {
        (self.opcode & 0xF000) >> 12
    }

    pub fn n2(&self) -> u16 {
        (self.opcode & 0x0f00) >> 8
    }

    pub fn n3(&self) -> u16 {
        (self.opcode & 0x00f0) >> 4
    }

    pub fn n4(&self) -> u16 {
        self.opcode & 0x000F
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opcode_new() {
        let o = Opcode::new(11);
        assert_eq!(o.opcode, 11);
    }

    #[test]
    fn trivial_split() {
        let o = Opcode::new(0);
        assert_eq!(o.n1(), 0);
        assert_eq!(o.n2(), 0);
        assert_eq!(o.n3(), 0);
        assert_eq!(o.n4(), 0);
    }

    #[test]
    fn split_opcode() {
        let o = Opcode::new(0x1234);
        assert_eq!(o.n1(), 1);
        assert_eq!(o.n2(), 2);
        assert_eq!(o.n3(), 3);
        assert_eq!(o.n4(), 4);
    }

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
