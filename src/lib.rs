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
        self.program_counter += 2;
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
    fn decode_execute(&mut self, opcode: Opcode) {
        match opcode.n1() {
            0x0 => {
                match opcode.n4() {
                    0x0 => (), // clear screen,
                    0xE => (), // return
                    _ => (),   // illegal opcode
                }
            }
            0x1 => (), // jump to NNN
            0x2 => (), // call subroutine at NNN
            0x3 => (), // conditional skip
            0x4 => (), // conditional skip
            0x5 => (), // conditional skip
            0x6 => (), // set register to constant
            0x7 => (), // add constant to vx
            0x8 => {
                match opcode.n4() {
                    0x0 => (), // set vx to vy
                    0x1 => (), // vx = vx | vy
                    0x2 => (), // vx = vx & vy
                    0x3 => (), // vx = vx ^ vy,
                    0x4 => (), // vx += vy,
                    0x5 => (), // vx -= vy,
                    0x6 => (), // vf = vx & 1 : vx>>1
                    0x7 => (), // vx = vy - vx
                    0xE => (), // vx<<1
                    _ => (),   // illegal opcode
                }
            }
            0x9 => (), // skip if vx != vy
            0xA => (), // set addr register to NNN
            0xB => (), // jump to address + v0,
            0xC => (), // random bitwise and with constant
            0xD => (), // draw sprite at coordinate
            0xE => {
                match opcode.n3() {
                    0x9 => (), // skip next if key is present
                    0xE => (), // skip next if key is not present
                    _ => (),   // illegal opcode!
                }
            }
            0xF => {
                match opcode.n3() {
                    0x0 => {
                        match opcode.n4() {
                            0x7 => (), // set vx to the delay timer
                            0xA => (), // store keypress into vx
                            _ => (),   //illegial opcode
                        }
                    }
                    0x1 => {
                        match opcode.n4() {
                            0x5 => (), // set delay timer
                            0x8 => (), // set sound timer
                            0xE => (), // add vx to address register
                            _ => (),   // illegal opcode
                        }
                    }
                    0x2 => (), // set address register to sprite_addr[vx]
                    0x3 => (), // write binary coded decimal to address register
                    0x5 => (), // dump registers into memory
                    0x6 => (), // load registers from memory
                    _ => (),   // illegal opcode
                }
            }
            _ => (), // illegal opcode
        }
    }
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
