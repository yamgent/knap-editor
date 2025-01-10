use std::io::{self, Read};

use crossterm::terminal;

fn main() {
    terminal::enable_raw_mode().expect("able to enable raw mode");
    io::stdin().bytes().any(|byte| {
        let ch = byte.expect("ascii character") as char;
        println!("{}", ch);

        let quit = ch == 'q';

        if quit {
            terminal::disable_raw_mode().expect("able to disable raw mode");
        }

        quit
    });
}
