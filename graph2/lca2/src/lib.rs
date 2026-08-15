use csr2::CSR;
use search::Visitor;

pub struct LCA {
    in_out: Vec<usize>,
}

impl LCA {
    /// # Preconditions
    ///
    /// - `graph` must be a tree
    /// - If `graph` is directed, `root` must be a root of the `graph`
    pub fn new<W, G>(graph: &CSR<W, G>, root: usize) -> Self {
        let mut visitor = Visitor::new(graph);

        let mut dfs = visitor.dfs(root);
        while let Some(t) = dfs.next() {
            match t {
                search::Traverse::Visit(_) => todo!(),
                search::Traverse::Leave(_) => todo!(),
                search::Traverse::Visited(_) => (),
            }
        }
        todo!()
    }

    pub fn lcs_pair(&self, x: usize, y: usize) -> usize {
        let mut l = self.in_out[x * 2];
        let mut r = self.in_out[y * 2];
        if l > r {
            std::mem::swap(&mut l, &mut r);
        }

        todo!()
    }

    pub fn lcs(&self, nodes: &[usize]) -> Option<usize> {
        let [mut l, mut r] = [usize::MAX, usize::MIN];
        for &i in nodes {
            l = l.min(self.in_out[i * 2]);
            r = r.max(self.in_out[i * 2]);
        }

        if l <= r {
            todo!()
        } else {
            None
        }
    }
}
