use super::Chip;

/// Represents a single 2 byte opcode and provides convenient access to each
/// nibble
pub struct Opcode {
    opcode: u16,
}

impl Opcode {
    pub fn new(opcode: u16) -> Opcode {
        Opcode { opcode }
    }

    fn n1(&self) -> u16 {
        (self.opcode & 0xF000) >> 12
    }

    fn n2(&self) -> u16 {
        (self.opcode & 0x0f00) >> 8
    }

    fn n3(&self) -> u16 {
        (self.opcode & 0x00f0) >> 4
    }

    fn n4(&self) -> u16 {
        self.opcode & 0x000F
    }

    /// Decode and execute a single instruction
    ///
    /// # Arguments
    ///
    /// opcode The opcode to be executed
    pub fn decode_execute(&self, mut chip: &mut Chip) {
        match self.n1() {
            0x0 => match self.n4() {
                0x0 => Opcode::clear_screen(&mut chip),
                0xE => (Opcode::return_from_subroutine(&mut chip)),
                _ => (panic!("Illegal opcode! {}", self.opcode)),
            },
            0x1 => (), // jump to NNN
            0x2 => (), // call subroutine at NNN
            0x3 => (), // conditional skip
            0x4 => (), // conditional skip
            0x5 => (), // conditional skip
            0x6 => (), // set register to constant
            0x7 => (), // add constant to vx
            0x8 => {
                match self.n4() {
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
                match self.n3() {
                    0x9 => (), // skip next if key is present
                    0xE => (), // skip next if key is not present
                    _ => (),   // illegal opcode!
                }
            }
            0xF => {
                match self.n3() {
                    0x0 => {
                        match self.n4() {
                            0x7 => (), // set vx to the delay timer
                            0xA => (), // store key press into vx
                            _ => (),   //illegal opcode
                        }
                    }
                    0x1 => {
                        match self.n4() {
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

    fn clear_screen(chip: &mut Chip) {
        chip.screen_buffer.clear();
    }

    fn return_from_subroutine(chip: &mut Chip) {
        if let Ok(addr) = chip.stack.pop() {
            chip.program_counter = addr;
        } else {
            eprintln!("Error popping value off the stack. Exiting...");
            std::process::exit(1);
        }
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
    fn clear_screen_buffer() {
        let mut chip = Chip::new();
        chip.screen_buffer = vec![true; chip.screen_buffer.len()];
        Opcode::clear_screen(&mut chip);

        assert_eq!(chip.screen_buffer, vec![false; chip.screen_buffer.len()]);
    }

    #[test]
    fn function_return() {
        let mut chip = Chip::new();
        chip.stack.push(0x123).unwrap();
        Opcode::return_from_subroutine(&mut chip);

        assert_eq!(0x123, chip.program_counter);
    }
}
