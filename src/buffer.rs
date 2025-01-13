use anyhow::Result;

pub struct Buffer {
    pub content: Vec<String>,
}

impl Buffer {
    pub fn new() -> Self {
        Self { content: vec![] }
    }

    pub fn new_from_file<T: AsRef<str>>(filename: T) -> Result<Self> {
        let content = std::fs::read_to_string(filename.as_ref())?;

        Ok(Self {
            content: content.lines().map(|x| x.to_string()).collect(),
        })
    }
}
