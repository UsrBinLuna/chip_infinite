use std::io::Write;
use colored::Colorize;
mod emu;

impl Warn {

    fn underflow() {
        println!(
            "{}, {}, Did not pop at {} due to SP = 0.", 
            "ERR: ".red(),
            "E001: ".yellow(),
            
            emu::self.sp;
        );
    }

}