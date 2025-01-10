use std::io::{self, Read};

use crossterm::terminal;

fn main() {
    terminal::enable_raw_mode().expect("able to enable raw mode");

    io::stdin().bytes().any(|byte| {
        let byte = byte.expect("successful read");
        let ch = byte as char;

        if ch.is_control() {
            println!("Binary: {0:08b} ASCII: {0:#03} \r", byte);
        } else {
            println!(
                "Binary: {0:08b} ASCII: {0:#03} Character: {1:#?} \r",
                byte, ch
            );
        }

        let quit = ch == 'q';
        quit
    });

    terminal::disable_raw_mode().expect("able to disable raw mode");
}
