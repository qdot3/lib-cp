use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeIndex(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Edge<W> {
    pub source: usize,
    pub target: usize,
    pub weight: W,
    pub index: usize,
}

impl<W> Edge<W> {
    pub fn discard_weight(self) -> Edge<()> {
        Edge {
            source: self.source,
            target: self.target,
            weight: (),
            index: self.index,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutEdge<W> {
    pub target: usize,
    pub weight: W,
    pub index: usize,
}

impl<W> OutEdge<W> {
    /// # SAFETY
    ///
    /// Giving an incorrect `source` results in UB.
    pub unsafe fn set_source(self, source: usize) -> Edge<W> {
        Edge {
            source,
            target: self.target,
            weight: self.weight,
            index: self.index,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Directed;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Undirected;

pub trait EdgeType {
    const DIRECTED: bool;
}

impl EdgeType for Directed {
    const DIRECTED: bool = true;
}

impl EdgeType for Undirected {
    const DIRECTED: bool = false;
}

#[derive(Debug, Clone)]
pub struct CSRBuilder<W, E>
where
    E: EdgeType,
{
    edges: Vec<Edge<W>>,
    num_node: usize,

    edge_type: PhantomData<E>,
}

impl<W> CSRBuilder<W, Directed> {
    #[must_use]
    pub fn with_capacity(num_node: usize, capacity: usize) -> Self {
        Self {
            edges: Vec::with_capacity(capacity),
            num_node,
            edge_type: PhantomData,
        }
    }

    /// Appends a directed edge.
    /// Returns `true` if `source` and `target` are valid.
    pub fn push_edge(&mut self, source: usize, target: usize, weight: W) -> bool {
        if source.max(target) < self.num_node {
            let index = self.edges.len();
            self.edges.push(Edge {
                source,
                target,
                weight,
                index,
            });

            true
        } else {
            false
        }
    }
}

impl<W> CSRBuilder<W, Undirected> {
    #[must_use]
    pub fn with_capacity(num_node: usize, capacity: usize) -> Self {
        Self {
            edges: Vec::with_capacity(capacity.saturating_mul(2)),
            num_node,
            edge_type: PhantomData,
        }
    }

    /// Appends an undirected edge.
    ///
    /// Node index must be compact.
    pub fn push_edge(&mut self, source: usize, target: usize, weight: W) -> bool
    where
        W: Clone,
    {
        if source.max(target) < self.num_node {
            let index = self.edges.len() / 2;
            self.edges.push(Edge {
                source,
                target,
                weight: weight.clone(),
                index,
            });
            self.edges.push(Edge {
                source: target,
                target: source,
                weight,
                index,
            });

            true
        } else {
            false
        }
    }
}

impl<W, E> CSRBuilder<W, E>
where
    E: EdgeType,
{
    /// # Time complexity
    ///
    /// O(N + M), where `N` is the max index of nodes and `M` is the number of edges.
    #[must_use]
    pub fn build(self) -> CSR<W, E> {
        let edges = self.edges;
        let n = self.num_node.checked_add(1).unwrap();

        let mut cnt = vec![0; n];
        edges.iter().for_each(|e| {
            cnt[e.source] += 1;
        });
        for i in 1..cnt.len() {
            cnt[i] += cnt[i - 1]
        }
        debug_assert_eq!(cnt[n - 1], edges.len());

        let n_edges = edges.len();
        let mut out_edges = Vec::with_capacity(edges.len());
        {
            let uninit = out_edges.spare_capacity_mut();
            assert!(uninit.len() >= edges.len(), "guard");

            for Edge {
                source,
                target,
                weight,
                index,
            } in edges
            {
                cnt[source] -= 1;
                uninit[cnt[source] as usize].write(OutEdge {
                    target,
                    weight,
                    index,
                });
            }
        }
        // SAFETY:
        // - `out_edges` has sufficient capacity, or this function would have already panicked.
        // - the first `n_edges` elements have been initialized.
        unsafe { out_edges.set_len(n_edges) };

        CSR {
            out_edges,
            partition: cnt,

            edge_type: PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CSR<W, E>
where
    E: EdgeType,
{
    out_edges: Vec<OutEdge<W>>,
    partition: Vec<usize>,

    edge_type: PhantomData<E>,
}

impl<W, E> CSR<W, E>
where
    E: EdgeType,
{
    /// # Panics
    ///
    /// Panics if `source` node does not exist.
    pub fn out_edges(&self, source: usize) -> &[OutEdge<W>] {
        &self.out_edges[self.partition[source]..self.partition[source + 1]]
    }

    /// # Panics
    ///
    /// Panics if `source` node does not exist.
    pub fn nth_edge(&self, source: usize, nth: usize) -> Option<OutEdge<&W>> {
        if let Some(e) = self.out_edges(source).get(nth) {
            Some(OutEdge {
                target: e.target,
                weight: &e.weight,
                index: e.index,
            })
        } else {
            None
        }
    }

    pub fn num_nodes(&self) -> usize {
        // `partition` has at least one element.
        self.partition.len() - 1
    }

    pub fn num_edges(&self) -> usize {
        if E::DIRECTED {
            self.out_edges.len()
        } else {
            self.out_edges.len() / 2
        }
    }
}

impl<W> CSR<W, Directed> {
    pub fn nth_edge_mut(&mut self, source: usize, nth: usize) -> Option<OutEdge<&mut W>> {
        let edges = &mut self.out_edges[self.partition[source]..self.partition[source + 1]];
        if let Some(e) = edges.get_mut(nth) {
            Some(OutEdge {
                target: e.target,
                weight: &mut e.weight,
                index: e.index,
            })
        } else {
            None
        }
    }
}

impl<W> CSR<W, Directed> {
    #[must_use]
    pub fn find_cycle(&self) -> Option<impl ExactSizeIterator<Item = Edge<&W>>> {
        struct Counter {
            cnt: Vec<usize>,
        }

        impl Counter {
            const B: usize = usize::BITS as usize / 2;

            fn new(n: usize) -> Self {
                Self {
                    cnt: vec![0; n.div_ceil(Self::B)],
                }
            }

            fn inc(&mut self, i: usize) {
                let (b, i) = (i / Self::B, i % Self::B);
                self.cnt[b] += 1 << i * 2;
            }

            fn get(&self, i: usize) -> usize {
                let (b, i) = (i / Self::B, i % Self::B);
                (self.cnt[b] >> i * 2) & 0b11
            }
        }

        let mut cnt = Counter::new(self.num_nodes());
        let mut stack = Vec::with_capacity(self.num_nodes());
        for i in 0..self.num_nodes() {
            if cnt.get(i) > 0 {
                continue;
            }

            cnt.inc(i);
            stack.push((i, 0));
            while let Some((src, nth)) = stack.pop() {
                if let Some(tar) = self.nth_edge(src, nth).map(|v| v.target) {
                    stack.push((src, nth + 1));

                    match cnt.get(tar) {
                        0 => {
                            cnt.inc(tar);
                            stack.push((tar, 0));
                        }
                        2 => (),
                        _cnt => {
                            assert_eq!(_cnt, 1);

                            let i = stack
                                .iter()
                                .position(|v| v.0 == tar)
                                .expect("loop is detected");

                            let iter = stack.into_iter().skip(i).map(|(source, nth)| {
                                let &OutEdge {
                                    target,
                                    weight,
                                    index,
                                } = &self.nth_edge(source, nth - 1).unwrap();
                                Edge {
                                    source,
                                    target,
                                    weight,
                                    index,
                                }
                            });
                            return Some(iter);
                        }
                    }
                } else {
                    cnt.inc(src);
                }
            }
        }

        None
    }
}
