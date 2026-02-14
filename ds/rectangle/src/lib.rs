use std::ops::{Index, IndexMut};

use num::Zero;

#[derive(Debug, Clone)]
pub struct Rectangle<T> {
    buffer: Vec<T>,
    width: usize,
}

impl<T: Clone> Rectangle<T> {
    pub fn new_with(value: T, (width, height): (usize, usize)) -> Rectangle<T> {
        Self {
            buffer: vec![value; width * height],
            width,
        }
    }
}

impl<T: Zero> Rectangle<T> {
    pub fn zero(width: usize, height: usize) -> Self {
        Self {
            buffer: std::iter::repeat_with(|| T::zero())
                .take(width * height)
                .collect(),
            width,
        }
    }
}

impl<T> Index<usize> for Rectangle<T> {
    type Output = [T];

    fn index(&self, index: usize) -> &Self::Output {
        &self.buffer[self.width * index..self.width * (index + 1)]
    }
}

impl<T> IndexMut<usize> for Rectangle<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.buffer[self.width * index..self.width * (index + 1)]
    }
}
