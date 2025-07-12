use generic_array::ArrayLength;
use std::marker::PhantomData;

pub struct Window<N>
where
    N: ArrayLength,
{
    pub title: String,
    pub buffer_size: u32,
    _marker: PhantomData<N>,
}

impl<N> Window<N>
where
    N: ArrayLength,
{
    pub fn new(title: String) -> Self {
        let buffer_size = N::to_u32();
        Self {
            title,
            buffer_size,
            _marker: PhantomData,
        }
    }
}
