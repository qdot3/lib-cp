use std::{
    fmt::Debug,
    ops::{Range, RangeBounds},
};

use ops::Monoid;

/// 作用素モノイド M とモノイド S が下記の条件を満たすとき、その組を遅延伝搬構造と呼ぶ。
///
/// - ∀ f, g ∈ M, ∀ x ∈ S, f * (g * x) = (f * g) * x
/// - e ∈ M, ∀ x ∈ S, e * x = x
/// - ∀ f ∈ M, ∀ x, y ∈ S, (f * x) + (f * y) = f * (x + y)
//
// 遅延伝搬の部分だけ記述すれば使えるようにしたい。
// trait の自動実装が難しく、orphan rule を回避するためにこうなった。
//
// 最適化により、match 式は消えるはず
pub enum LazyPropagation<M: Monoid, S: Monoid> {
    SizeDependent(Box<dyn Fn(M::Set, S::Set, usize) -> S::Set>),
    SizeIndependent(Box<dyn Fn(M::Set, S::Set) -> S::Set>),
}

impl<M, S> LazyPropagation<M, S>
where
    M: Monoid,
    S: Monoid,
{
    /// 区間幅を利用する場合
    pub fn new_size_dependent(f: impl Fn(M::Set, S::Set, usize) -> S::Set + 'static) -> Self {
        Self::SizeDependent(Box::new(f))
    }

    /// 区間幅を利用しない場合
    pub fn new_size_independent(f: impl Fn(M::Set, S::Set) -> S::Set + 'static) -> Self {
        Self::SizeIndependent(Box::new(f))
    }

    fn is_size_dependent(&self) -> bool {
        match self {
            LazyPropagation::SizeDependent(_) => true,
            LazyPropagation::SizeIndependent(_) => false,
        }
    }
}

pub struct LazySegtree<M, S>
where
    M: Monoid<Set: Copy>,
    S: Monoid<Set: Copy>,
{
    // 要素数が n のとき、配列のサイズは 2n とし、ルートにはアクセスしないことにする。
    // 配列は 1-origin とする。
    data: Box<[S::Set]>,
    maps: Box<[M::Set]>,
    // 完全二分木とは限らないので必要に応じてサイズを前計算する
    size: Option<Box<[usize]>>,

    lazy: LazyPropagation<M, S>,

    // 遅延伝搬・再計算で訪問する頂点を降順にメモする
    // TODO: [T; usize::BITS * 2] のラッパー定義し、Vec のように使いたい
    ancestors: Vec<usize>,
}

/// データセグメントを更新してから、遅延評価関数を合成する
//
// メソッドにするには借用規則の回避が面倒
macro_rules! push {
    ( $this:expr, $i:expr, $new_map:expr ) => {
        $this.data[$i] = match &$this.lazy {
            LazyPropagation::SizeDependent(f) => {
                let size = $this.size.as_ref().unwrap()[$i];
                f($new_map, $this.data[$i], size)
            }
            LazyPropagation::SizeIndependent(f) => f($new_map, $this.data[$i]),
        };

        if let Some(map) = $this.maps.get_mut($i) {
            // 時系列順に作用素を合成
            *map = M::op(*map, $new_map)
        }
    };
}

