use std::{fmt::Debug, ops::RangeBounds};

use ops::{marker::Commutative, Group, Monoid, SemiGroup};

/// 可換な半群の値の列に対して、一点更新・累積クエリを高速に計算するデータ構造。
#[derive(Debug, Clone)]
pub struct FenwickTree<T: SemiGroup + Commutative> {
    lefts: Box<[T::Set]>,
}

impl<T> FenwickTree<T>
where
    T: Monoid + Commutative,
    T::Set: Copy,
{
    /// 単位元で初期化する。
    ///
    /// # Time Complexity
    ///
    /// *O*(*N*)
    pub fn new(n: usize) -> Self {
        Self {
            lefts: vec![T::id(); n].into_boxed_slice(),
        }
    }

    /// `0..n`番目の要素について累積計算した結果を返す。
    ///
    /// # Time Complexity
    ///
    /// *Θ*(log *N*)
    pub fn prefix_query(&self, mut n: usize) -> T::Set {
        let mut res = T::id();
        while n > 0 {
            res = T::op(self.lefts[n - 1], res);
            n &= n - 1
        }

        res
    }

    /// `i`番目の要素を差分計算する。
    /// つまり、`T::op(self, additional)`で更新する。
    ///
    /// # Time Complexity
    ///
    /// *Θ*(log *N*)
    pub fn point_update(&mut self, mut i: usize, additional: T::Set) {
        i += 1;
        while i <= self.lefts.len() {
            self.lefts[i - 1] = T::op(self.lefts[i - 1], additional);
            i += i & i.wrapping_neg()
        }
    }

    /// 条件`pred`を満たす最大の累積値と合成された要素数を返す。
    ///
    /// より形式的には、`i`番目までの累積値`s_i`について、下記を満たす`j`を返す。
    /// ```txt
    /// pred(s_i) = true      for all i < j
    /// pred(s_i) = false     for all i >= j
    /// ```
    ///
    /// # Time Complexity
    ///
    /// *Θ*(log *N*)
    ///
    /// # Example
    ///
    /// ```
    /// use ops::ops::Additive;
    /// use fenwick::FenwickTree;
    ///
    /// let prefix_sum = FenwickTree::<Additive<u32>>::from_iter(0..10);
    ///
    /// let a = 0 + 1 + 2 + 3 + 4;
    /// let (i, sum) = prefix_sum.partition_point(|v| *v <= a);
    /// assert_eq!(i, 5);
    /// assert_eq!(sum, 0 + 1 + 2 + 3 + 4);
    ///
    /// let (i, sum) = prefix_sum.partition_point(|v| *v < 1_000_000);
    /// assert_eq!(i, 10);
    /// assert_eq!(sum, 45);
    /// ```
    pub fn partition_point<P>(&self, mut pred: P) -> (usize, T::Set)
    where
        P: FnMut(&T::Set) -> bool,
        T::Set: Debug,
    {
        let mut i = 0;
        let mut prefix = T::id();
        // ブロックサイズを小さくしていく
        let mut additional = 1 << self.lefts.len().ilog2();
        while additional > 0 {
            let temp = T::op(prefix, self.lefts[i + additional - 1]);
            if pred(&temp) {
                prefix = temp;
                i += additional
            }
            while {
                additional >>= 1;
                i + additional > self.lefts.len()
            } {}
        }

        (i, prefix)
    }
}

impl<T> FenwickTree<T>
where
    T: Group + Commutative,
    T::Set: Copy,
{
    pub fn range_query<R>(&self, range: R) -> T::Set
    where
        R: RangeBounds<usize>,
    {
        // 1-based で`l+1..=r`が所望の区間
        let mut l = match range.start_bound() {
            std::ops::Bound::Included(l) => *l,
            std::ops::Bound::Excluded(l) => l + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let mut r = match range.end_bound() {
            std::ops::Bound::Included(r) => r + 1,
            std::ops::Bound::Excluded(r) => *r,
            std::ops::Bound::Unbounded => self.lefts.len(),
        };

        let mut res = T::id();
        while l != r {
            if r > l {
                // r > 0
                res = T::op(res, self.lefts[r - 1]);
                r &= r - 1
            } else {
                // l > 0
                res = T::op(res, T::inv(self.lefts[l - 1]));
                l &= l - 1
            }
        }
        res
    }
}

impl<T> From<Vec<T::Set>> for FenwickTree<T>
where
    T: Monoid + Commutative,
    T::Set: Copy,
{
    /// # Time Complexity
    ///
    /// *Θ*(*N*)
    fn from(mut value: Vec<T::Set>) -> Self {
        // 正順に親を更新していく
        for i in 0..value.len() {
            let mut p = i + 1;
            p += 1 << p.trailing_zeros();
            if p <= value.len() {
                value[p - 1] = T::op(value[p - 1], value[i]);
            }
        }

        Self {
            lefts: value.into_boxed_slice(),
        }
    }
}

impl<T> FromIterator<T::Set> for FenwickTree<T>
where
    T: Monoid + Commutative,
    T::Set: Copy,
{
    /// # Time Complexity
    ///
    /// *Θ*(*N*)
    fn from_iter<I: IntoIterator<Item = T::Set>>(iter: I) -> Self {
        Self::from(Vec::from_iter(iter))
    }
}
