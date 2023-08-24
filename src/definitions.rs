const RAM: usize = 4096;
const REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;

// CHIP-8 programs load from 0x200 in memory onwards, as everything before was reserved for the interpreter.
pub const START_ADDR: u16 = 0x200;

pub const DISP_ROWS: usize = 32;
pub const DISP_COLS: usize = 64;

pub struct cpu {
    
    // memory
    pub mem: [u8; RAM],

    // RAM RW index register
    pub i: u16,

    // program counter register
    pub pc: u16,

    // V / general purpose registers [V0-VF]
    pub v: [u8; REGS],

    // display
    pub display: [bool; DISP_COLS * DISP_ROWS],

    // CPU stack
    pub sp: u16, // stack pointer, used as index
    pub stack: [u16; STACK_SIZE],

    // keypad - we use an array of booleans to store only if the key is pressed (1) or not (0)
    pub keypad: [bool; NUM_KEYS],

    // timers
    // * delay timer: general purpose timer, counts down every cycle
    pub dt: u8,
    // * sound timer: same as DT, but emits a sound when it reaches 0
    pub st: u8 

}