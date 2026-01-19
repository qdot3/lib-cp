use ops::{marker::Commutative, Monoid, SemiGroup};

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
    /// 初めの`n`要素について累積計算した結果を返す。
    ///
    /// # Time Complexity
    ///
    /// *Θ*(log *N*)
    pub fn prefix_query(&self, mut n: usize) -> T::Set {
        let mut res = T::id();
        while n > 0 {
            res = T::op(self.lefts[n - 1], res);
            n -= 1 << n.trailing_zeros();
        }

        res
    }

    /// `i`番目の要素を差分計算する。
    /// つまり、`T::op(self, diff)`で更新する。
    ///
    /// # Time Complexity
    ///
    /// *Θ*(log *N*)
    pub fn point_update(&mut self, mut i: usize, diff: T::Set) {
        i += 1;
        while i <= self.lefts.len() {
            self.lefts[i - 1] = T::op(self.lefts[i - 1], diff);
            i += 1 << i.trailing_zeros()
        }
    }
}

impl<T> FenwickTree<T>
where
    T: Monoid + Commutative,
    T::Set: Copy,
{
    pub fn new(n: usize) -> Self {
        Self {
            lefts: vec![T::id(); n].into_boxed_slice(),
        }
    }
}

impl<T> From<Vec<T::Set>> for FenwickTree<T>
where
    T: SemiGroup + Commutative,
    T::Set: Copy,
{
    /// # Time Complexity
    ///
    /// *Θ*(*N* log *N*)
    fn from(mut value: Vec<T::Set>) -> Self {
        for i in (0..value.len()).rev() {
            let mut j = i + 1;
            j += 1 << j.trailing_zeros();
            while j <= value.len() {
                value[i] = T::op(value[i], value[j - 1]);
                j += 1 << j.trailing_zeros()
            }
        }

        Self {
            lefts: value.into_boxed_slice(),
        }
    }
}
