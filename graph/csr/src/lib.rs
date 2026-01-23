use std::fmt::Debug;

/// `CSR`構築時に情報落ちがないことが保証されたインデックス
pub trait Index: Debug + Copy + Eq + Ord {
    fn into_usize(self) -> usize;
    fn from_usize(n: usize) -> Self;
    fn assert_bounds(max: usize);
}

macro_rules! index_impl {
    ($( $t:ty )+) => {$(
        impl Index for $t {
            #[inline]
            fn into_usize(self) -> usize {
                self as usize
            }

            #[inline]
            fn from_usize(n: usize) -> Self {
                n as $t
            }

            fn assert_bounds(max_index: usize) {
                if <$t>::BITS <= usize::BITS {
                    // max_index + 1 がノード数を与えたり、フラグに利用したりする
                    assert!(<$t>::MAX as usize > max_index, "You should use a larger unsigned primitive integer.")
                } else {
                    // メモリ効率が悪いか、バグのどちらか
                    panic!("You should use a smaller unsigned primitive integer.")
                }
            }
        }
    )+};
}
// メモリの節約が目的なので、u128 は不要
index_impl!( u8 u16 u32 usize );

/// 有向辺
pub struct Edge<Idx: Index, W> {
    pub source: Idx,
    pub target: Idx,
    pub weight: W,
}

impl<Idx: Index> From<(Idx, Idx)> for Edge<Idx, ()> {
    fn from((source, target): (Idx, Idx)) -> Self {
        Self {
            source,
            target,
            weight: (),
        }
    }
}

impl<Idx: Index, W> From<(Idx, Idx, W)> for Edge<Idx, W> {
    fn from((source, target, weight): (Idx, Idx, W)) -> Self {
        Self {
            source,
            target,
            weight,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Directed;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Undirected;

#[derive(Debug, Clone)]
pub struct UndirectedCSR<Idx: Index, W> {
    target: Box<[(Idx, W)]>,
    partition: Box<[usize]>,
}

impl<Idx, W> UndirectedCSR<Idx, W>
where
    Idx: Index,
    W: Clone,
{
    /// 無向辺からグラフをつくる。
    ///
    /// # Time Complexity
    ///
    /// *Θ*(*N*)
    pub fn new<I>(edges: I, max_index: Idx) -> Self
    where
        I: IntoIterator<Item: Into<Edge<Idx, W>>>,
    {
        let edges: Vec<Edge<Idx, W>> = edges.into_iter().map(|e| e.into()).collect();
        let mut degree = vec![0; max_index.into_usize() + 2];
        for e in edges.iter() {
            degree[e.source.into_usize()] += 1;
            degree[e.target.into_usize()] += 1;
        }

        for i in 0..=max_index.into_usize() {
            degree[i + 1] += degree[i]
        }
        let mut partition_end = degree;

        let mut target = Vec::with_capacity(partition_end.last().copied().unwrap());
        {
            assert_eq!(target.len(), 0);
            let target = target.spare_capacity_mut();
            for e in edges {
                let (is, it) = (e.source.into_usize(), e.target.into_usize());
                partition_end[is] -= 1;
                target[partition_end[is]].write((e.target, e.weight.clone()));
                partition_end[it] -= 1;
                target[partition_end[it]].write((e.source, e.weight));
            }
        }
        // SAFETY
        //
        // `edges`の長さを n とおくと、`partition_end`の末尾の要素は 2n である。
        // for ループで異なる場所に 2n 個の要素を書き込むが、その最大のインデックスは 2n-1 である。
        // これは`target`の先頭の 2n 要素が初期化されたことを意味する。
        unsafe { target.set_len(partition_end.last().copied().unwrap()) };

        Self {
            target: target.into_boxed_slice(),
            partition: partition_end.into_boxed_slice(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DirectedCSR<Idx: Index, W> {
    target: Box<[(Idx, W)]>,
    partition: Box<[usize]>,
}

impl<Idx, W> DirectedCSR<Idx, W>
where
    Idx: Index,
{
    /// 有向辺からグラフをつくる。
    ///
    /// # Time Complexity
    ///
    /// *Θ*(*N*)
    pub fn new<I>(edges: I, max_index: Idx) -> Self
    where
        I: IntoIterator<Item: Into<Edge<Idx, W>>>,
    {
        let edges: Vec<Edge<Idx, W>> = edges.into_iter().map(|e| e.into()).collect();
        let mut degree = vec![0; max_index.into_usize() + 2];
        for e in edges.iter() {
            degree[e.source.into_usize()] += 1;
        }

        for i in 0..=max_index.into_usize() {
            degree[i + 1] += degree[i]
        }
        let mut partition_end = degree;

        let mut target = Vec::with_capacity(partition_end.last().copied().unwrap());
        {
            assert_eq!(target.len(), 0);
            let target = target.spare_capacity_mut();
            for e in edges {
                let is = e.source.into_usize();
                partition_end[is] -= 1;
                target[partition_end[is]].write((e.target, e.weight));
            }
        }
        // SAFETY
        //
        // `edges`の長さを n とおくと、`partition_end`の末尾の要素は n である。
        // for ループで異なる場所に n 個の要素を書き込むが、その最大のインデックスは n-1 である。
        // これは`target`の先頭の n 要素が初期化されたことを意味する。
        unsafe { target.set_len(partition_end.last().copied().unwrap()) };

        Self {
            target: target.into_boxed_slice(),
            partition: partition_end.into_boxed_slice(),
        }
    }
}

pub trait CSR<Idx: Index, W> {
    fn target(&self, source: Idx) -> &[(Idx, W)];
    fn max_index(&self) -> usize;
    fn num_edges(&self) -> usize;
}

impl<Idx: Index, W> CSR<Idx, W> for UndirectedCSR<Idx, W> {
    fn target(&self, source: Idx) -> &[(Idx, W)] {
        let i = source.into_usize();
        &self.target[self.partition[i]..self.partition[i + 1]]
    }

    fn max_index(&self) -> usize {
        self.partition.len() - 2
    }

    fn num_edges(&self) -> usize {
        self.target.len()
    }
}

impl<Idx: Index, W> CSR<Idx, W> for DirectedCSR<Idx, W> {
    fn target(&self, source: Idx) -> &[(Idx, W)] {
        let i = source.into_usize();
        &self.target[self.partition[i]..self.partition[i + 1]]
    }

    fn max_index(&self) -> usize {
        self.partition.len() - 2
    }

    fn num_edges(&self) -> usize {
        self.target.len()
    }
}
