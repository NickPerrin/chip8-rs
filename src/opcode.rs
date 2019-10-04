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
                0x0 => self.clear_screen(&mut chip),
                0xE => self.return_from_subroutine(&mut chip),
                _ => panic!("Illegal opcode! {}", self.opcode),
            },
            0x1 => self.jump_unconditional(&mut chip, self.opcode & 0x0FFF),
            0x2 => self.call_subroutine(&mut chip, self.opcode & 0x0FFF),
            0x3 => self.skip_if_equal(
                &mut chip,
                usize::from(self.n3()),
                (self.opcode & 0x00FF) as u8,
            ),
            0x4 => self.skip_if_not_equal(
                &mut chip,
                usize::from(self.n3()),
                (self.opcode & 0x00FF) as u8,
            ),
            0x5 => self.skip_equal_registers(
                &mut chip, 
                usize::from(self.n3()), 
                usize::from(self.n4())
            ),
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
                    _ => (panic!("Illegal opcode! {}", self.opcode)),
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
                    _ => (panic!("Illegal opcode! {}", self.opcode)),
                }
            }
            0xF => {
                match self.n3() {
                    0x0 => {
                        match self.n4() {
                            0x7 => (), // set vx to the delay timer
                            0xA => (), // store key press into vx
                            _ => (panic!("Illegal opcode! {}", self.opcode)),
                        }
                    }
                    0x1 => {
                        match self.n4() {
                            0x5 => (), // set delay timer
                            0x8 => (), // set sound timer
                            0xE => (), // add vx to address register
                            _ => (panic!("Illegal opcode! {}", self.opcode)),
                        }
                    }
                    0x2 => (), // set address register to sprite_addr[vx]
                    0x3 => (), // write binary coded decimal to address register
                    0x5 => (), // dump registers into memory
                    0x6 => (), // load registers from memory
                    _ => (panic!("Illegal opcode! {}", self.opcode)),
                }
            }
            _ => (panic!("Illegal opcode! {}", self.opcode)),
        }
    }

    /// Clear the screen buffer
    fn clear_screen(&self, chip: &mut Chip) {
        chip.screen_buffer.clear();
        chip.increment_program_counter(None);
    }

    /// Function return
    fn return_from_subroutine(&self, chip: &mut Chip) {
        if let Ok(addr) = chip.stack.pop() {
            chip.program_counter = addr;
            chip.increment_program_counter(None);
        } else {
            panic!("Error popping value off the stack. Exiting...");
        }
    }

    /// Jump to the given address
    ///
    /// address The address to be jumped to
    fn jump_unconditional(&self, chip: &mut Chip, address: u16) {
        if (address & 0xF000) != 0 {
            panic!("Invalid memory address provided to jump!");
        }
        chip.program_counter = address;
    }

    /// Call a given subroutine
    ///
    /// address The address of the subroutine
    fn call_subroutine(&self, chip: &mut Chip, address: u16) {
        if (address & 0xF000) != 0 {
            panic!("Invalid memory address provided to call_subroutine!");
        }
        if let Ok(_) = chip.stack.push(chip.program_counter) {
            chip.program_counter = address;
        } else {
            panic!("The call stack has run out of space!");
        }
    }

    /// Skip the next instruction if the value matches the given register
    ///
    /// register The register to be compared. V0, V1,...
    /// value The constant value to compare
    fn skip_if_equal(&self, chip: &mut Chip, register: usize, value: u8) {
        if register >= chip.registers.len() {
            panic!("Invalid register provided");
        } else if chip.registers[register] == value {
            chip.increment_program_counter(Some(2));
        } else {
            chip.increment_program_counter(None);
        }
    }

    /// Skip the next instruction if the value does not match the given register
    ///
    /// register The register to be compared. V0, V1,...
    /// value The constant value to compare
    fn skip_if_not_equal(&self, chip: &mut Chip, register: usize, value: u8) {
        if register >= chip.registers.len() {
            panic!("Invalid register provided");
        } else if chip.registers[register] != value {
            chip.increment_program_counter(Some(2));
        } else {
            chip.increment_program_counter(None);
        }
    }

    /// Skip the next instruction if Vx == Vy
    fn skip_equal_registers(&self, chip: &mut Chip, vx: usize, vy: usize) {
        if (vx >= chip.registers.len()) || (vy >= chip.registers.len()) {
            panic!("Invalid register provided");
        } else if chip.registers[vx] == chip.registers[vy] {
            chip.increment_program_counter(Some(2));
        } else {
            chip.increment_program_counter(None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skip_equal_registers_true() {
        let (mut chip, opcode) = chip_opcode();
        chip.program_counter = 0x200;
        chip.registers[0xF] = 6;
        chip.registers[0xE] = 6;
        opcode.skip_equal_registers(&mut chip, 0xF, 0xE);
        assert_eq!(0x204, chip.program_counter);
    }

    #[test]
    fn skip_equal_registers_false() {
        let (mut chip, opcode) = chip_opcode();
        chip.program_counter = 0x200;
        chip.registers[0xF] = 6;
        chip.registers[0xE] = 7;
        opcode.skip_equal_registers(&mut chip, 0xF, 0xE);
        assert_eq!(0x202, chip.program_counter);
    }

    #[test]
    #[should_panic]
    fn skip_equal_registers_invalid_vx() {
        let (mut chip, opcode) = chip_opcode();
        opcode.skip_equal_registers(&mut chip, 0x10, 0);
    }

    #[test]
    #[should_panic]
    fn skip_equal_registers_invalid_vy() {
        let (mut chip, opcode) = chip_opcode();
        opcode.skip_equal_registers(&mut chip, 0x1, 0x10);
    }

    #[test]
    fn skip_if_not_equal_true() {
        let (mut chip, opcode) = chip_opcode();
        chip.program_counter = 0x200;
        chip.registers[0xF] = 6;
        opcode.skip_if_not_equal(&mut chip, 0xF, 7);
        assert_eq!(0x204, chip.program_counter);
    }

    #[test]
    fn skip_if_not_equal_false() {
        let (mut chip, opcode) = chip_opcode();
        chip.program_counter = 0x200;
        chip.registers[0xF] = 6;
        opcode.skip_if_not_equal(&mut chip, 0xF, 6);
        assert_eq!(0x202, chip.program_counter);
    }

    #[test]
    #[should_panic]
    fn skip_if_not_equal_invalid() {
        let (mut chip, opcode) = chip_opcode();
        opcode.skip_if_equal(&mut chip, 0x10, 0);
    }

    #[test]
    fn skip_if_equal_true() {
        let (mut chip, opcode) = chip_opcode();
        chip.program_counter = 0x200;
        chip.registers[0xF] = 6;
        opcode.skip_if_equal(&mut chip, 0xF, 6);
        assert_eq!(0x204, chip.program_counter);
    }

    #[test]
    fn skip_if_equal_false() {
        let (mut chip, opcode) = chip_opcode();
        chip.program_counter = 0x200;
        chip.registers[0xF] = 6;
        opcode.skip_if_equal(&mut chip, 0xF, 5);
        assert_eq!(0x202, chip.program_counter);
    }

    #[test]
    #[should_panic]
    fn skip_if_equal_invalid() {
        let (mut chip, opcode) = chip_opcode();
        opcode.skip_if_equal(&mut chip, 0x10, 0);
    }

    #[test]
    fn clear_screen_buffer() {
        let (mut chip, opcode) = chip_opcode();
        chip.screen_buffer = vec![true; chip.screen_buffer.len()];
        opcode.clear_screen(&mut chip);
        assert_eq!(chip.screen_buffer, vec![false; chip.screen_buffer.len()]);
        assert_eq!(0x202, chip.program_counter);
    }

    #[test]
    fn function_return() {
        let (mut chip, opcode) = chip_opcode();
        chip.stack.push(0x123).unwrap();
        opcode.return_from_subroutine(&mut chip);
        assert_eq!(0x123 + 2, chip.program_counter);
    }

    #[test]
    #[should_panic]
    fn return_empty_stack() {
        let (mut chip, opcode) = chip_opcode();
        opcode.return_from_subroutine(&mut chip);
    }

    #[test]
    fn jump_unconditional() {
        let (mut chip, opcode) = chip_opcode();
        opcode.jump_unconditional(&mut chip, 0xFFF);
        assert_eq!(0xFFF, chip.program_counter);
    }

    #[test]
    #[should_panic]
    fn jump_invalid() {
        let (mut chip, opcode) = chip_opcode();
        opcode.jump_unconditional(&mut chip, 0xFFFF);
    }

    #[test]
    fn call_subroutine() {
        let (mut chip, opcode) = chip_opcode();
        opcode.call_subroutine(&mut chip, 0x2);
        assert_eq!(0x2, chip.program_counter);
    }

    #[test]
    #[should_panic]
    fn call_invalid() {
        let (mut chip, opcode) = chip_opcode();
        opcode.call_subroutine(&mut chip, 0xFFFF);
    }

    #[test]
    #[should_panic]
    fn call_full_stack() {
        let (mut chip, opcode) = chip_opcode();
        chip.stack.size = 0;
        chip.stack.head = 0;
        opcode.call_subroutine(&mut chip, 0xFFF);
    }

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

    fn chip_opcode() -> (Chip, Opcode) {
        (Chip::new(), Opcode::new(0))
    }
}
