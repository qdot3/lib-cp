use std::ops::ControlFlow;

use csr2::{Edge, EdgeType, CSR};

#[derive(Debug)]
pub struct Visitor<'a, W, E>
where
    E: EdgeType,
{
    csr: &'a CSR<W, E>,

    /// Working buffer used as a stack (DFS) or queue (BFS).
    buf: Vec<[usize; 2]>,
    used_node: BitSet,
    used_edge: BitSet,
}

impl<'a, W, E> Visitor<'a, W, E>
where
    E: EdgeType,
{
    pub fn new(csr: &'a CSR<W, E>) -> Self {
        let buf = Vec::with_capacity(csr.num_nodes());

        Self {
            buf,
            used_node: BitSet::new(csr.num_nodes()),
            used_edge: BitSet::new(csr.num_edges()),
            csr,
        }
    }

    /// Check whether node `i` has already been visited.
    pub fn visited(&self, i: usize) -> bool {
        self.used_node.contains(i)
    }

    /// Clear all "visited" state, so this `Visitor` can be reused for a brand-new traversal.
    pub fn reset(&mut self) {
        self.buf.clear();
        self.used_edge.clear();
        self.used_node.clear();
    }

    /// `root`から未使用の辺で DFS をする。
    ///
    /// # Time complexity
    ///
    /// 未訪問の`root`のみが与えられる場合、グラフ全体で O(|V| + |E|)。
    pub fn dfs<B>(
        &mut self,
        root: usize,
        mut cursor: impl FnMut(DFSTraversal<&W>) -> ControlFlow<B>,
    ) -> ControlFlow<B> {
        let Visitor {
            ref csr,
            buf,
            used_node,
            used_edge,
        } = self;

        used_node.insert(root);
        buf.clear();
        buf.push([root, 0]);

        loop {
            let Some([source, mut nth]) = buf.pop() else {
                return ControlFlow::Continue(());
            };

            // 使用済みの辺は無視する
            while csr
                .nth_edge(source, nth)
                .is_some_and(|e| !used_edge.insert(e.index))
            {
                nth += 1;
            }

            // 未使用の辺があれば、それを使う
            if let Some(e) = csr.nth_edge(source, nth).map(|e|
                // SAFETY: `source` is correct.
                unsafe { e.set_source(source) })
            {
                // 次に試す辺をスタックのトップに置く
                buf.push([source, nth + 1]);

                if used_node.insert(e.target) {
                    buf.push([e.target, 0]);
                    cursor(DFSTraversal::Descend(e))?;
                } else {
                    cursor(DFSTraversal::Glance(e))?;
                }
            }
            // すべての辺を使用したので、木辺を昇る。
            else if let Some([parent, nth]) = buf.last().copied() {
                let e = csr
                    .nth_edge(parent, nth - 1)
                    .map(|e| {
                        // SAFETY: `parent` is correct.
                        unsafe { e.set_source(parent) }
                    })
                    .unwrap();

                cursor(DFSTraversal::Ascend(e))?;
            }
        }
    }

    /// `root`から未使用の辺で BFS をする。
    ///
    /// # Time complexity
    ///
    /// 未訪問の`root`のみが与えられる場合、グラフ全体で O(|V| + |E|)。
    pub fn bfs<B>(
        &mut self,
        root: usize,
        mut cursor: impl FnMut(BFSTraversal<&W>) -> ControlFlow<B>,
    ) -> ControlFlow<B> {
        let Visitor {
            ref csr,
            buf,
            used_node,
            used_edge,
        } = self;

        used_node.insert(root);
        buf.clear();
        buf.push([root, 0]);

        let mut i = 0;
        loop {
            let e = loop {
                let Some([source, nth]) = buf.get_mut(i) else {
                    return ControlFlow::Continue(());
                };
                // 未使用の辺を探す
                if let Some(e) = csr.nth_edge(*source, *nth) {
                    *nth += 1;
                    if used_edge.insert(e.index) {
                        // SAFETY: `source` is correct.
                        break unsafe { e.set_source(*source) };
                    }
                } else {
                    i += 1;
                }
            };

            if used_node.insert(e.target) {
                buf.push([e.target, 0]);
                cursor(BFSTraversal::Discover(e))?;
            } else {
                cursor(BFSTraversal::Glance(e))?;
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DFSTraversal<W> {
    /// 未使用の木辺を降る
    Descend(Edge<W>),
    /// 使用済みの木辺を昇る
    Ascend(Edge<W>),
    /// 訪問済み頂点に向かう未使用の辺を見るが、移動しない。
    Glance(Edge<W>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BFSTraversal<W> {
    /// 未使用の辺で未訪問の頂点に移動する
    Discover(Edge<W>),
    /// 未使用の辺で訪問済みの頂点を見るが、移動しない。
    Glance(Edge<W>),
}

#[derive(Debug, Clone)]
struct BitSet(Vec<usize>);

impl BitSet {
    const B: usize = usize::BITS as usize;

    fn new(n: usize) -> Self {
        Self(vec![0; n.div_ceil(usize::BITS as usize)])
    }

    fn clear(&mut self) {
        self.0.fill(0);
    }

    fn insert(&mut self, i: usize) -> bool {
        let (b, i) = (i / Self::B, i % Self::B);
        let was_empty = (self.0[b] >> i) & 1 == 0;
        self.0[b] |= 1 << i;
        was_empty
    }

    fn contains(&self, i: usize) -> bool {
        let (b, i) = (i / Self::B, i % Self::B);

        (self.0[b] >> i) & 1 > 0
    }
}
