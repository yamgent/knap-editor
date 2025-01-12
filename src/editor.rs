use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal,
};

pub struct Editor {
    should_quit: bool,
}

impl Editor {
    pub fn new() -> Self {
        Self { should_quit: false }
    }

    pub fn run(&mut self) {
        if let Err(err) = self.repl() {
            panic!("{err:#?}");
        }
        print!("Goodbye.\r\n");
    }

    fn repl(&mut self) -> Result<(), std::io::Error> {
        terminal::enable_raw_mode()?;

        while !self.should_quit {
            if let Event::Key(KeyEvent {
                code,
                modifiers,
                kind,
                state,
            }) = event::read()?
            {
                println!("{code:?} {modifiers:?} {kind:?} {state:?} \r");

                if matches!(
                    (code, modifiers),
                    (KeyCode::Char('q'), KeyModifiers::CONTROL)
                ) {
                    self.should_quit = true;
                }
            }
        }

        terminal::disable_raw_mode()?;
        Ok(())
    }
}
