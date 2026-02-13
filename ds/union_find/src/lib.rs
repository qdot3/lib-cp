use ops::{marker::Commutative, SemiGroup};

/// 素集合を管理するデータ構造。
#[derive(Debug, Clone)]
pub struct UnionFind<T>
where
    T: Commutative + SemiGroup<Set: Copy>,
{
    /// 非負なら親へのポインター、負なら要素数を表す。
    parent_or_size: Box<[i32]>,

    /// ノードのもつ値
    value: Box<[T::Set]>,
}

impl<T> UnionFind<T>
where
    T: Commutative + SemiGroup<Set: Copy>,
{
    /// `n`要素で初期化する。
    ///
    /// # Time Complexity
    ///
    /// *Θ*(*N*)
    pub fn new(value: Box<[T::Set]>) -> Self {
        Self {
            parent_or_size: vec![-1; value.len()].into_boxed_slice(),
            value,
        }
    }

    /// ノードの値を必要としない場合
    ///
    /// # Time Complexity
    ///
    /// *Θ*(*N*)
    pub fn with_size(n: usize) -> UnionFind<()> {
        UnionFind {
            parent_or_size: vec![-1; n].into_boxed_slice(),
            value: vec![(); n].into_boxed_slice(),
        }
    }

    /// `x`が所属する素集合の代表元のインデックスを返す。
    /// 新たに辺を張らない限り不変。
    ///
    /// # Time Complexity
    ///
    /// *O*(α(*N*)) amortized
    pub fn find(&mut self, mut x: usize) -> usize {
        // path halving
        loop {
            let px = self.parent_or_size[x];
            if px.is_negative() {
                break x;
            }

            let ppx = self.parent_or_size[px as usize];
            if ppx.is_negative() {
                break px as usize;
            } else {
                self.parent_or_size[x] = ppx;
                x = ppx as usize
            }
        }
    }

    /// `x`が所属する素集合の代表元のもつ値を返す。
    ///
    /// # Time Complexity
    ///
    /// *O*(α(*N*)) amortized
    pub fn find_value(&mut self, x: usize) -> T::Set {
        self.value[self.find(x)]
    }

    /// `x`と`y`の所属する素集合を結合する。
    /// すでに同じ集合に属している場合は`false`を、そうでない場合は`true`を返す。
    ///
    /// # Time Complexity
    ///
    /// *O*(α(*N*)) amortized
    pub fn union(&mut self, mut x: usize, mut y: usize) -> bool {
        x = self.find(x);
        y = self.find(y);
        if x == y {
            return false;
        }

        // union by size
        if self.parent_or_size[x] > self.parent_or_size[y] {
            std::mem::swap(&mut x, &mut y);
        }
        self.parent_or_size[x] += self.parent_or_size[y];
        self.value[x] = T::op(self.value[x], self.value[y]);
        self.parent_or_size[y] = x as i32;

        true
    }

    /// `x`と`y`が同じ素集合に所属している場合は`true`を、そうでない場合は`false`を返す。
    ///
    /// # Time Complexity
    ///
    /// *O*(α(*N*)) amortized
    pub fn same(&mut self, x: usize, y: usize) -> bool {
        self.find(x) == self.find(y)
    }

    /// `x`が所属する素集合の要素数を返す。
    ///
    /// # Time Complexity
    ///
    /// *O*(α(*N*)) amortized
    pub fn size(&mut self, x: usize) -> usize {
        -self.parent_or_size[self.find(x)] as usize
    }

    pub fn leaders(&self) -> impl Iterator<Item = (&i32, &T::Set)> {
        self.parent_or_size
            .iter()
            .zip(self.value.iter())
            .filter(|(i, _)| i.is_negative())
    }
}
