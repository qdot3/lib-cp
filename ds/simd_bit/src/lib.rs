use std::ops::{Add, BitAnd, RangeBounds, Sub};

pub trait SimdAbel: Copy + Add<Output = Self> + Sub<Output = Self> + BitAnd<Output = Self> {
    const ZERO: Self;
    const ONES: Self;
}

macro_rules! simd_abel_impl {
    ($( $t:ty )*) => {$(
        impl SimdAbel for $t {
            const ZERO: Self = 0;
            const ONES: Self = !0;
        }
    )*};
}
simd_abel_impl!( u8 u16 u32 u64 usize i8 i16 i32 i64 isize );

pub struct LaneCount<const B: usize>;

pub trait SupportedLaneCount {}

macro_rules! supported_lane_count_impl {
    ($( $b:expr )*) => {$(
        impl SupportedLaneCount for LaneCount<$b> {}
    )*};
}
supported_lane_count_impl!( 1 2 4 8 16 32 64 );

#[derive(Debug, Clone)]
pub struct SimdBIT<T, const B: usize>
where
    T: SimdAbel,
    LaneCount<B>: SupportedLaneCount,
{
    bit: Box<[T]>,
    /// sum over i-th segments corresponds to the i-th (implicit) leaf of bit
    data: Box<[[T; B]]>,
    len: usize,
}

impl<T, const B: usize> SimdBIT<T, B>
where
    T: SimdAbel,
    LaneCount<B>: SupportedLaneCount,
{
    /// Adds `diff` to the `i`-th element.
    ///
    /// # Time Complexity
    ///
    /// *O*(log *N/B* + *B*)
    pub fn point_add(&mut self, i: usize, diff: T) {
        let mask: [[T; B]; B] = const {
            let mut mask = [[T::ZERO; B]; B];

            let mut i = 0;
            while i < B {
                let mut j = i;
                while j < B {
                    mask[i][j] = T::ONES;
                    j += 1
                }
                i += 1
            }

            mask
        };

        let [mut b, i] = [i / B, i % B];

        // update block
        {
            let block = &mut self.data[b];
            let mask = mask[i];
            for i in 0..B {
                block[i] = block[i] + (mask[i] & diff);
            }
        }

        // update BIT (0-based indexing)
        while let Some(v) = self.bit.get_mut(b) {
            *v = *v + diff;

            b |= b + 1;
        }
    }

    /// Returns `i`-th element
    ///
    /// # Time Complexity
    ///
    /// *O*(1)
    pub fn get(&self, i: usize) -> T {
        let [b, i] = [i / B, i % B];

        let data = self.data[b];
        if i > 0 {
            data[i] - data[i - 1]
        } else {
            data[0]
        }
    }

    /// Returns the sum over the first `n` elements.
    ///
    /// This is equivalent to `values[..n].iter().sum()`.
    ///
    /// # Time Complexity
    ///
    /// *O*(log *N/B*)
    pub fn prefix_sum(&self, n: usize) -> T {
        assert!(n <= self.len);

        if let Some(n) = n.checked_sub(1) {
            let [mut b, i] = [n / B, n % B];
            let mut sum = self.data[b][i];
            // 1-based indexing
            while b > 0 {
                sum = sum + self.bit[b - 1];
                b &= b - 1;
            }

            sum
        } else {
            T::ZERO
        }
    }

    /// Returns the sum over the `range`.
    ///
    /// This is equivalent to `values[range].iter().sum()`.
    ///
    /// # Time Complexity
    ///
    /// *O*(log *N/B*)
    pub fn range_sum<R>(&self, range: R) -> T
    where
        R: RangeBounds<usize>,
    {
        let r = match range.end_bound() {
            std::ops::Bound::Included(r) => r + 1,
            std::ops::Bound::Excluded(r) => *r,
            std::ops::Bound::Unbounded => self.len,
        };
        let l = match range.start_bound() {
            std::ops::Bound::Included(l) => *l,
            std::ops::Bound::Excluded(l) => l + 1,
            std::ops::Bound::Unbounded => return self.prefix_sum(r),
        };

        if (l..r).is_empty() {
            return T::ZERO;
        }

        if l.leading_zeros() != r.leading_zeros() {
            self.prefix_sum(r) - self.prefix_sum(l)
        } else {
            debug_assert!(l > 0 && r > 0);
            let [mut bl, il] = [(l - 1) / B, (l - 1) % B];
            let [mut br, ir] = [(r - 1) / B, (r - 1) % B];

            let lcp = {
                let lcp_len = usize::BITS - (bl ^ br).leading_zeros();
                bl >> lcp_len << lcp_len
            };

            // 0-based indexing
            let mut sum = self.data[br][ir];
            // 1-based indexing
            while br != lcp {
                sum = sum + self.bit[br - 1];
                br &= br - 1;
            }

            sum = sum - self.data[bl][il];
            while bl != lcp {
                sum = sum - self.bit[bl - 1];
                bl &= bl - 1;
            }

            sum
        }
    }

    pub fn partition_point<P>(&self, mut pred: P) -> (usize, T)
    where
        P: FnMut(T) -> bool,
    {
        let mut b = 0;
        let mut sum = T::ZERO;

        // 1-based indexing
        let mut additional = 1 << self.bit.len().ilog2();
        while additional > 0 {
            let temp = sum + self.bit[b + additional - 1];
            if pred(temp) {
                sum = temp;
                b += additional
            }
            while {
                additional /= 2;
                b + additional > self.bit.len()
            } {}
        }

        let i = self.data[b].partition_point(|v| pred(sum + *v));
        if i > 0 {
            sum = sum + self.data[b][i - 1]
        }

        ((b * B + i).min(self.len), sum)
    }
}

impl<T, const B: usize> From<Vec<T>> for SimdBIT<T, B>
where
    T: SimdAbel,
    LaneCount<B>: SupportedLaneCount,
{
    fn from(mut value: Vec<T>) -> Self {
        let len = value.len();
        let mut data = {
            value.extend_from_slice(&[T::ZERO; B][..(B - len % B) % B]);
            // FIXME: use `Vec::into_chunks()` if stabilized
            value.as_chunks().0.to_owned().into_boxed_slice()
        };
        let mut bit = vec![T::ZERO; data.len()].into_boxed_slice();

        for (b, chunk) in data.iter_mut().enumerate() {
            for i in 1..B {
                chunk[i] = chunk[i] + chunk[i - 1]
            }

            let sum = chunk[B - 1] + bit[b];
            bit[b] = sum;
            if let Some(v) = bit.get_mut(b | (b + 1)) {
                *v = *v + sum
            }
        }

        Self { bit, data, len }
    }
}
