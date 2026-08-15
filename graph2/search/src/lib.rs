use csr2::{Edge, OutEdge, CSR};

#[derive(Debug, Clone)]
pub enum Traverse<W> {
    /// 未訪問の頂点に進む未使用の辺。頂点を移動する。
    Visit(Edge<W>),
    /// 逆進する使用済みの辺。頂点を移動する。
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

    /// 訪問履歴を削除する。
    pub fn reset(&mut self) {
        // FIXME: ビットセットはライブラリ化する
        self.visited.0.fill(0);
    }

    /// 訪問済みなら`true`を返す。
    pub fn is_visited(&self, i: usize) -> bool {
        self.visited.get(i)
    }

    /// `source`から未訪問の頂点をDFSする。
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

#[derive(Debug)]
pub struct BFS<'a, W, G> {
    visitor: &'a mut Visitor<'a, W, G>,
    n: usize,
}

impl<'a, W, G> BFS<'a, W, G> {
    fn next(&mut self) -> Option<Traverse<&W>> {
        let Visitor {
            graph,
            stack,
            visited,
        } = self.visitor;

        let &[source, nth, parent] = stack.as_chunks::<3>().0.get(self.n)?;
        let OutEdge { target, weight } = graph.nth_edge(source, nth).unwrap();

        self.n += 1;

        // 初回訪問時のみ辺を追加する
        if visited.get(source) {
            stack.extend(
                graph
                    .out_edges(source)
                    .iter()
                    .enumerate()
                    .flat_map(|(i, e)| [e.target, i, source]),
            );
        }

        let e = Edge {
            source,
            target,
            weight,
        };
        Some(if !visited.get(target) {
            Traverse::Visit(e)
        } else if target == parent {
            todo!()
        } else {
            Traverse::Visited(e)
        })
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
