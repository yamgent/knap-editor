use std::panic;

use anyhow::Result;
use knap_base::math::Vec2f;

use crate::terminal;

pub struct Window;

fn setup_panic_hook() {
    let current_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // we can't do anything to recover if end_terminal returns an error,
        // so just ignore the Result
        let _ = terminal::end_terminal();
        current_hook(panic_info);
    }));
}

impl Window {
    pub fn new() -> Self {
        Self
    }

    pub fn init(&self) {
        setup_panic_hook();
        terminal::init_terminal().expect("able to initialize terminal");
    }

    pub fn deinit(&self) {
        terminal::end_terminal().expect("able to deinit terminal");
    }

    pub fn size(&self) -> Vec2f {
        terminal::size_f64().expect("able to get terminal size")
    }

    pub fn set_title(&self, title: &str) -> Result<()> {
        terminal::set_title(title)?;
        Ok(())
    }
}
