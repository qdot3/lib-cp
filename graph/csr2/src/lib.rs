use std::{fmt::Debug, marker::PhantomData, ops::Add};

pub trait Index: Debug + Copy + Add<Output = Self> + Ord {
    fn from_usize(i: usize) -> Self;
    fn into_usize(self) -> usize;

    fn inc(&mut self);
    fn dec(&mut self);
}

pub struct Edge<Idx, W> {
    pub source: Idx,
    pub target: Idx,
    pub weight: W,
}

impl<Idx> From<(Idx, Idx)> for Edge<Idx, ()> {
    fn from((source, target): (Idx, Idx)) -> Self {
        Self {
            source,
            target,
            weight: (),
        }
    }
}

impl<Idx, W> From<(Idx, Idx, W)> for Edge<Idx, W> {
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

pub trait EdgeType {
    const IS_DIRECTED: bool;
}

impl EdgeType for Directed {
    const IS_DIRECTED: bool = true;
}

impl EdgeType for Undirected {
    const IS_DIRECTED: bool = false;
}

pub struct CSR<Idx: Index, W, E: EdgeType> {
    target: Box<[(Idx, W)]>,
    partition: Box<[Idx]>,
    edge_ty: PhantomData<E>,
}

impl<Idx: Index, W, E: EdgeType> CSR<Idx, W, E> {
    pub fn target(&self, source: Idx) -> &[(Idx, W)] {
        let i = source.into_usize();
        &self.target[self.partition[i].into_usize()..self.partition[i + 1].into_usize()]
    }

    pub fn max_index(&self) -> usize {
        self.partition.len() - 2
    }

    pub fn num_edges(&self) -> usize {
        self.target.len()
    }
}

impl<Idx: Index, W, E: EdgeType> CSR<Idx, W, E>
where
    W: Clone,
{
    /// 座標圧縮すること。
    ///
    /// # Time Complexity
    ///
    /// - *Θ*(*N*) in time
    pub fn new<I>(edges: I, max_index: Idx) -> Self
    where
        I: IntoIterator<Item: Into<Edge<Idx, W>>>,
    {
        let edges: Vec<Edge<Idx, W>> = edges.into_iter().map(|e| e.into()).collect();

        let mut degree = vec![Idx::from_usize(0); max_index.into_usize() + 2];
        for e in edges.iter() {
            degree[e.source.into_usize()].inc();
            if !E::IS_DIRECTED {
                degree[e.target.into_usize()].inc();
            }
        }

        for i in 0..=max_index.into_usize() {
            degree[i + 1] = degree[i + 1] + degree[i]
        }
        let mut partition_end = degree;

        let mut target = Vec::with_capacity(partition_end.last().copied().unwrap().into_usize());
        {
            assert_eq!(target.len(), 0);
            let target = target.spare_capacity_mut();
            for e in edges {
                let (is, it) = (e.source.into_usize(), e.target.into_usize());

                if !E::IS_DIRECTED {
                    partition_end[it].dec();
                    target[partition_end[it].into_usize()].write((e.source, e.weight.clone()));
                }

                partition_end[is].dec();
                target[partition_end[is].into_usize()].write((e.target, e.weight));
            }
        }
        // SAFETY
        //
        // `edges`の長さを n とおくと、`partition_end`の末尾の要素は n[2n] である。
        // for ループで異なる場所に n[2n] 個の要素を書き込むが、その最大のインデックスは n[2n]-1 である。
        // これは`target`の先頭の n[2n] 要素が初期化されたことを意味する。
        unsafe { target.set_len(partition_end.last().copied().unwrap().into_usize()) };

        Self {
            target: target.into_boxed_slice(),
            partition: partition_end.into_boxed_slice(),
            edge_ty: PhantomData,
        }
    }
}

pub struct TreeCSR {}
