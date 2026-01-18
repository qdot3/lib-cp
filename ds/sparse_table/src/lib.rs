use ops::{marker::Idempotent, SemiGroup};

use std::{ops::RangeBounds, usize};

/// 冪等性を満たす半群について区間クエリを定数時間で返すデータ構造
#[derive(Clone, Debug)]
pub struct SparseTable<T: SemiGroup + Idempotent> {
    table: Box<[T::Set]>,
    partition: Box<[usize]>,
}

impl<T> SparseTable<T>
where
    T: SemiGroup + Idempotent,
    T::Set: Copy,
{
    /// # Time Complexity
    ///
    /// *O*(1)
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
            std::ops::Bound::Unbounded => self.partition[1],
        };

        if l >= r {
            return None;
        }

        let w = (r - l).ilog2() as usize;
        Some(T::op(
            self.table[self.partition[w] + l],
            self.table[self.partition[w] + r - (1 << w)],
        ))
    }
}

impl<T> From<Vec<T::Set>> for SparseTable<T>
where
    T: SemiGroup + Idempotent,
    T::Set: Copy,
{
    /// # Time Complexity
    ///
    /// *Θ*(*N* log *N*)
    fn from(mut value: Vec<T::Set>) -> Self {
        // panic 回避のため
        if value.is_empty() {
            return Self {
                table: Vec::new().into_boxed_slice(),
                partition: Vec::new().into_boxed_slice(),
            };
        }

        // 追加で必要なメモリは Σ_{0 <= k <= ⌊lg n⌋} (n - 2^k) < ⌊lg n⌋ n + 1
        let height = value.len().ilog2() as usize + 1;
        value.reserve(height * value.len());

        let mut partition = Vec::with_capacity(height + 2);
        partition.extend_from_slice(&[0, value.len()]);

        for i in 0..height {
            let half = 1 << i;
            for j in (partition[i]..partition[i + 1]).skip(half) {
                value.push(T::op(value[j - half], value[j]));
            }
            partition.push(value.len());
        }

        Self {
            table: value.into_boxed_slice(),
            partition: partition.into_boxed_slice(),
        }
    }
}

impl<T> FromIterator<T::Set> for SparseTable<T>
where
    T: SemiGroup + Idempotent,
    T::Set: Copy,
{
    /// # Time Complexity
    ///
    /// *Θ*(*N* log *N*)
    fn from_iter<I: IntoIterator<Item = T::Set>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (_, max) = iter.size_hint();

        // TODO: 競プロでは十分だが、改良したい
        if max.is_some_and(|n| n >> (usize::BITS / 2) == 0 && n != 0) {
            let n = max.unwrap();

            // 合計の要素数は高々 (⌊lg n⌋ + 1) n + 1。From<Vec<T>> を参照
            // 事前に容量を確保すると、N 回分のコピーを削減できる。
            let capacity = (n.ilog2() as usize + 1) * n;
            let mut vec = Vec::with_capacity(capacity);
            vec.extend(iter);
            Self::from(vec)
        } else {
            Self::from(Vec::from_iter(iter))
        }
    }
}
