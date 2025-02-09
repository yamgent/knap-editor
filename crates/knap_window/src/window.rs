use anyhow::Result;

use crate::terminal;

pub struct Window;

impl Window {
    pub fn new() -> Self {
        Self
    }

    pub fn set_title(&self, title: &str) -> Result<()> {
        terminal::set_title(title)?;
        Ok(())
    }
}
