use std::ops::{Index, IndexMut};

#[derive(Debug, Clone)]
pub struct NDArray<T, const D: usize> {
    data: Box<[T]>,
    len: [usize; D],
}

impl<T, const D: usize> NDArray<T, D> {
    pub fn repeat(len: [usize; D], value: T) -> Self
    where
        T: Clone,
    {
        let n = len
            .iter()
            .try_fold(1_usize, |prod, l| prod.checked_mul(*l))
            .expect("msg");

        Self {
            data: vec![value; n].into_boxed_slice(),
            len,
        }
    }

    pub fn fill(&mut self, value: T)
    where
        T: Clone,
    {
        self.data.fill(value);
    }
}

impl<T, const D: usize> Index<[usize; D]> for NDArray<T, D> {
    type Output = T;

    fn index(&self, index: [usize; D]) -> &Self::Output {
        let i = self
            .len
            .iter()
            .zip(index)
            .try_fold(0_usize, |acc, (l, i)| acc.checked_mul(*l)?.checked_add(i))
            .expect("msg");

        &self.data[i]
    }
}

impl<T, const D: usize> IndexMut<[usize; D]> for NDArray<T, D> {
    fn index_mut(&mut self, index: [usize; D]) -> &mut Self::Output {
        let i = self
            .len
            .iter()
            .zip(index)
            .try_fold(0_usize, |acc, (l, i)| acc.checked_mul(*l)?.checked_add(i))
            .expect("msg");

        &mut self.data[i]
    }
}
