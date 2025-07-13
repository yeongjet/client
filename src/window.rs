pub struct Window<const N: usize> {
    pub title: String,
}

impl<const N: usize> Window<N> {
    pub fn new(title: String) -> Self {
        Self { title }
    }
}
