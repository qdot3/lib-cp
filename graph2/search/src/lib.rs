use csr2::{Edge, OutEdge, CSR};

#[derive(Debug, Clone)]
pub enum Traverse<W> {
    /// 未使用の辺で未訪問の頂点に進む。
    Visit(Edge<W>),
    /// 使用済みの辺を逆進する。
    Leave(Edge<W>),
    /// 訪問済み頂点に至る未使用の辺。頂点を移動しない。
    Visited(Edge<W>),
}

#[derive(Debug)]
pub struct Visitor<'a, W, G> {
    graph: &'a CSR<W, G>,

    stack: Vec<usize>,
    visited: BitSet,
}

impl<'a, W, G> Visitor<'a, W, G> {
    pub fn new(graph: &'a CSR<W, G>) -> Self {
        let stack = Vec::with_capacity(graph.num_nodes() * 2);
        let visited = BitSet::new(graph.num_nodes());

        Self {
            graph,
            stack,
            visited,
        }
    }

    /// 訪問履歴を削除する
    pub fn reset(&mut self) {
        // FIXME: ビットセットはライブラリ化する
        self.visited.0.fill(0);
    }

    pub fn is_visited(&self, i: usize) -> bool {
        self.visited.get(i)
    }

    /// `source`からDFSする。`source`が訪問済みの場合は何もしない
    pub fn dfs(&'a mut self, source: usize) -> DFS<'a, W, G> {
        self.stack.clear();
        if !self.visited.get(source) {
            self.visited.set(source);
            self.stack.extend([source, 0]);
        }
        DFS(self)
    }
}

#[derive(Debug)]
pub struct DFS<'a, W, G>(&'a mut Visitor<'a, W, G>);

impl<'a, W, G> DFS<'a, W, G> {
    pub fn next(&mut self) -> Option<Traverse<&W>> {
        let Visitor {
            graph,
            stack,
            visited,
        } = self.0;

        let [source, nth] = stack.last_chunk_mut::<2>()?;

        // HACK: see <https://docs.rs/polonius-the-crab/latest/polonius_the_crab/index.html>
        if graph.nth_edge(*source, *nth).is_some() {
            let OutEdge { target, weight } = graph.nth_edge(*source, *nth).unwrap();
            *nth += 1;

            let e = Edge {
                source: *source,
                target,
                weight,
            };

            if visited.get(target) {
                return Some(Traverse::Visited(e));
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
                .nth_edge(parent, nth - 1)
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
