use super::Chip;
use std::io::{Error, ErrorKind};

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

    fn constant(&self) -> u8 {
        (self.opcode & 0xFF) as u8
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
                self.constant(),
            ),
            0x4 => self.skip_if_not_equal(
                &mut chip,
                usize::from(self.n3()),
                self.constant(),
            ),
            0x5 => self.skip_equal_registers(
                &mut chip,
                usize::from(self.n3()),
                usize::from(self.n4()),
            ),
            0x6 => self.load_constant(
                &mut chip,
                usize::from(self.n3()),
                self.constant(),
            ),
            0x7 => self.add_constant(
                &mut chip,
                usize::from(self.n3()),
                self.constant(),
            ),
            0x8 => match self.n4() {
                0x0 => self.set_vx_from_vy(
                    &mut chip,
                    usize::from(self.n3()),
                    usize::from(self.n2()),
                ),
                0x1 => self.vx_or_vy(
                    &mut chip,
                    usize::from(self.n3()),
                    usize::from(self.n2()),
                ),
                0x2 => self.vx_and_vy(
                    &mut chip,
                    usize::from(self.n3()),
                    usize::from(self.n2()),
                ),
                0x3 => self.vx_xor_vy(
                    &mut chip,
                    usize::from(self.n3()),
                    usize::from(self.n2()),
                ),
                0x4 => self.add_vx_vy(
                    &mut chip,
                    usize::from(self.n3()),
                    usize::from(self.n2()),
                ),
                0x5 => self.subtract_vx_vy(
                    &mut chip,
                    usize::from(self.n3()),
                    usize::from(self.n2()),
                ),
                0x6 => self.shift_right_vx(&mut chip, usize::from(self.n3())),
                0x7 => self.subtract_vy_vx(
                    &mut chip,
                    usize::from(self.n3()),
                    usize::from(self.n2()),
                ),
                0xE => self.shift_left_vx(&mut chip, usize::from(self.n3())),
                _ => (panic!("Illegal opcode! {}", self.opcode)),
            },
            0x9 => self.skip_vx_not_equal_vy(
                &mut chip,
                usize::from(self.n3()),
                usize::from(self.n2()),
            ),
            0xA => self.set_address_register(&mut chip, self.opcode),
            0xB => self.jump_addr_v0(&mut chip, self.opcode),
            0xC => self.set_vx_rand(
                &mut chip,
                usize::from(self.n3()),
                self.constant(),
            ),
            0xD => self.draw_sprite(
                &mut chip,
                usize::from(self.n3()),
                usize::from(self.n3()),
                (self.n1() & 0xFF) as u8,
            ),
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

    fn valid_registers(registers: &[usize], chip: &Chip) -> Result<(), Error> {
        if registers.len() == 0 {
            return Err(Error::from(ErrorKind::Other));
        }

        let result: Result<(), Error> = Err(Error::from(ErrorKind::Other));
        let max = chip.registers.len();
        for index in registers {
            if *index >= max {
                return result;
            }
        }
        Ok(())
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
    fn jump_unconditional(&self, chip: &mut Chip, address: u16) {
        if (address & 0xF000) != 0 {
            panic!("Invalid memory address provided to jump!");
        }
        chip.program_counter = address;
    }

    /// Call a given subroutine
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
    fn skip_if_equal(&self, chip: &mut Chip, vx: usize, value: u8) {
        if vx >= chip.registers.len() {
            panic!("Invalid register provided");
        } else if chip.registers[vx] == value {
            chip.increment_program_counter(Some(2));
        } else {
            chip.increment_program_counter(None);
        }
    }

    /// Skip the next instruction if the value does not match the given register
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

    /// Set vx to a constant value
    fn load_constant(&self, chip: &mut Chip, vx: usize, value: u8) {
        if vx >= chip.registers.len() {
            panic!("Invalid register provided");
        }
        chip.registers[vx] = value;
        chip.increment_program_counter(None);
    }

    /// add a constant to vx
    ///
    /// note Carry flag is not changed
    fn add_constant(&self, chip: &mut Chip, vx: usize, value: u8) {
        Opcode::valid_registers(&[vx], &chip)
            .expect("Invalid register in add_constant");
        chip.registers[vx] += chip.registers[vx].wrapping_add(value);
        chip.increment_program_counter(None);
    }

    /// Set the value of vx to vy
    fn set_vx_from_vy(&self, chip: &mut Chip, vx: usize, vy: usize) {
        Opcode::valid_registers(&[vx, vy], &chip)
            .expect("Invalid register in set_vx_from_vy");
        chip.registers[vx] = chip.registers[vy];
        chip.increment_program_counter(None);
    }

    /// Sets vx = vx | vy
    fn vx_or_vy(&self, chip: &mut Chip, vx: usize, vy: usize) {
        Opcode::valid_registers(&[vx, vy], &chip)
            .expect("Invalid register in vx_or_vy");
        chip.registers[vx] = chip.registers[vx] | chip.registers[vy];
        chip.increment_program_counter(None);
    }

    /// Sets vx = vx & vy
    fn vx_and_vy(&self, chip: &mut Chip, vx: usize, vy: usize) {
        Opcode::valid_registers(&[vx, vy], &chip)
            .expect("Invalid register in vx_and_vy");
        chip.registers[vx] = chip.registers[vx] & chip.registers[vy];
        chip.increment_program_counter(None);
    }

    /// Sets vx = vx ^ vy
    fn vx_xor_vy(&self, chip: &mut Chip, vx: usize, vy: usize) {
        Opcode::valid_registers(&[vx, vy], &chip)
            .expect("Invalid register in vx_xor_vy");
        chip.registers[vx] = chip.registers[vx] ^ chip.registers[vy];
        chip.increment_program_counter(None);
    }

    /// vx = vx + vy
    /// vf is set to 1 if overflow occurs
    fn add_vx_vy(&self, chip: &mut Chip, vx: usize, vy: usize) {
        Opcode::valid_registers(&[vx, vy], &chip)
            .expect("Invalid register in add_vx_vy");

        let (x, overflow) =
            chip.registers[vx].overflowing_add(chip.registers[vy]);
        chip.registers[vx] = x;
        chip.registers[0xF] = u8::from(overflow);
        chip.increment_program_counter(None);
    }

    /// Subtract vy from vx
    fn subtract_vx_vy(&self, chip: &mut Chip, vx: usize, vy: usize) {
        Opcode::valid_registers(&[vx, vy], &chip)
            .expect("Invalid register in subtract_vx_vy");

        let (x, overflow) =
            chip.registers[vx].overflowing_sub(chip.registers[vy]);
        chip.registers[vx] = x;
        chip.registers[0xF] = u8::from(!overflow);
        chip.increment_program_counter(None);
    }

    /// shift vx once to the right. Store lsb in vf
    fn shift_right_vx(&self, chip: &mut Chip, vx: usize) {
        Opcode::valid_registers(&[vx], &chip)
            .expect("Invalid register in shift_right_vx");

        chip.registers[0xF] = chip.registers[vx] & 0x1;
        chip.registers[vx] = chip.registers[vx] >> 1;
    }

    /// Subtract vx from vy
    fn subtract_vy_vx(&self, chip: &mut Chip, vx: usize, vy: usize) {
        Opcode::valid_registers(&[vx, vy], &chip)
            .expect("Invalid register in subtract_vx_vy");

        let (x, overflow) =
            chip.registers[vy].overflowing_sub(chip.registers[vx]);
        chip.registers[vx] = x;
        chip.registers[0xF] = u8::from(!overflow);
        chip.increment_program_counter(None);
    }

    /// Left shift vx, store ms_bit in vf
    fn shift_left_vx(&self, chip: &mut Chip, vx: usize) {
        Opcode::valid_registers(&[vx], &chip)
            .expect("Invalid register in shift_right_vx");

        chip.registers[0xF] = ((chip.registers[vx] & 0x80) >> 7) & 0x1;
        chip.registers[vx] = chip.registers[vx] << 1;
    }

    /// Skip next instruction if vx != vy
    fn skip_vx_not_equal_vy(&self, chip: &mut Chip, vx: usize, vy: usize) {
        Opcode::valid_registers(&[vx, vy], &chip)
            .expect("Invalid register in shift_right_vx");

        if chip.registers[vx] != chip.registers[vy] {
            chip.increment_program_counter(Some(2));
        } else {
            chip.increment_program_counter(None);
        }
    }

    /// Set the address register I
    fn set_address_register(&self, chip: &mut Chip, addr: u16) {
        chip.address = addr & 0x0FFF;
        chip.increment_program_counter(None);
    }

    /// Jump to address + v0
    fn jump_addr_v0(&self, chip: &mut Chip, addr: u16) {
        let mask_addr = addr & 0x0FFF;
        chip.program_counter += mask_addr + u16::from(chip.registers[0]);
    }

    /// Set vx to a random value (0..255)
    fn set_vx_rand(&self, chip: &mut Chip, vx: usize, constant: u8) {
        Opcode::valid_registers(&[vx], &chip)
            .expect("Invalid register in shift_right_vx");
        let random_byte = rand::random::<u8>();
        chip.registers[vx] = random_byte & constant;
        chip.increment_program_counter(None);
    }

    /// Determine if a pixel has switched from 1 to 0
    fn pixel_collision(b1: u8, b2: u8) -> bool {
        for shift in 0..0x8 {
            let mask = 1 << shift;

            if b1 & mask == 0 {
                continue;
            } else {
                if b2 & mask != 0 {
                    return true;
                }
            }
        }
        false
    }

    /// Draw a sprite to the screen
    fn draw_sprite(&self, chip: &mut Chip, vx: usize, vy: usize, height: u8) {
        Opcode::valid_registers(&[vx, vy], &chip)
            .expect("Invalid register in shift_right_vx");

        chip.registers[0xF] = 0;

        for row in 0..height {
            let chunk_index: u8 = ((chip.registers[vy] + row)
                * chip.screen_width)
                + chip.registers[vx];
            let chunk: u8 = chip.screen_buffer[usize::from(chunk_index)];

            // @todo remove
            println!("row: {}", row);
            println!("chunk: {}", chunk);

            let new = chip.memory[usize::from(chip.address) + usize::from(row)];

            // @todo remove
            println!("new: {}", new);

            chip.screen_buffer[usize::from(chunk_index)] = chunk ^ new;
            if Opcode::pixel_collision(chunk, new) {
                chip.registers[0xF] = 1;
                break;
            }
        }
        chip.increment_program_counter(None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_sprite_trivial() {
        let (mut chip, opcode) = chip_opcode();
        chip.program_counter = 0x200;
        chip.registers[0] = 0;
        chip.registers[1] = 0;

        chip.address = 0;
        chip.memory[usize::from(chip.address)] = 0xFF;
        chip.screen_buffer[usize::from(chip.address)] = 0;

        opcode.draw_sprite(&mut chip, 0, 1, 1);
        assert_eq!(0, chip.registers[0xF]);
        assert_eq!(0x202, chip.program_counter);
        assert_eq!(0xFF, chip.screen_buffer[usize::from(chip.address)]);
    }

    #[test]
    fn draw_sprite_flip_bit() {
        let (mut chip, opcode) = chip_opcode();
        chip.program_counter = 0x200;
        chip.registers[0] = 0;
        chip.registers[1] = 0;

        chip.address = 0;
        chip.memory[usize::from(chip.address)] = 0xA5;
        chip.screen_buffer[usize::from(chip.address)] = 0xA5;

        opcode.draw_sprite(&mut chip, 0, 1, 1);
        assert_eq!(1, chip.registers[0xF]);
        assert_eq!(0x202, chip.program_counter);
        assert_eq!(0, chip.screen_buffer[usize::from(chip.address)]);
    }

    #[test]
    fn draw_sprite_multi_line() {
        let (mut chip, opcode) = chip_opcode();
        chip.program_counter = 0x200;
        chip.registers[0] = 0;
        chip.registers[1] = 0;

        chip.address = 0;
        chip.memory[usize::from(chip.address)] = 0x00;
        chip.memory[usize::from(chip.address) + 1] = 0x01;
        chip.screen_buffer[usize::from(chip.address)] = 0xA5;
        chip.screen_buffer
            [usize::from(chip.address) + usize::from(chip.screen_width)] = 0x01;

        opcode.draw_sprite(&mut chip, 0, 1, 2);
        assert_eq!(1, chip.registers[0xF]);
        assert_eq!(0x202, chip.program_counter);
        assert_eq!(0xA5, chip.screen_buffer[usize::from(chip.address)]);
        assert_eq!(0x00, chip.screen_buffer[usize::from(chip.address) + 1]);
    }

    #[test]
    fn pixel_collision() {
        let mut b1 = 0;
        let mut b2 = 0;
        assert!(!Opcode::pixel_collision(b1, b2));

        b1 = 1;
        b2 = 1;
        assert!(Opcode::pixel_collision(b1, b2));

        b1 = 0x10;
        b2 = 0x1F;
        assert!(Opcode::pixel_collision(b1, b2));

        b1 = 0x80;
        b2 = 0x80;
        assert!(Opcode::pixel_collision(b1, b2));
    }

    #[test]
    fn jump_v0() {
        let (mut chip, opcode) = chip_opcode();
        chip.program_counter = 0x200;
        chip.registers[0] = 0x34;
        opcode.jump_addr_v0(&mut chip, 0x100);
        assert_eq!(0x334, chip.program_counter);
    }

    #[test]
    fn set_addr() {
        let (mut chip, opcode) = chip_opcode();
        chip.program_counter = 0x200;

        opcode.set_address_register(&mut chip, 0x1234);
        assert_eq!(0x234, chip.address);
        assert_eq!(0x202, chip.program_counter);
    }

    #[test]
    fn set_addr_too_big() {
        let (mut chip, opcode) = chip_opcode();
        chip.program_counter = 0x200;

        opcode.set_address_register(&mut chip, 0x123);
        assert_eq!(0x123, chip.address);
        assert_eq!(0x202, chip.program_counter);
    }

    #[test]
    fn skip_vx_ne_vy() {
        let (mut chip, opcode) = chip_opcode();
        chip.program_counter = 0x200;
        chip.registers[0] = 0;
        chip.registers[1] = 1;
        opcode.skip_vx_not_equal_vy(&mut chip, 0, 1);
        assert_eq!(0x204, chip.program_counter);
    }

    #[test]
    fn skip_vx_eq_vy() {
        let (mut chip, opcode) = chip_opcode();
        chip.program_counter = 0x200;
        chip.registers[0] = 4;
        chip.registers[1] = 4;
        opcode.skip_vx_not_equal_vy(&mut chip, 0, 1);
        assert_eq!(0x202, chip.program_counter);
    }

    #[test]
    fn shift_left_vx_0() {
        let (mut chip, opcode) = chip_opcode();
        chip.program_counter = 0x200;
        chip.registers[0] = 0x7F;
        opcode.shift_left_vx(&mut chip, 0);
        assert_eq!(0xFE, chip.registers[0]);
        assert_eq!(0, chip.registers[0xF]);
    }

    #[test]
    fn shift_left_vx_1() {
        let (mut chip, opcode) = chip_opcode();
        chip.program_counter = 0x200;
        chip.registers[0] = 0xFF;
        opcode.shift_left_vx(&mut chip, 0);
        assert_eq!(0xFE, chip.registers[0]);
        assert_eq!(1, chip.registers[0xF]);
    }

    #[test]
    #[should_panic]
    fn shift_left_vx_invalid() {
        let (mut chip, opcode) = chip_opcode();
        opcode.shift_left_vx(&mut chip, 0x10);
    }

    #[test]
    fn shift_right_vx_0() {
        let (mut chip, opcode) = chip_opcode();
        chip.program_counter = 0x200;
        chip.registers[0] = 0x2;
        opcode.shift_right_vx(&mut chip, 0);
        assert_eq!(1, chip.registers[0]);
        assert_eq!(0, chip.registers[0xF]);
    }

    #[test]
    fn shift_right_vx_1() {
        let (mut chip, opcode) = chip_opcode();
        chip.program_counter = 0x200;
        chip.registers[0] = 0x3;
        opcode.shift_right_vx(&mut chip, 0);
        assert_eq!(1, chip.registers[0]);
        assert_eq!(1, chip.registers[0xF]);
    }

    #[test]
    #[should_panic]
    fn shift_right_vx_invalid() {
        let (mut chip, opcode) = chip_opcode();
        opcode.shift_right_vx(&mut chip, 0x10);
    }

    #[test]
    fn subtract_vx_greater_than_vy() {
        let (mut chip, opcode) = chip_opcode();
        chip.program_counter = 0x200;
        chip.registers[0] = 2;
        chip.registers[1] = 1;
        opcode.subtract_vx_vy(&mut chip, 0, 1);
        assert_eq!(1, chip.registers[0]);
        assert_eq!(1, chip.registers[0xF]);
        assert_eq!(0x202, chip.program_counter);
    }

    #[test]
    fn subtract_vx_less_than_vy() {
        let (mut chip, opcode) = chip_opcode();
        chip.program_counter = 0x200;
        chip.registers[0] = 0;
        chip.registers[1] = 1;
        opcode.subtract_vx_vy(&mut chip, 0, 1);
        assert_eq!(255, chip.registers[0]);
        assert_eq!(0, chip.registers[0xF]);
        assert_eq!(0x202, chip.program_counter);
    }

    #[test]
    #[should_panic]
    fn subtract_vx_vy_invalid_vx() {
        let (mut chip, opcode) = chip_opcode();
        opcode.subtract_vx_vy(&mut chip, 0x10, 0x0);
    }

    #[test]
    #[should_panic]
    fn subtract_vx_vy_invalid_vy() {
        let (mut chip, opcode) = chip_opcode();
        opcode.subtract_vx_vy(&mut chip, 0x0, 0x10);
    }

    #[test]
    fn add_vx_vy_no_overflow() {
        let (mut chip, opcode) = chip_opcode();
        chip.program_counter = 0x200;
        chip.registers[0] = 0;
        chip.registers[1] = 1;
        opcode.add_vx_vy(&mut chip, 0, 1);
        assert_eq!(1, chip.registers[0]);
        assert_eq!(0, chip.registers[0xF]);
        assert_eq!(0x202, chip.program_counter);
    }
    #[test]
    fn add_vx_vy_overflow() {
        let (mut chip, opcode) = chip_opcode();
        chip.program_counter = 0x200;
        chip.registers[0] = 255;
        chip.registers[1] = 1;
        opcode.add_vx_vy(&mut chip, 0, 1);
        assert_eq!(0, chip.registers[0]);
        assert_eq!(1, chip.registers[0xF]);
        assert_eq!(0x202, chip.program_counter);
    }

    #[test]
    #[should_panic]
    fn add_vx_vy_invalid_vx() {
        let (mut chip, opcode) = chip_opcode();
        opcode.add_vx_vy(&mut chip, 0x10, 0x0);
    }

    #[test]
    #[should_panic]
    fn add_vx_vy_invalid_vy() {
        let (mut chip, opcode) = chip_opcode();
        opcode.add_vx_vy(&mut chip, 0x0, 0x10);
    }

    #[test]
    fn vx_xor_vy() {
        let (mut chip, opcode) = chip_opcode();
        chip.program_counter = 0x200;
        chip.registers[0] = 0;
        chip.registers[1] = 1;
        opcode.vx_xor_vy(&mut chip, 0, 1);
        assert_eq!(1, chip.registers[0]);
        assert_eq!(0x202, chip.program_counter);

        chip.registers[0] = 0;
        chip.registers[1] = 0;
        opcode.vx_xor_vy(&mut chip, 0, 1);
        assert_eq!(0, chip.registers[0]);
        assert_eq!(0x204, chip.program_counter);

        chip.registers[0] = 1;
        chip.registers[1] = 1;
        opcode.vx_xor_vy(&mut chip, 0, 1);
        assert_eq!(0, chip.registers[0]);
        assert_eq!(0x206, chip.program_counter);
    }

    #[test]
    #[should_panic]
    fn vx_xor_vy_invalid_vx() {
        let (mut chip, opcode) = chip_opcode();
        opcode.vx_xor_vy(&mut chip, 0x10, 0x0);
    }

    #[test]
    #[should_panic]
    fn vx_xor_vy_invalid_vy() {
        let (mut chip, opcode) = chip_opcode();
        opcode.vx_xor_vy(&mut chip, 0x0, 0x10);
    }

    #[test]
    fn vx_and_equals_vy() {
        let (mut chip, opcode) = chip_opcode();
        chip.program_counter = 0x200;
        chip.registers[0] = 0;
        chip.registers[1] = 1;
        opcode.vx_and_vy(&mut chip, 0, 1);
        assert_eq!(0, chip.registers[0]);
        assert_eq!(0x202, chip.program_counter);
    }

    #[test]
    #[should_panic]
    fn vx_and_equals_vy_invalid_vx() {
        let (mut chip, opcode) = chip_opcode();
        opcode.vx_and_vy(&mut chip, 0x10, 0x0);
    }

    #[test]
    #[should_panic]
    fn vx_and_equals_vy_invalid_vy() {
        let (mut chip, opcode) = chip_opcode();
        opcode.vx_and_vy(&mut chip, 0x0, 0x10);
    }

    #[test]
    fn vx_or_equals_vy() {
        let (mut chip, opcode) = chip_opcode();
        chip.program_counter = 0x200;
        chip.registers[0] = 0;
        chip.registers[1] = 1;
        opcode.vx_or_vy(&mut chip, 0, 1);
        assert_eq!(1, chip.registers[0]);
        assert_eq!(0x202, chip.program_counter);
    }

    #[test]
    #[should_panic]
    fn vx_or_equals_vy_invalid_vx() {
        let (mut chip, opcode) = chip_opcode();
        opcode.vx_or_vy(&mut chip, 0x10, 0x0);
    }

    #[test]
    #[should_panic]
    fn vx_or_equals_vy_invalid_vy() {
        let (mut chip, opcode) = chip_opcode();
        opcode.vx_or_vy(&mut chip, 0x0, 0x10);
    }

    #[test]
    fn set_vx_from_vy() {
        let (mut chip, opcode) = chip_opcode();
        chip.program_counter = 0x200;
        chip.registers[0] = 0;
        chip.registers[1] = 1;
        opcode.set_vx_from_vy(&mut chip, 0, 1);
        assert_eq!(1, chip.registers[0]);
        assert_eq!(0x202, chip.program_counter);
    }

    #[test]
    #[should_panic]
    fn set_vx_invalid_vx() {
        let (mut chip, opcode) = chip_opcode();
        opcode.set_vx_from_vy(&mut chip, 0x10, 0x0);
    }

    #[test]
    #[should_panic]
    fn set_vx_invalid_vy() {
        let (mut chip, opcode) = chip_opcode();
        opcode.set_vx_from_vy(&mut chip, 0x0, 0x10);
    }

    #[test]
    fn add_constant() {
        let (mut chip, opcode) = chip_opcode();
        chip.program_counter = 0x200;
        chip.registers[0] = 0;
        opcode.add_constant(&mut chip, 0, 2);
        assert_eq!(2, chip.registers[0]);
        assert_eq!(0x202, chip.program_counter);
    }

    #[test]
    fn add_constant_overflow() {
        let (mut chip, opcode) = chip_opcode();
        chip.program_counter = 0x200;
        chip.registers[0] = 1;
        opcode.add_constant(&mut chip, 0, 255);
        assert_eq!(1, chip.registers[0]);
        assert_eq!(0x202, chip.program_counter);
    }

    #[test]
    #[should_panic]
    fn add_constant_invalid() {
        let (mut chip, opcode) = chip_opcode();
        opcode.add_constant(&mut chip, 0x10, 0);
    }

    #[test]
    fn load_constant() {
        let (mut chip, opcode) = chip_opcode();
        chip.program_counter = 0x200;
        chip.registers[0] = 1;
        opcode.load_constant(&mut chip, 0, 0);
        assert_eq!(0, chip.registers[0]);
        assert_eq!(0x202, chip.program_counter);
    }

    #[test]
    #[should_panic]
    fn load_constant_invalid() {
        let (mut chip, opcode) = chip_opcode();
        opcode.load_constant(&mut chip, 0x10, 0);
    }

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
        chip.screen_buffer = vec![1; chip.screen_buffer.len()];
        opcode.clear_screen(&mut chip);
        assert_eq!(chip.screen_buffer, vec![0; chip.screen_buffer.len()]);
        assert_eq!(0x202, chip.program_counter);
    }

    #[test]
    fn function_return() {
        let (mut chip, opcode) = chip_opcode();
        chip.stack.push(0x123).unwrap();
        opcode.return_from_subroutine(&mut chip);
        assert_eq!(0x125, chip.program_counter);
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
    fn valid_registers_trivial() {
        let registers = vec![0x0];
        let chip = Chip::new();
        if let Ok(_) = Opcode::valid_registers(&registers, &chip) {
            assert!(true);
        } else {
            assert!(false);
        }

        let registers = vec![0xF];
        if let Ok(_) = Opcode::valid_registers(&registers, &chip) {
            assert!(true);
        } else {
            assert!(false);
        }

        let mut registers: Vec<usize> = Vec::new();
        if let Ok(_) = Opcode::valid_registers(&registers, &chip) {
            assert!(false);
        } else {
            assert!(true);
        }

        for item in 0..0xF {
            registers.push(item);
        }
        if let Ok(_) = Opcode::valid_registers(&registers, &chip) {
            assert!(true);
        } else {
            assert!(false);
        }

        registers.push(0x10);
        if let Ok(_) = Opcode::valid_registers(&registers, &chip) {
            assert!(false);
        } else {
            assert!(true);
        }
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
