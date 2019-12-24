use chip8_rs::Chip;

enum Command {
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
}

trait Debugger {
    fn get_user_input() -> Option<Command>;
}

struct Chip8Debugger {
    chip: &chip8_rs::Chip,
}

impl Debugger for Chip8Debugger {
    fn get_user_input() -> Option<Command> {
        None
    }
}

impl Debugger {
    fn new(chip: &chip8_rs::Chip) -> Chip8Debugger {
        Chip8Debugger { chip };
    }
}
