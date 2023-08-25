use std::string;
use colored::Colorize;

use crate::definitions::Cpu;

mod definitions;
mod font;


// Implement a new constructor for our Cpu struct - PC is 0x200. We use &mut self to 
// access and modify all values under self
impl Cpu {

    pub fn new(&mut self) -> Self {

        // Initialize all variables
        let mut new_emu: Cpu = Self {
            pc: definitions::START_ADDR,
            ram: [0; definitions::RAM], // sets all RAM values to 0
            display: [false; definitions::DISP_COLS * definitions::DISP_ROWS],
            v_reg: [0; definitions::REGS], // creates REGS number of [V0-VF] and sets them to 0
            i_reg: 0,
            stack: [0; definitions::STACK_SIZE],
            keypad: [false; definitions::NUM_KEYS],
            dt: 0,
            st: 0,
            sp: 0
        };

        // Load fonts to memory, FONTSET_SIZE being the addresses.
        new_emu.ram[..font::FONTSET_SIZE].copy_from_slice(&font::FONTSET);

        // Initialize the emulator and return
        new_emu

    }

    // Stack functions
    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val; // val being the value we want to push to the stack, and sp is the index where to.
        self.sp += 1; // increase stack pointer by one, so next object is one "level" higher.
    }

    fn pop(&mut self) -> u16 {
        if self.sp > 0 { // underflow protection
            self.sp -= 1; // decrease stack pointer by one, so next object is one "level" lower.
            self.stack[self.sp as usize] // reads value but is left alone for next push to override. no semicolon = `return <var>`
        } else {
            underflow_panic(self.sp);
            1
            // return nothing and don't do anything, as there is no slot lower than 0.
        }
    }

    // RESET function - resets all values to their default, and reloads fonts to memory.
    pub fn reset(&mut self) {
        self.pc = definitions::START_ADDR;
        self.ram = [0; definitions::RAM];
        self.display = [false; definitions::DISP_COLS * definitions::DISP_ROWS];
        self.v_reg = [0; definitions::REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; definitions::STACK_SIZE];
        self.keypad = [false; definitions::NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..font::FONTSET_SIZE].copy_from_slice(&font::FONTSET);
    }

    // CPU loop - runs every CYCLE
    pub fn tick(&mut self) {
        // fetch
        let op = self.fetch();

        // decode
        // execute
    }

    // Fetch opcode from RAM address at Program Counter
    // Luckily CHIP-8 encodes all args into the OPCODE, so the syntax is the same.
    fn fetch(&mut self) -> u16 {

        // Get value from current 2 bytes of memory, as all instructions are 2B.
        let high_byte = self.ram[self.pc as usize] as u16;
        let low_byte = self.ram[(self.pc + 1) as usize] as u16;

        // Combine both values as Big Endian - all RAM values are 8 bit
        let op = (high_byte << 8) | low_byte;
        // Move to next instruction
        self.pc += 2;
        // Return OPCODE
        op

    }

    fn exec(&mut self, op: u16) {
        // Separate the byte into 4 HEX digits
        let d1 = (op & 0xF000) >> 12;
        let d2 = (op & 0x0F00) >> 8;
        let d3 = (op & 0x00F0) >> 4;
        let d4 = op & 0x000F;

        // Match opcodes
        match (d1, d2, d3, d4) {
            // NOP - 0x000 "Do nothing"
            (0, 0, 0, 0) => return,

            // CLS - 0x00E0 "Clear screen" (reset buffer)
            (0, 0, 0xE, 0) => {
                self.display = [false; definitions::DISP_COLS * definitions::DISP_ROWS];
            },

            // RET - 0x00EE "Return from subroutine (function)" - move PC to specified address, then return to original addr.
            // Reads the address from the CPU stack and moves PC to it.
            (0, 0, 0xE, 0xE) => {
                let ret_addr: u16 = self.pop();
                self.pc = ret_addr;
            },

            // JMP NNN - 0x1NNN - jump to specified address (specified by the 3 Ns)
            (1, _, _, _) => {
                let addr = op & 0xFFF;
                self.pc = addr;
            },

            // CALL NNN - 0x2NNN - add current PC to stack, then jump to (NN)
            (2, _, _ ,_) => {
                let addr = op & 0xFFF;
                self.push(self.pc);
                self.pc = addr;
            },

            // SKIP VX = NN - 0x3XNN - if V reg = arg, make true and continue to next instruction
            (3, _, _, _) => {
                let x = d2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            },

            // SKIP VX != NN - 0x4XNN - if V reg != arg, make true and continue to next instruction
            (4, _, _, _) => {
                let x = d2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            },

            // SKIP VX = VY - 0x5XY0 - if VX = VY, make true and continue to next instruction
            (5, _, _, 0) => {
                let x = d2 as usize;
                let y = d3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            },

            // VX = NN - 0x6XNN - set VX to value given by NN.
            (6, _, _, _) => {
                let x = d2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = nn;
            },

            // VX = NN - 0x7XNN - add value NN to VX. Rust will panic if overflow, so normal addition is not usable.
            (7, _, _, _) => {
                let x = d2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            },

            // VX = VY - 0x8XY0
            (8, _, _, 0) => {
                let x = d2 as usize;
                let y = d3 as usize;
                self.v_reg[x] = self.v_reg[y];
            },

            // 0x8XY1, 0x8XY2, 0x8XY3 - Bitwise operations
            // VX |= VY - 0x8XY1 - Binary OR (represented by | symbol)
            (8, _, _, 1) => {
                let x = d2 as usize;
                let y = d3 as usize;
                self.v_reg[x] |= self.v_reg[y];
            },

            // VX &= VY - 0x8XY2 - Binary AND (represented by & symbol)
            (8, _, _, 2) => {
                let x = d2 as usize;
                let y = d3 as usize;
                self.v_reg[x] &= self.v_reg[y];
            },

            // VX ^= VY - 0x8XY3 - Binary XOR (represented by ^ symbol)
            (8, _, _, 3) => {
                let x = d2 as usize;
                let y = d3 as usize;
                self.v_reg[x] ^= self.v_reg[y];
            },

            // "_" stands for "everything else"
            (_, _, _, _) => no_instr(false, op),

        }

    }

    // Timer Tick - modifies ST and DT every FRAME, so needs its own function
    pub fn timer_tick(&mut self) {

        // Lower by 1 every frame, making it work like a timer.
        // Once it reaches 0, it won't be reset until the game needs it again.
        if (self.dt > 0) {
            self.dt -= 1;
        }

        if (self.st > 0) {
            if (self.st == 1) {
                no_audio();
            } 
        }
        self.st -= 1;

    }

}


// WARNINGS
pub fn underflow_panic(sp: u16) {
    println!(
        "{} {} Did not pop at {} due to SP = 0.", 
        "WARN:".red(),
        "W001:".yellow(),
        sp
    );
}

pub fn no_instr(no_panic: bool, op: u16) {
    println!(
        "{} {} Panic! Unimplemented opcode {}",
        "ERR:".red(),
        "E001".yellow(),
        op
    );
    if !no_panic {
        unimplemented!();
    } 
}

pub fn no_audio() {
    println!(
        "{} {} Audio not implemented!",
        "WARN:".red(),
        "W002:".yellow()
    )
}

fn warn_tests() {
    println!("*** RUNNING WARNING TESTS ***");
    underflow_panic(0);
    no_audio();
    no_instr(true, 0000);   
}

// MAIN
fn main(){
    println!("*** Welcome to Chip-Infinite! ***");
    
    warn_tests();
}

