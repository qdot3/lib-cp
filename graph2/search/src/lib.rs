use csr2::{Edge, EdgeType, CSR};

#[derive(Debug)]
pub struct Visitor<W, E>
where
    E: EdgeType,
{
    csr: CSR<W, E>,

    // Working buffer used as a stack (DFS) or queue (BFS).
    // Each entry is [node_index, next_edge_index_to_try].
    buf: Vec<[usize; 2]>,
    used_node: BitSet,
    used_edge: BitSet,
}

impl<W, E> Visitor<W, E>
where
    E: EdgeType,
{
    pub fn new(csr: CSR<W, E>) -> Self {
        // Reserve capacity up front based on the number of edges,
        // so `buf` does not need to reallocate during traversal.
        let buf = Vec::with_capacity(csr.num_edges());

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
        self.used_edge.clear();
        self.used_node.clear();
    }

    pub fn csr(&self) -> &CSR<W, E> {
        &self.csr
    }

    /// Give back ownership of the inner graph.
    pub fn into_csr(self) -> CSR<W, E> {
        self.csr
    }

    // pub fn replace_csr(mut self)

    /// Creates a lending iterator for a DFS traversal starting at `source`,
    /// visiting only unvisited nodes.
    pub fn dfs<'a>(&'a mut self, source: usize) -> DFS<'a, W, E> {
        if self.used_node.insert(source) {
            self.buf.push([source, 0]);
        }

        DFS { visitor: self }
    }

    /// Creates a lending iterator for a BFS traversal starting at `source`,
    /// visiting only unvisited nodes.
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

/// One step result produced while doing a DFS.
#[derive(Debug, Clone)]
pub enum DFSTraversal<W> {
    /// Went deeper: moved into a not-yet-visited node through an unused edge.
    Descend(Edge<W>),
    /// Went back up to the parent node, through the edge that was used
    /// to originally arrive at the current node.
    Ascend(Edge<W>),
    /// Looked at an already-visited node through an unused edge
    /// (edge gets marked as used), but stayed at the current node.
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

        // In an undirected graph, edge (u -> v) and its mirror (v -> u) represent the same physical edge.
        // If the mirror edge was already consumed from the other side, we must skip it here too,
        // otherwise we would traverse the same edge twice.
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
                // SAFETY: `source` is correct.
                unsafe { e.set_source(source) })
        {
            // There might be more edges from `source` after this one,
            // so push back the frame with the next edge index to try.
            buf.push([source, nth + 1]);

            if used_node.insert(e.target) {
                buf.push([e.target, 0]);
                return Some(DFSTraversal::Descend(e));
            } else {
                return Some(DFSTraversal::Glance(e));
            }
        } else {
            // No more edges left to try from `source`.
            let e = {
                let &[parent, nth] = buf.last()?;
                let e = csr
                    .nth_edge(parent, nth - 1)
                    .expect("this edge has already been used.");
                // SAFETY: `parent` is correct source node.
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

/// One step result produced while doing a BFS.
pub enum BFSTraversal<W> {
    /// Found a brand-new node through an unused edge.
    Discover(Edge<W>),
    /// Looked at an already-visited node through an unused edge.
    Glance(Edge<W>),
}

#[derive(Debug)]
pub struct BFS<'a, W, E>
where
    E: EdgeType,
{
    visitor: &'a mut Visitor<W, E>,
    // Index of the frame in `buf` currently being expanded.
    // Frames before `cursor` are already fully processed.
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

        // Skip used edges
        let e = loop {
            let [source, nth] = buf.get_mut(self.cursor)?;
            if let Some(e) = csr.nth_edge(*source, *nth) {
                *nth += 1;
                if used_edge.insert(e.index) {
                    // Found an edge that has not been used yet.
                    // SAFETY: `source` is correct.
                    break unsafe { e.set_source(*source) };
                }
                // Otherwise this edge was already used.
            } else {
                // No edges left in this frame, advance to the next node.
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

/// A simple fixed-size bit set, used to mark visited nodes / used edges.
#[derive(Debug, Clone)]
struct BitSet(Vec<usize>);

impl BitSet {
    /// Number of bits stored in one `usize` word.
    const B: usize = usize::BITS as usize;

    /// Create a bit set big enough to hold `n` bits (all zero/false).
    fn new(n: usize) -> Self {
        Self(vec![0; n.div_ceil(usize::BITS as usize)])
    }

    /// Reset all bits back to zero/false.
    fn clear(&mut self) {
        self.0.fill(0);
    }

    /// Set bit `i` to true. Returns `true` if it was false before
    /// (i.e. this is the first time `i` is inserted).
    fn insert(&mut self, i: usize) -> bool {
        let (b, i) = (i / Self::B, i % Self::B);
        let was_empty = (self.0[b] >> i) & 1 == 0;
        self.0[b] |= 1 << i;
        was_empty
    }

    /// Check whether bit `i` is set.
    fn contains(&self, i: usize) -> bool {
        let (b, i) = (i / Self::B, i % Self::B);

        (self.0[b] >> i) & 1 > 0
    }
}
