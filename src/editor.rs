use crossterm::{
    event::{self, Event, KeyCode},
    terminal,
};

pub struct Editor;

impl Editor {
    pub fn run(&self) {
        terminal::enable_raw_mode().expect("able to enable raw mode");

        loop {
            match event::read() {
                Ok(Event::Key(event)) => {
                    println!("{:?} \r", event);

                    if let KeyCode::Char(ch) = event.code {
                        if ch == 'q' {
                            break;
                        }
                    }
                }
                Err(err) => eprintln!("Error: {}", err),
                _ => {}
            }
        }

        terminal::disable_raw_mode().expect("able to disable raw mode");
    }
}
