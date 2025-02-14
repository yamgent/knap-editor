use knap_base::color::Color;

// TODO: Stub, reimplement in the future
pub struct TextHighlights;

impl TextHighlights {
    pub fn new() -> Self {
        Self {}
    }

    pub(crate) fn get_highlight_at(&self, byte_idx: usize) -> Option<TextColor> {
        None
    }
}

// TODO: This can be part of theme in the future
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextColor {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
}
