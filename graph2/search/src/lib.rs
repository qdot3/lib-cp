use csr2::{Edge, EdgeType, CSR};

#[derive(Debug)]
pub struct Visitor<W, E>
where
    E: EdgeType,
{
    csr: CSR<W, E>,

    buf: Vec<[usize; 2]>,
    used_node: BitSet,
    used_edge: BitSet,
}

impl<W, E> Visitor<W, E>
where
    E: EdgeType,
{
    pub fn new(csr: CSR<W, E>) -> Self {
        let buf = Vec::with_capacity(csr.num_edges());

        Self {
            buf,
            used_node: BitSet::new(csr.num_nodes()),
            used_edge: BitSet::new(csr.num_edges()),
            csr,
        }
    }

    pub fn visited(&self, i: usize) -> bool {
        self.used_node.contains(i)
    }

    pub fn reset(&mut self) {
        self.used_edge.clear();
        self.used_node.clear();
    }

    /// `source`から未訪問の頂点をDFSする。
    pub fn dfs<'a>(&'a mut self, source: usize) -> DFS<'a, W, E> {
        if self.used_node.insert(source) {
            self.buf.push([source, 0]);
        }

        DFS { visitor: self }
    }

    /// `source`から未訪問の頂点をBFSする。
    pub fn bfs<'a>(&'a mut self, source: usize) -> BFS<'a, W, E> {
        if self.used_node.insert(source) {
            self.buf.push([source, 0]);
        }
        BFS {
            visitor: self,
            cursor: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DFSTraversal<W> {
    /// Descend into an unvisited node through an unused edge.
    Descend(Edge<W>),
    /// Ascend to the parent through the edge used to arrive here.
    Ascend(Edge<W>),
    /// Glance an visited node through an unused edge, which will be marked as used.
    /// Stay in the current node.
    Glance(Edge<W>),
}

#[derive(Debug)]
pub struct DFS<'a, W, E>
where
    E: EdgeType,
{
    visitor: &'a mut Visitor<W, E>,
}

impl<'a, W, E> DFS<'a, W, E>
where
    E: EdgeType,
{
    pub fn next(&mut self) -> Option<DFSTraversal<&W>> {
        let Visitor {
            csr,
            buf,
            used_node,
            used_edge,
        } = self.visitor;

        if !E::DIRECTED {
            let [source, mut nth] = buf.pop()?;
            // Skip used edges
            while csr
                .nth_edge(source, nth)
                .is_some_and(|e| !used_edge.insert(e.index))
            {
                nth += 1;
            }
            buf.push([source, nth]);
        }

        let [source, nth] = buf.pop()?;
        if let Some(e) = csr.nth_edge(source, nth).map(|e|
                // SAFETY: Giving correct source
                unsafe { e.set_source(source) })
        {
            buf.push([source, nth + 1]);

            if used_node.insert(e.target) {
                buf.push([e.target, 0]);
                return Some(DFSTraversal::Descend(e));
            } else {
                return Some(DFSTraversal::Glance(e));
            }
        } else {
            let e = {
                let &[parent, nth] = buf.last()?;
                let e = csr
                    .nth_edge(parent, nth - 1)
                    .expect("this edge has already been used.");
                // SAFETY: Giving correct source
                unsafe { e.set_source(parent) }
            };

            return Some(DFSTraversal::Ascend(e));
        }
    }
}

impl<'a, W, E> Drop for DFS<'a, W, E>
where
    E: EdgeType,
{
    fn drop(&mut self) {
        self.visitor.buf.clear();
    }
}

pub enum BFSTraversal<W> {
    /// Discover a new vertex through an unused edge.
    Discover(Edge<W>),
    /// Glance an visited node through an unused edge.
    Glance(Edge<W>),
}

#[derive(Debug)]
pub struct BFS<'a, W, E>
where
    E: EdgeType,
{
    visitor: &'a mut Visitor<W, E>,
    cursor: usize,
}

impl<'a, W, E> BFS<'a, W, E>
where
    E: EdgeType,
{
    pub fn next(&mut self) -> Option<BFSTraversal<&W>> {
        let Visitor {
            csr,
            buf,
            used_node,
            used_edge,
        } = self.visitor;

        // Find next unused edge.
        let e = loop {
            let [source, nth] = buf.get_mut(self.cursor)?;
            if let Some(e) = csr.nth_edge(*source, *nth) {
                *nth += 1;
                if used_edge.insert(e.index) {
                    // SAFETY: Giving correct `source`
                    break unsafe { e.set_source(*source) };
                }
            } else {
                self.cursor += 1;
            }
        };

        if used_node.insert(e.target) {
            buf.push([e.target, 0]);
            Some(BFSTraversal::Discover(e))
        } else {
            Some(BFSTraversal::Glance(e))
        }
    }
}

impl<'a, W, E> Drop for BFS<'a, W, E>
where
    E: EdgeType,
{
    fn drop(&mut self) {
        self.visitor.buf.clear();
    }
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
