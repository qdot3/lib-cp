use std::ops::RangeBounds;

use ops::Monoid;

#[derive(Debug, Clone)]
pub struct Segtree<T>
where
    T: Monoid,
{
    /// full binary tree.
    /// `data.len() = (offset + n_leave).next_multiple_of(2)`
    data: Box<[T::Set]>,
    offset: usize,
    net_leaves: usize,
}

impl<T> Segtree<T>
where
    T: Monoid<Set: Copy>,
{
    /// # Time Complexity
    ///
    /// *Θ*(log *N*)
    pub fn point_update_with<F>(&mut self, mut i: usize, f: F)
    where
        F: FnOnce(T::Set) -> T::Set,
    {
        i += self.offset;
        self.data[i] = f(self.data[i]);

        while i > 1 {
            i >>= 1;
            self.data[i] = T::op(self.data[i << 1], self.data[(i << 1) | 1])
        }
    }

    /// # Time Complexity
    ///
    /// *O*(log *N*)
    pub fn range_query<R>(&self, range: R) -> T::Set
    where
        R: RangeBounds<usize>,
    {
        let mut l = match range.start_bound() {
            std::ops::Bound::Included(&l) => l,
            std::ops::Bound::Excluded(&l) => l.checked_sub(1).unwrap(),
            std::ops::Bound::Unbounded => 0,
        } + self.offset;
        let mut r = match range.end_bound() {
            std::ops::Bound::Included(&r) => self.offset + r + 1,
            std::ops::Bound::Excluded(&r) => self.offset + r,
            std::ops::Bound::Unbounded => self.data.len(),
        };

        if (l..r).is_empty() {
            return T::id();
        }

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

    /// # Time Complexity
    ///
    /// *O*(1)
    pub fn range_full_query(&self) -> T::Set {
        self.data[1]
    }

    /// # Time Complexity
    ///
    /// *Θ*(log *N*)
    pub fn partition_right<F>(&self, mut l: usize, mut pred: F) -> usize
    where
        F: FnMut(T::Set) -> bool,
    {
        // verified with <https://atcoder.jp/contests/practice2/tasks/practice2_j>
        l += self.offset;
        l >>= l.trailing_zeros();

        let mut acc = T::id();
        let mut temp;

        // go up
        while {
            temp = T::op(acc, self.data[l]);
            pred(temp)
        } {
            acc = temp;
            l += 1;
            l >>= l.trailing_zeros();

            if l == 1 {
                return self.net_leaves;
            }
        }

        // go down
        while {
            l <<= 1;
            l < self.data.len()
        } {
            temp = T::op(acc, self.data[l]);
            if pred(temp) {
                acc = temp;
                l |= 1;
            }
        }
        l >>= 1;

        (l ^ self.offset).min(self.net_leaves)
    }

    /// # Time Complexity
    ///
    /// *Θ*(log *N*)
    ///
    /// # Example
    ///
    /// ```
    /// use segtree::Segtree;
    /// use ops::ops::Additive;
    ///
    /// let n = 300;
    /// let segtree = Segtree::<Additive<u32>>::from(vec![1; n]);
    ///
    /// for r in 0..n {
    ///     for sum in 0..=r as u32 {
    ///         let l = segtree.partition_left(r, |v| v <= sum);
    ///
    ///         assert_eq!(l, r - sum as usize);
    ///         assert!((l..r).all(|l| segtree.range_query(l..r) <= sum));
    ///         assert!((0..l).all(|l| segtree.range_query(l..r) >  sum));
    ///     }
    /// }
    /// ```
    pub fn partition_left<F>(&self, mut r: usize, mut pred: F) -> usize
    where
        F: FnMut(T::Set) -> bool,
    {
        r += self.offset;
        r >>= r.trailing_zeros();

        let mut acc = T::id();
        let mut temp;

        // go up
        while {
            temp = T::op(self.data[r - 1], acc);
            pred(temp)
        } {
            acc = temp;

            if r.is_power_of_two() {
                return 0;
            }

            r -= 1;
            r >>= r.trailing_zeros();
        }

        // go down
        r -= 1;
        while {
            r <<= 1;
            r < self.data.len()
        } {
            temp = T::op(self.data[r | 1], acc);
            if pred(temp) {
                acc = temp;
            } else {
                r |= 1
            }
        }
        r >>= 1;

        (r ^ self.offset) + 1
    }
}

impl<T> From<Vec<T::Set>> for Segtree<T>
where
    T: Monoid<Set: Copy>,
{
    /// # Panics
    ///
    /// - `value.len()` should be less than `isize::MAX`
    fn from(value: Vec<T::Set>) -> Self {
        assert!(
            value.len() < usize::MAX / 2,
            "given data is too large to allocate buffer."
        );

        // never overflow
        let offset = value.len().next_power_of_two();
        let capacity = offset + value.len() + (value.len() & 1);
        let net_leaves = value.len();

        // initialize leaves
        let mut data = Vec::with_capacity(capacity);
        data.extend(std::iter::repeat_n(T::id(), offset));
        data.extend(value);
        if data.len() != capacity {
            data.push(T::id());
        }
        debug_assert_eq!(data.len(), capacity, "capacity - data.len() = 0 or 1");

        // update internal nodes
        let mut data = data.into_boxed_slice();
        for i in (1..capacity / 2).rev() {
            data[i] = T::op(data[i << 1], data[(i << 1) | 1])
        }

        Self {
            data,
            offset,
            net_leaves,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partition_left() {}
}
