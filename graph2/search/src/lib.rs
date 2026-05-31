use csr2::{Directed, Edge, OutEdge, CSR};

#[derive(Debug, Clone)]
pub enum Traverse<W> {
    Visit(Edge<W>),
    Leave(Edge<W>),
    Revisit(Edge<W>),
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
