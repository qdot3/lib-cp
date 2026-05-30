use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Edge<W> {
    pub source: usize,
    pub target: usize,
    pub weight: W,
}

impl<W> Edge<W> {
    pub fn reverse(mut self) -> Self {
        std::mem::swap(&mut self.source, &mut self.target);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutEdge<W> {
    pub target: usize,
    pub weight: W,
}

#[derive(Debug)]
pub struct Directed;

#[derive(Debug)]
pub struct Undirected;

#[derive(Debug, Clone)]
pub struct CSRBuilder<W, G> {
    edges: Vec<Edge<W>>,
    num_node: usize,

    graph_ty: PhantomData<G>,
}

impl<W> CSRBuilder<W, Directed> {
    #[must_use]
    pub fn with_capacity(capacity: usize, num_node: usize) -> Self {
        Self {
            edges: Vec::with_capacity(capacity),
            num_node,
            graph_ty: PhantomData,
        }
    }

    /// Appends a directed edge.
    ///
    /// # Panics
    ///
    /// Node index must be compact.
    #[inline(always)]
    pub fn push_edge(&mut self, edge: Edge<W>) {
        assert!(
            edge.source.max(edge.target) < self.num_node,
            "Node index must be compact"
        );
        self.edges.push(edge);
    }
}

impl<W> CSRBuilder<W, Undirected> {
    #[must_use]
    pub fn with_capacity(capacity: usize, num_node: usize) -> Self {
        Self {
            edges: Vec::with_capacity(capacity.saturating_mul(2)),
            num_node,
            graph_ty: PhantomData,
        }
    }

    /// Appends an undirected edge.
    ///
    /// Node index must be compact.
    #[inline(always)]
    pub fn push_edge(&mut self, edge: Edge<W>)
    where
        W: Clone,
    {
        assert!(
            edge.source.max(edge.target) < self.num_node,
            "Node index must be compact"
        );
        self.edges.push(edge.clone());
        self.edges.push(edge.reverse());
    }
}

impl<W, G> CSRBuilder<W, G> {
    /// # Time complexity
    ///
    /// O(N + M), where `N` is the max index of nodes and `M` is the number of edges.
    #[must_use]
    pub fn build(self) -> CSR<W, G> {
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
        let mut target = Vec::with_capacity(edges.len());
        {
            let uninit = target.spare_capacity_mut();
            assert!(uninit.len() >= edges.len(), "guard");

            for Edge {
                source,
                target,
                weight,
            } in edges
            {
                cnt[source] -= 1;
                uninit[cnt[source] as usize].write(OutEdge { target, weight });
            }
        }
        // SAFETY:
        // - `target` has sufficient capacity, or this function would have already panicked.
        // - the first `n_edges` elements have been initialized.
        unsafe { target.set_len(n_edges) };

        CSR {
            target,
            partition: cnt,

            graph_ty: PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CSR<W, G> {
    target: Vec<OutEdge<W>>,
    partition: Vec<usize>,

    graph_ty: PhantomData<G>,
}

impl<W, G> CSR<W, G> {
    pub fn out_edges(&self, source: usize) -> &[OutEdge<W>] {
        &self.target[self.partition[source]..self.partition[source + 1]]
    }

    pub fn nth_edge(&self, source: usize, nth: usize) -> Option<OutEdge<&W>> {
        if let Some(e) = self.out_edges(source).get(nth) {
            Some(OutEdge {
                target: e.target,
                weight: &e.weight,
            })
        } else {
            None
        }
    }

    pub fn nth_edge_mut(&mut self, source: usize, nth: usize) -> Option<OutEdge<&mut W>> {
        let edges = &mut self.target[self.partition[source]..self.partition[source + 1]];
        if let Some(e) = edges.get_mut(nth) {
            Some(OutEdge {
                target: e.target,
                weight: &mut e.weight,
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
        self.target.len()
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
                                let &OutEdge { target, weight } =
                                    &self.nth_edge(source, nth - 1).unwrap();
                                Edge {
                                    source,
                                    target,
                                    weight,
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

pub mod scc {
    use super::*;

    pub struct SCC {
        scc: Vec<usize>,
        partition: Vec<usize>,
    }

    impl SCC {
        #[must_use]
        pub fn new<W>(csr: &CSR<W, Directed>) -> Self {
            struct BitSet {
                block: Vec<usize>,
            }

            impl BitSet {
                const B: usize = usize::BITS as usize;

                fn new(n: usize) -> Self {
                    Self {
                        block: vec![0; n.div_ceil(Self::B)],
                    }
                }

                fn is_false(&self, i: usize) -> bool {
                    let (b, i) = (i / Self::B, i % Self::B);
                    (self.block[b] >> i) & 1 == 0
                }

                fn set(&mut self, i: usize) {
                    let (b, i) = (i / Self::B, i % Self::B);
                    self.block[b] |= 1 << i
                }
            }

            const INF: usize = !0;

            let mut stack = Vec::with_capacity(csr.num_nodes());
            let mut ord_low = vec![(INF, INF); csr.num_nodes()];
            let mut removed = BitSet::new(csr.num_nodes());

            let mut scc = Vec::with_capacity(csr.num_nodes());
            let mut partition = Vec::with_capacity(csr.num_nodes());

            let mut order = 0;
            let mut cursor = csr.num_nodes();
            for i in 0..csr.num_nodes() {
                if ord_low[i].0 != INF {
                    continue;
                }

                stack.push((i, i, 0));
                ord_low[i] = (order, order);
                order += 1;
                scc.push(i);

                while let Some((v, p, nth)) = stack.pop() {
                    if let Some(OutEdge { target: c, .. }) = csr.nth_edge(v, nth) {
                        stack.push((v, p, nth + 1));

                        if ord_low[c].0 == INF {
                            ord_low[c] = (order, order);
                            order += 1;
                            scc.push(c);

                            stack.push((c, v, 0));
                        } else if removed.is_false(c) {
                            ord_low[v].1 = ord_low[v].1.min(ord_low[c].0)
                        }
                    } else {
                        ord_low[p].1 = ord_low[p].1.min(ord_low[v].1);

                        if removed.is_false(v) && {
                            let (ord, low) = ord_low[v];
                            ord == low
                        } {
                            // `v` is the representative node of the scc

                            partition.push(cursor);
                            while let Some(u) = scc.pop() {
                                cursor -= 1;
                                removed.set(u);

                                let i = cursor - scc.len();
                                scc.spare_capacity_mut()[i].write(u);

                                if u == v {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            assert_eq!(cursor, 0);

            // SAFETY: all elements have been initialized
            unsafe { scc.set_len(csr.num_nodes()) };
            partition.push(0);

            Self { scc, partition }
        }

        #[must_use]
        pub const fn len(&self) -> usize {
            self.partition.len() - 1
        }

        #[must_use]
        pub fn topological_iter(&self) -> impl DoubleEndedIterator<Item = &[usize]> {
            self.partition
                .windows(2)
                .rev()
                .map(|w| &self.scc[w[1]..w[0]])
        }
    }
}

pub mod search {
    use super::*;
    struct BitSet(Vec<usize>);

    impl BitSet {
        const B: usize = usize::BITS as usize;

        fn new(n: usize) -> Self {
            Self(vec![0; n.div_ceil(usize::BITS as usize)])
        }

        fn set(&mut self, i: usize) {
            let (b, i) = (i / Self::B, i % Self::B);
            self.0[b] |= (1 as usize) << i;
        }

        fn get(&self, i: usize) -> bool {
            let (b, i) = (i / Self::B, i % Self::B);

            (self.0[b] >> i) & 1 > 0
        }
    }

    #[derive(Debug, Clone)]
    pub enum Traverse<W> {
        Visit(Edge<W>),
        Leave(Edge<W>),
        Revisit(Edge<W>),
    }

    pub struct DFS<W, G> {
        pub graph: CSR<W, G>,

        stack: Vec<usize>,
        visited: BitSet,
    }

    impl<W> DFS<W, Directed> {
        pub fn new(graph: CSR<W, Directed>) -> Self {
            let stack = Vec::with_capacity(graph.num_nodes() * 2);
            let visited = BitSet::new(graph.num_nodes());

            DFS {
                graph,
                stack,
                visited,
            }
        }

        pub fn set_source(&mut self, source: usize) {
            self.stack.clear();
            self.stack.extend([source, 0]);

            self.visited.set(source);
        }

        pub fn is_visited(&self, i: usize) -> bool {
            self.visited.get(i)
        }

        pub fn next(&mut self) -> Option<Traverse<&mut W>> {
            let Self {
                graph,
                stack,
                visited,
            } = self;

            let [source, nth] = stack.last_chunk_mut::<2>()?;

            // hack the borrow checker. see <https://docs.rs/polonius-the-crab/latest/polonius_the_crab/index.html>
            if graph.nth_edge_mut(*source, *nth).is_some() {
                let OutEdge { target, weight } = graph.nth_edge_mut(*source, *nth).unwrap();
                *nth += 1;

                let e = Edge {
                    source: *source,
                    target,
                    weight,
                };

                if visited.get(target) {
                    return Some(Traverse::Revisit(e));
                } else {
                    visited.set(*source);
                    stack.extend([target, 0]);
                    return Some(Traverse::Visit(e));
                }
            } else {
                stack.pop();
                stack.pop();

                let &[parent, nth] = stack.last_chunk::<2>()?;
                let OutEdge { target, weight } = graph
                    .nth_edge_mut(parent, nth - 1)
                    .expect("this edge has already been passed.");

                let e = Edge {
                    source: parent,
                    target,
                    weight,
                };

                return Some(Traverse::Leave(e));
            }
        }
    }
}
