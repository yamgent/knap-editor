use anyhow::Result;

pub struct Buffer {
    // TODO: Any way to make it private?
    pub content: Vec<String>,
}

impl Buffer {
    pub fn new() -> Self {
        Self { content: vec![] }
    }

    pub fn new_from_file<T: AsRef<str>>(filename: T) -> Result<Self> {
        let content = std::fs::read_to_string(filename.as_ref())?;

        Ok(Self {
            content: content.lines().map(ToString::to_string).collect(),
        })
    }

    pub fn get_line_len(&self, idx: usize) -> usize {
        self.content.get(idx).map_or(0, String::len)
    }
}
