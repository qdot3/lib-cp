use std::marker::PhantomData;

use csr::{Index, UndirectedCSR, CSR};
use fixedbitset::FixedBitSet;
use ops::{marker::Commutative, Monoid};

#[derive(Debug, Clone)]
pub struct Mapping<Idx, W, M, FC, FP>
where
    Idx: Index,
    M: Monoid,
    FC: Fn(M::Set, &W) -> M::Set,
    FP: Fn(Idx, M::Set) -> M::Set,
{
    map_child: FC,
    map_parent: FP,
    phantom: PhantomData<(Idx, M, W)>,
}

impl<Idx, W, M, FC, FP> Mapping<Idx, W, M, FC, FP>
where
    Idx: Index,
    M: Monoid,
    FC: Fn(M::Set, &W) -> M::Set,
    FP: Fn(Idx, M::Set) -> M::Set,
{
    /// 下記のように部分木の結果を計算する
    ///
    /// 1. 子ノードまでの部分木の結果に`map_child`を作用させる
    /// 2. `M::op`で修正した結果をまとめる
    /// 3. まとめた結果に`map_parent`を作用させる
    pub fn new(map_child: FC, map_parent: FP) -> Self {
        Self {
            map_child,
            map_parent,
            phantom: PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TreeDP<Idx, W, M, FC, FP>
where
    Idx: Index,
    M: Monoid<Set: Copy> + Commutative,
    FC: Fn(M::Set, &W) -> M::Set,
    FP: Fn(Idx, M::Set) -> M::Set,
{
    csr: UndirectedCSR<Idx, W>,
    root: Idx,
    dp: Box<[M::Set]>,

    /// 累積和の前処理
    map: Mapping<Idx, W, M, FC, FP>,
}

impl<Idx, W, M, FC, FP> TreeDP<Idx, W, M, FC, FP>
where
    Idx: Index,
    M: Monoid<Set: Copy> + Commutative,
    FC: Fn(M::Set, &W) -> M::Set,
    FP: Fn(Idx, M::Set) -> M::Set,
{
    /// 全方位木 DP
    ///
    /// # Time Complexity
    ///
    /// *Θ*(*N*)
    pub fn rerooting(self) -> (UndirectedCSR<Idx, W>, Box<[M::Set]>) {
        let Self {
            csr,
            root,
            mut dp,
            map,
        } = self;

        // n 要素の dp を構築できているので、オーバーフローしない
        let n = csr.max_index() + 1;
        let mut dfs = Vec::with_capacity(n);
        dfs.push((root, root, dp[root.into_usize()]));
        let mut acc_l = Vec::with_capacity(n);
        let mut acc_r = acc_l.clone();
        // DFS 順序で再計算する。
        while let Some((
            src,
            par,
            // src を根としたときに、par を根とする部分木の結果
            mut sub_par,
        )) = dfs.pop()
        {
            // 根を付け替える。最後にもとに戻す。
            std::mem::swap(&mut dp[par.into_usize()], &mut sub_par);

            acc_l.push(M::id());
            for (tar, w) in csr.target(src) {
                let v = (map.map_child)(dp[tar.into_usize()], w);
                acc_l.push(M::op(acc_l.last().copied().unwrap(), v));
            }
            dp[src.into_usize()] = (map.map_parent)(src, acc_l.pop().unwrap());

            acc_r.push(M::id());
            for (tar, w) in csr.target(src).into_iter().rev() {
                let v = (map.map_child)(dp[tar.into_usize()], w);
                acc_r.push(M::op(acc_r.last().copied().unwrap(), v));
            }
            acc_r.pop();

            for ((&(tar, _), acc_l), acc_r) in csr
                .target(src)
                .iter()
                .zip(acc_l.drain(..))
                .zip(acc_r.drain(..).rev())
            {
                // tar を根としたときの、src を根とする部分木の結果
                if tar != par {
                    let v = (map.map_parent)(tar, M::op(acc_l, acc_r));
                    dfs.push((tar, src, v));
                }
            }
            // 根の値をもとに戻す
            dp[par.into_usize()] = sub_par
        }

        (csr, dp)
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum BuildTreeDpError {
    #[error("given root index is too large.")]
    InvalidRoot,
    #[error("given CSR is not a tree.")]
    NotTree,
}

impl<Idx, W, M, FC, FP> TryFrom<(Idx, UndirectedCSR<Idx, W>, Mapping<Idx, W, M, FC, FP>)>
    for TreeDP<Idx, W, M, FC, FP>
where
    Idx: Index,
    M: Monoid<Set: Copy> + Commutative,
    FC: Fn(M::Set, &W) -> M::Set,
    FP: Fn(Idx, M::Set) -> M::Set,
{
    type Error = BuildTreeDpError;

    /// # Time Complexity
    ///
    /// *Θ*(*N*)
    fn try_from(
        (root, csr, map): (Idx, UndirectedCSR<Idx, W>, Mapping<Idx, W, M, FC, FP>),
    ) -> Result<Self, Self::Error> {
        let n = csr.max_index() + 1;
        if root.into_usize() >= n {
            return Err(BuildTreeDpError::InvalidRoot);
        }
        // |E| = |V| - 1（無向辺）より連結であることと木であることは同値
        if csr.max_index() * 2 != csr.num_edges() {
            return Err(BuildTreeDpError::NotTree);
        }

        let mut dp = Vec::with_capacity(n);
        let uninit_dp = dp.spare_capacity_mut();

        let mut dfs = Vec::with_capacity(n);
        dfs.push(root);
        let mut dfs_cnt = 0;
        // 前半は行きがけ、後半は帰りがけ。帰りがけに remove すする戦略はループがあると破綻する。
        let mut visited = FixedBitSet::with_capacity(2 * n);
        while let Some(src) = dfs.last().copied() {
            if !visited[src.into_usize()] {
                // 行きがけ
                visited.insert(src.into_usize());
                dfs_cnt += 1;

                for &(tar, _) in csr.target(src) {
                    if !visited[tar.into_usize()] {
                        dfs.push(tar);
                    }
                }
            } else {
                // 帰りがけ
                visited.insert(src.into_usize() + n);
                dfs.pop();

                let mut acc = M::id();
                for (tar, w) in csr.target(src) {
                    if visited[tar.into_usize() + n] {
                        acc = M::op(
                            acc,
                            // SAFETY: 帰りがけなので、子孫は初期化済み
                            (map.map_child)(
                                unsafe { uninit_dp[tar.into_usize()].assume_init() },
                                w,
                            ),
                        );
                    }
                }
                uninit_dp[src.into_usize()].write((map.map_parent)(src, acc));
            }
        }
        // |E| = |V| - 1（無向辺）より連結であることと木であることは同値
        if dfs_cnt != n {
            return Err(BuildTreeDpError::NotTree);
        }
        // SAFETY
        // n 要素が初期化されたため。容量不足なら uninit_dp の範囲外アクセスでパニックしている。
        unsafe { dp.set_len(n) };

        Ok(Self {
            csr,
            root,
            dp: dp.into_boxed_slice(),
            map,
        })
    }
}
