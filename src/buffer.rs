pub struct Buffer {
    pub content: Vec<String>,
}

impl Buffer {
    pub fn new() -> Self {
        // TODO: Remove "Hello world" debug content
        Self {
            content: vec!["Hello world".to_string()],
        }
    }
}
