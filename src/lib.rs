#![allow(dead_code)]
use std::vec::Vec;
mod stack;

const SCREEN_WIDTH: u8 = 64;
const SCREEN_HEIGHT: u8 = 32;

struct Chip {
    screen_buffer: Vec<bool>,
    registers: Vec<u8>,
    address: u16,
    stack: stack::Stack<u8>,
}

#[cfg(test)]
mod tests {}
