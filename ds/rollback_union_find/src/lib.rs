use std::{
    fmt::Debug,
    rc::{Rc, Weak},
};

use ops::{marker::Commutative, Monoid};

/// 操作の巻き戻しが可能な素集合データ構造。
#[derive(Debug, Clone)]
pub struct RollbackUnionFind<T>
where
    T: Commutative + Monoid<Set: Copy>,
{
    /// `i32`: pointer to the parent or size of the set.
    /// `T`: value of the group.
    node: Box<[(i32, T::Set)]>,
    history: Vec<Rc<[(u32, (i32, T::Set)); 2]>>,
}

impl<T> RollbackUnionFind<T>
where
    T: Commutative + Monoid<Set: Copy>,
{
    /// サイズ`n`の集合で初期化する。
    /// 復帰点は設定されない。
    ///
    /// # Time Complexity
    ///
    /// *O*(*N*)
    #[inline(always)]
    pub fn new(n: usize) -> Self {
        Self {
            node: vec![(-1, T::id()); n].into_boxed_slice(),
            history: Vec::with_capacity(n),
        }
    }

    /// 各要素が値を持つ場合の初期化子。
    /// 復帰点は設定されない。
    ///
    /// # Time Complexity
    ///
    /// *O*(*N*)
    #[inline(always)]
    pub fn with_values(values: Vec<T::Set>) -> Self {
        Self {
            history: Vec::with_capacity(values.len()),
            node: Vec::from_iter(values.into_iter().map(|v| (-1, v))).into_boxed_slice(),
        }
    }

    /// `x`が所属する集合の代表元を返す。
    /// [`merge`]しない限り、代表点は固定される。
    ///
    /// # Time Complexity
    ///
    /// *O*(log *N*)
    ///
    /// [`merge`]: Self::merge
    #[inline(always)]
    pub fn find(&mut self, mut x: usize) -> usize {
        while !self.node[x].0.is_negative() {
            x = self.node[x].0 as usize
        }

        x
    }

    /// `x`と`y`が同じ集合に属しているか判定する。
    ///
    /// # Time Complexity
    ///
    /// *O*(log *N*)
    ///
    /// [`merge`]: Self::merge
    pub fn same(&mut self, x: usize, y: usize) -> bool {
        self.find(x) == self.find(y)
    }

    /// `x`が所属する集合の要素数と値を返す。
    ///
    /// # Time Complexity
    ///
    /// *O*(log *N*)
    #[inline(always)]
    pub fn size_value(&mut self, x: usize) -> (usize, T::Set) {
        let (size, value) = self.node[self.find(x)];
        (-size as usize, value)
    }

    /// `x`と`y`がそれぞれ所属する集合をまとめる。
    /// 元から同じ集合に属していた場合は`false`を返し、そうでなければ`true`を返す。
    ///
    /// # Time Complexity
    ///
    /// *O*(log *N*)
    pub fn merge(&mut self, mut x: usize, mut y: usize) -> bool {
        x = self.find(x);
        y = self.find(y);

        if x == y {
            return false;
        }

        self.history.push(Rc::new([
            (x as u32, self.node[x]),
            (y as u32, self.node[y]),
        ]));

        // マージテク
        if self.node[x].0 > self.node[y].0 {
            std::mem::swap(&mut x, &mut y);
        }
        self.node[x] = (
            self.node[x].0 + self.node[y].0,
            T::op(self.node[x].1, self.node[y].1),
        );
        self.node[y].0 = x as i32;

        true
    }

    /// `(代表元、メンバーの数、値)`をイテレートする。
    ///
    /// # Time Complexity
    ///
    /// *O*(*N*)
    #[inline(always)]
    pub fn leaders(&self) -> impl Iterator<Item = (usize, usize, T::Set)> + '_ {
        self.node.iter().enumerate().filter_map(|(i, v)| {
            if v.0.is_negative() {
                Some((i, -v.0 as usize, v.1))
            } else {
                None
            }
        })
    }

    /// 有効な`union`操作を１回分だけ巻き戻す。
    /// 初期状態の場合は何もしない。
    ///
    /// # Time Complexity
    ///
    /// *O*(1)
    #[inline(always)]
    pub fn undo(&mut self) {
        if let Some(op) = self.history.pop() {
            let [x, y] = Rc::into_inner(op).unwrap();
            self.node[x.0 as usize] = x.1;
            self.node[y.0 as usize] = y.1;
        }
    }

    /// 状態を表すユニークなキーを返す。
    ///
    /// # Time Complexity
    ///
    /// *O*(1)
    #[inline(always)]
    pub fn get_key(&mut self) -> Key<T> {
        if let Some(key) = self.history.last() {
            Key(Rc::downgrade(key))
        } else {
            todo!()
        }
    }

    /// 有効なキーに対応する復帰点まで状態を回復する。
    /// 取り消されたキーは自動的に無効化される。
    ///
    /// # Time Complexity
    ///
    /// *O*(1) (amortized)
    ///
    /// # Example
    ///
    /// ```
    /// use rollback_union_find::{RollbackUnionFind, Key};
    ///
    /// let mut ruf = RollbackUnionFind::<()>::new(3);
    ///
    /// assert!(ruf.merge(0, 1)); // (0) + (1) + (2) -> (0, 1) + (2)
    /// let key_oox = ruf.get_key();
    /// assert!(ruf.merge(0, 2)); // (0, 1) + (2) -> (0, 1, 2)
    /// let key_ooo = ruf.get_key();
    ///
    /// // rollback to (0, 1) + (2)
    /// ruf.rollback(&key_oox);
    /// assert!(key_oox.is_valid() && !key_ooo.is_valid());
    /// assert!(ruf.same(0, 1) && !ruf.same(1, 2));
    /// ```
    #[inline(always)]
    pub fn rollback(&mut self, key: &Key<T>) {
        debug_assert!(self.history.iter().all(|diff| Rc::strong_count(diff) == 1));
        if let Some(_) = key.0.upgrade() {
            while let Some(diff) = self.history.pop_if(|diff| Rc::strong_count(diff) == 1) {
                let [x, y] = Rc::into_inner(diff).unwrap();
                self.node[x.0 as usize] = x.1;
                self.node[y.0 as usize] = y.1;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Key<T>(Weak<[(u32, (i32, T::Set)); 2]>)
where
    T: Commutative + Monoid;

impl<T> Key<T>
where
    T: Commutative + Monoid,
{
    /// キーが有効な場合に`true`を返す。
    ///
    /// # Time Complexity
    ///
    /// *O*(1)
    pub fn is_valid(&self) -> bool {
        self.0.strong_count() > 0
    }
}

#[cfg(test)]
mod tests {
    use rand::{self, Rng};

    use super::*;

    #[test]
    fn undo_random() {
        let mut rng = rand::rng();
        let mut test_once = |n: usize, q: usize, p: f64| {
            let mut dsu = RollbackUnionFind::<()>::new(n);
            let mut stack = Vec::with_capacity(q);
            for _ in 0..q {
                match rng.random_bool(p) {
                    true => {
                        dsu.undo();
                        stack.pop();
                    }
                    false => {
                        let (x, y) = (rng.random_range(..n), rng.random_range(..n));
                        if dsu.merge(x, y) {
                            stack.push((x, y));
                        }
                    }
                }

                let l = Vec::from_iter(dsu.leaders());

                let mut dsu2 = RollbackUnionFind::<()>::new(n);
                stack.iter().for_each(|(x, y)| {
                    dsu2.merge(*x, *y);
                });
                let l2 = Vec::from_iter(dsu2.leaders());

                assert_eq!(l, l2, "{:?}", &stack);
            }
        };

        let n = 1 << 8;
        let q = 1 << 10;
        for p in rand::rng().random_iter::<u8>().take(100) {
            test_once(n, q, p as f64 / u8::MAX as f64);
        }
    }
}