impl<M, S> LazySegtree<M, S>
where
    M: Monoid<Set: Copy>,
    S: Monoid<Set: Copy>,
{
    /// 適切にオフセットを追加する。
    #[inline]
    fn into_inner_range<R>(&self, range: R) -> Range<usize>
    where
        R: RangeBounds<usize>,
    {
        let offset = self.data.len() >> 1;

        // 最適化により、分岐は消える。インライン化してもよさそう。
        let l = match range.start_bound() {
            std::ops::Bound::Included(l) => l + offset,
            std::ops::Bound::Excluded(l) => l + offset + 1,
            std::ops::Bound::Unbounded => offset,
        };
        let r = match range.end_bound() {
            std::ops::Bound::Included(r) => r + offset + 1,
            std::ops::Bound::Excluded(r) => r + offset,
            std::ops::Bound::Unbounded => self.data.len(),
        };

        l..r
    }

    /// # Time Complexity
    ///
    /// *O*(log *N*)
    fn write_ancestors(&mut self, range: Range<usize>) {
        if range.is_empty() {
            return;
        }

        let Range {
            start: mut l,
            end: mut r,
        } = range;
        l >>= l.trailing_zeros() + 1;
        r >>= r.trailing_zeros();
        r = (r - 1) >> 1;
        while l != r {
            if l > r {
                self.ancestors.push(l);
                l >>= 1
            } else {
                self.ancestors.push(r);
                r >>= 1;
            }
        }
        while l > 0 {
            self.ancestors.push(l);
            l >>= 1
        }
    }

    /// 区間クエリに答える。区間が空なら単位元を返す。
    ///
    /// # Time Complexity
    ///
    /// *Θ*(log *N*)
    ///
    /// # Panics
    ///
    /// `..`は問題ないが、明示的に範囲外の端点を指定するとパニックする。
    pub fn range_query<R>(&mut self, range: R) -> S::Set
    where
        R: RangeBounds<usize>,
    {
        let range = self.into_inner_range(range);
        if range.is_empty() {
            return S::id();
        }
        self.write_ancestors(range.clone());

        // 作用素を必要な分だけ上から順に遅延評価する。
        for i in self.ancestors.drain(..).rev() {
            let map = std::mem::replace(&mut self.maps[i], M::id());

            push!(self, i << 1, map);
            push!(self, (i << 1) | 1, map);
        }

        // クエリの答えを計算する
        {
            let Range { start, end } = range;
            let [mut l, mut r] = [start >> start.trailing_zeros(), end >> end.trailing_zeros()];
            let [mut acc_l, mut acc_r] = [S::id(); 2];
            while {
                if l >= r {
                    acc_l = S::op(acc_l, self.data[l]);
                    l += 1;
                    l >>= l.trailing_zeros()
                } else {
                    r -= 1;
                    acc_r = S::op(self.data[r], acc_r);
                    r >>= r.trailing_zeros()
                }

                l != r
            } {}

            S::op(acc_l, acc_r)
        }
    }

    /// 区間内のデータを作用素で更新する。区間が空なら何もしない。
    ///
    /// # Time Complexity
    ///
    /// *Θ*(log *N*)
    ///
    /// # Panics
    ///
    /// `..`は問題ないが、明示的に範囲外の端点を指定するとパニックする。
    pub fn range_update<R>(&mut self, range: R, map: M::Set)
    where
        R: RangeBounds<usize>,
    {
        let range = self.into_inner_range(range);
        // 早期リターンしないと更新処理がバグる
        if range.is_empty() {
            return;
        }
        self.write_ancestors(range.clone());

        // 作用素を必要な分だけ上から順に遅延評価する。
        for i in self.ancestors.iter().copied().rev() {
            let map = std::mem::replace(&mut self.maps[i], M::id());

            push!(self, i << 1, map);
            push!(self, (i << 1) | 1, map);
        }

        // 必要な分だけ更新する
        {
            let Range { start, end } = range;
            let [mut l, mut r] = [start >> start.trailing_zeros(), end >> end.trailing_zeros()];

            while {
                if l >= r {
                    push!(self, l, map);
                    l += 1;
                    l >>= l.trailing_zeros()
                } else {
                    r -= 1;
                    push!(self, r, map);
                    r >>= r.trailing_zeros()
                }

                l != r
            } {}
        }

        // 必要な分だけ親ノードの値を再計算する
        for i in self.ancestors.drain(..) {
            self.data[i] = S::op(self.data[i << 1], self.data[(i << 1) | 1])
        }
    }

    /// # Time Complexity
    ///
    /// *O*(*N*)
    pub fn data(&mut self) -> &[S::Set] {
        // 遅延伝搬する
        for i in 0..self.data.len() >> 1 {
            let map = std::mem::replace(&mut self.maps[i], M::id());
            push!(self, i << 1, map);
            push!(self, (i << 1) | 1, map);
        }

        &self.data[self.data.len() >> 1..]
    }
}

impl<M, S> LazySegtree<M, S>
where
    M: Monoid<Set: Copy>,
    S: Monoid<Set: Copy>,
{
    /// # Time COmplexity
    ///
    /// *Θ*(*N*)
    pub fn new(value: Vec<S::Set>, lazy_propagation: LazyPropagation<M, S>) -> Self {
        let n = value.len();

        let data = {
            let mut data = Vec::with_capacity(n << 1);
            {
                let uninit = data.spare_capacity_mut();
                for (i, v) in (n..n << 1).zip(value) {
                    uninit[i].write(v);
                }
                for i in (1..n).rev() {
                    // SAFETY: 子孫は初期化済み
                    let v = S::op(unsafe { uninit[i << 1].assume_init() }, unsafe {
                        uninit[(i << 1) | 1].assume_init()
                    });
                    uninit[i].write(v);
                }
                uninit[0].write(S::id());
            }
            // SAFETY: [0, 2n) を初期化した
            unsafe { data.set_len(n << 1) };

            data.into_boxed_slice()
        };

        let size = lazy_propagation.is_size_dependent().then_some({
            let mut size = Vec::with_capacity(n << 1);
            let uninit = size.spare_capacity_mut();
            for i in n..n << 1 {
                uninit[i].write(1);
            }
            for i in (0..n).rev() {
                // SAFETY: 直前までに初期化している
                uninit[i].write(
                    unsafe { uninit[i << 1].assume_init() }
                        + unsafe { uninit[(i << 1) | 1].assume_init() },
                );
            }
            // SAFETY: [0, 2n) を初期化した
            unsafe { size.set_len(n << 1) };

            size.into_boxed_slice()
        });

        Self {
            data,
            maps: Vec::from_iter(std::iter::repeat_n(M::id(), n)).into_boxed_slice(),
            size,
            lazy: lazy_propagation,
            // capacity の制約上 n.next_power_of_two() はオーバーフローしない
            ancestors: Vec::with_capacity(n.next_power_of_two().ilog2() as usize * 2),
        }
    }
}

impl<M, S> Debug for LazySegtree<M, S>
where
    M: Monoid<Set: Copy + Debug>,
    S: Monoid<Set: Copy + Debug>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LazySegtree")
            .field("data", &self.data)
            .field("maps", &self.maps)
            .field("size", &self.size)
            .field("lazy", &"/* inaccessible /*")
            .field("ancestors", &self.ancestors)
            .finish()
    }
}
