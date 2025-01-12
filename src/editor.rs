use crossterm::{
    event::{self, Event, KeyCode},
    terminal,
};

pub struct Editor;

impl Editor {
    pub fn run(&self) {
        if let Err(err) = self.repl() {
            panic!("{err:#?}");
        }
        print!("Goodbye.\r\n");
    }

    fn repl(&self) -> Result<(), std::io::Error> {
        terminal::enable_raw_mode()?;

        loop {
            if let Event::Key(event) = event::read()? {
                println!("{event:?} \r");

                if let KeyCode::Char(ch) = event.code {
                    if ch == 'q' {
                        break;
                    }
                }
            }
        }

        terminal::disable_raw_mode()?;
        Ok(())
    }
}
