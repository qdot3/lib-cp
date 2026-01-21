use std::mem::MaybeUninit;

pub struct SmallSize<const N: usize>;
pub trait SupportedSmallSize {}

pub struct SortedArray<T, const N: usize>
where
    T: Ord,
    SmallSize<N>: SupportedSmallSize,
{
    array: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> SortedArray<T, N>
where
    T: Ord,
    SmallSize<N>: SupportedSmallSize,
{
    pub fn new() -> Self {
        Self {
            array: std::array::from_fn(|_| MaybeUninit::uninit()),
            len: 0,
        }
    }
}
