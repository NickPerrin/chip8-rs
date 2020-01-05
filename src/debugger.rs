use crate::Chip;
use std::io::{self, Write};

#[derive(Debug)]
pub enum Command {
    // Execute a single instruction
    Step,

    // Print a region of memory. memory address, length
    PrintMemory((u8, u16)),

    // Print a register
    PrintRegisters(u8),

    // Print the next 10 instructions from the program counter
    PrintInstructions,

    // Print keys pressed
    PrintKeys,

    // Print the stack
    PrintStack,

    // Print the screen buffer
    PrintScreenBuffer,

    // Print the sound timer
    PrintSoundTimer,

    // Print the delay timer
    PrintDelayTimer,

    // Start automatic execution from the current state. The debug session will end when the
    // program exits.
    Handoff,

    // Exit the program and the debugging session
    Quit,
}

pub trait Debugger {
    fn get_user_input(&self) -> Option<Command> {
        println!("get_user_input() Unimplemented!");
        None
    }
}

pub struct Chip8Debugger<'a> {
    chip: &'a Chip,
}

impl<'a> Debugger for Chip8Debugger<'a> {
    fn get_user_input(&self) -> Option<Command> {
        // fail at the first invalid token, or no token
        // only parse first command of a given line
        print!("> ");
        io::stdout().flush().unwrap();

        let mut user_line = String::new();
        match io::stdin().read_line(&mut user_line) {
            Ok(bytes_read) => println!("valid"),
            Err(error) => println!("error: {}", error),
        }

        None
    }
}

impl<'a> Chip8Debugger<'a> {
    pub fn new(chip: &'a Chip) -> Chip8Debugger<'a> {
        Chip8Debugger { chip }
    }

    pub fn welcome(&self) {
        println!("Welcome to the chip8 debugger");
        println!("use 'help' to show available commands \n");
    }
}
