use std::ops::RangeBounds;

use ops::Monoid;

#[derive(Debug, Clone)]
pub struct SegmentTree<T: Monoid> {
    data: Box<[T::Set]>,
}

impl<T: Monoid<Set: Copy>> SegmentTree<T> {
    const fn inner_index(&self, i: usize) -> usize {
        self.data.len() / 2 + i
    }

    /// # Time Complexity
    ///
    /// *Θ*(log *N*)
    pub fn point_update(&mut self, i: usize, value: T::Set) {
        self.point_update_with(i, |_| value);
    }

    /// # Time Complexity
    ///
    /// *Θ*(log *N*)
    pub fn point_update_with(&mut self, i: usize, mut f: impl FnMut(T::Set) -> T::Set) {
        let mut i = self.inner_index(i);

        self.data[i] = f(self.data[i]);
        while i > 1 {
            i /= 2;
            self.data[i] = T::op(self.data[i * 2], self.data[i * 2 + 1]);
        }
    }

    /// # Time Complexity
    ///
    /// *Θ*(log *N*)
    pub fn range_query<R>(&self, range: R) -> T::Set
    where
        R: RangeBounds<usize>,
    {
        let mut l = match range.start_bound() {
            std::ops::Bound::Included(l) => self.inner_index(*l),
            std::ops::Bound::Excluded(l) => self.inner_index(l - 1),
            std::ops::Bound::Unbounded => self.inner_index(0),
        };
        let mut r = match range.end_bound() {
            std::ops::Bound::Included(r) => self.inner_index(r + 1),
            std::ops::Bound::Excluded(r) => self.inner_index(*r),
            std::ops::Bound::Unbounded => self.data.len(),
        };

        l >>= l.trailing_zeros();
        r >>= r.trailing_zeros();
        let [mut acc_l, mut acc_r] = [T::id(); 2];
        while {
            if l >= r {
                acc_l = T::op(acc_l, self.data[l]);
                l += 1;
                l >>= l.trailing_zeros();
            } else {
                r -= 1;
                acc_r = T::op(self.data[r], acc_r);
                r >>= r.trailing_zeros();
            }

            l != r
        } {}

        T::op(acc_l, acc_r)
    }
}

impl<T: Monoid<Set: Copy>> From<Vec<T::Set>> for SegmentTree<T> {
    /// # Time Complexity
    ///
    /// *Θ*(*N*)
    fn from(mut value: Vec<T::Set>) -> Self {
        value.extend_from_within(..);
        for i in (0..value.len() / 2).rev() {
            value[i] = T::op(value[i * 2], value[i * 2 + 1])
        }

        Self {
            data: value.into_boxed_slice(),
        }
    }
}
