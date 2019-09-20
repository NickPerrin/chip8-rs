use std::vec::Vec;
pub mod stack;

pub struct Chip {
    memory: Vec<u8>,
    stack: stack::Stack<u16>,
    registers: Vec<u8>,
    address: u16,
    program_counter: u16,
    screen_buffer: Vec<bool>,
    screen_width: u8,
    screen_height: u8,
}

#[cfg(test)]
mod tests {}
