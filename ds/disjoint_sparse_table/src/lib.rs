use ops::SemiGroup;

use std::{fmt::Debug, ops::RangeBounds};

/// 冪等性を仮定しない Sparse Table。
/// `T: Group` なら差分計算の方が速い。
#[derive(Debug, Clone)]
pub struct DisjointSparseTable<T>
where
    T: SemiGroup,
    T::Set: Copy,
{
    /// i 行目では 2^i 個の要素ごとにからの左右からの累積計算をする。
    /// 例えば 0 行目は与えられたデータ列そのものであり、1 列目の最初の８つの要素は
    /// { a[0]+a[1], a[1], a[2], a[2]+a[3], a[4]+a[5], a[5], a[6], a[6]+a[7] } である。
    /// i 行目の要素は table[i*n..(i+1)*n] に格納されている。
    table: Box<[T::Set]>,

    /// １行当たりの要素数
    len: usize,
}

impl<T> DisjointSparseTable<T>
where
    T: SemiGroup,
    T::Set: Copy,
{
    /// # Time Complexity
    ///
    /// *Θ*(1)
    pub fn range_query<R>(&self, range: R) -> Option<T::Set>
    where
        R: RangeBounds<usize>,
    {
        let l = match range.start_bound() {
            std::ops::Bound::Included(l) => *l,
            std::ops::Bound::Excluded(l) => l + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let r = match range.end_bound() {
            std::ops::Bound::Included(r) => r + 1,
            std::ops::Bound::Excluded(r) => *r,
            std::ops::Bound::Unbounded => self.len,
        };

        if l >= r {
            None
        } else if l + 1 == r {
            Some(self.table[l])
        } else {
            // 2^n < r-l <= 2^(n+1) となる n を計算する。
            // これは [l, r) をゼロでない右累積和と左累積和の和に分解できる唯一の n である。
            let n = (l ^ (r - 1)).ilog2() as usize;
            Some(T::op(
                self.table[n * self.len + l],
                self.table[n * self.len + (r - 1)],
            ))
        }
    }
}

impl<T> From<Vec<T::Set>> for DisjointSparseTable<T>
where
    T: SemiGroup,
    // TODO: Copy は過剰。Clone か 参照を引数にとる二項演算を考えるべき。
    T::Set: Copy,
{
    /// # Time Complexity
    ///
    /// *Θ*(*N* log *N*)
    fn from(mut value: Vec<T::Set>) -> Self {
        let len = value.len();

        if len <= 1 {
            return Self {
                table: value.into_boxed_slice(),
                len,
            };
        }

        let extra_row = if len.is_power_of_two() {
            // サイズが 2^n の区間は 2^(n-1) の区間の和として計算するので、2^n の区間は不要
            (len - 1).ilog2()
        } else {
            len.ilog2()
        } as usize;

        value.reserve(len * extra_row);
        for i in 1..=extra_row {
            value.extend_from_within(0..len);
            let width = 1 << i;

            // 偶数個目の区間は右から、奇数個目の区間は左から累積和を計算をする
            for acc_r in value[i * len..].chunks_mut(width).step_by(2) {
                let mut last = acc_r.last().unwrap().clone();
                for first in acc_r.iter_mut().rev().skip(1) {
                    *first = T::op(*first, last);
                    last = *first
                }
            }
            for acc_l in value[i * len..].chunks_mut(width).skip(1).step_by(2) {
                let mut first = acc_l[0];
                for last in acc_l.iter_mut().skip(1) {
                    *last = T::op(first, *last);
                    first = *last
                }
            }
        }

        Self {
            table: value.into_boxed_slice(),
            len,
        }
    }
}

// impl<T> FromIterator<T::Set> for DisjointSparseTable<T>
// where
//     T: SemiGroup,
//     // TODO: Copy は過剰。Clone か 参照を引数にとる二項演算を考えるべき。
//     T::Set: Copy,
// {
//     fn from_iter<I: IntoIterator<Item = T::Set>>(iter: I) -> Self {
//         let iter = iter.into_iter();
//         let (_, max) = iter.size_hint();

//         if  max.is_some_and(|max| max > 1) {}
//         todo!()
//     }
// }

#[cfg(test)]
mod tests {
    use ops::ops::Additive;
    use rand::Rng;

    use super::*;

    /// # Time Complexity
    ///
    /// *Θ*(*N*^2)
    fn template(n: usize) {
        let value = Vec::from_iter(0..n as i32);
        let dst = DisjointSparseTable::<Additive<i32>>::from(value);

        for l in 0..n {
            for r in l..n {
                assert_eq!(
                    dst.range_query(l..=r).unwrap() as usize,
                    (l + r) * (r - l + 1) / 2,
                    "{:?}",
                    l..=r
                )
            }
        }
    }

    #[test]
    fn pow_two() {
        for i in 5..10 {
            template(1 << i);
        }
    }

    #[test]
    fn random() {
        let mut rng = rand::rng();
        for _ in 0..10 {
            template(rng.random_range(1 << 5..1 << 10));
        }
    }
}
