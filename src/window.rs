pub struct Window {
    pub title: String,
    pub buffer_size: u32,
}

impl Window {
    pub fn new(title: String, buffer_size: u32) -> Self {
        Self { title, buffer_size }
    }
}
