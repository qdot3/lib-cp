use std::ops::ControlFlow;

use csr2::{EdgeType, CSR};
use op_min::OpMin;
use search::Visitor;
use sparse_table::SparseTable;

pub struct LCA {
    time: Box<[usize]>,
    // (depth, index)
    rmq: SparseTable<(OpMin<usize>, OpMin<usize>)>,
}

impl LCA {
    /// # SAFETY
    ///
    /// `root`からすべての頂点に到達可能
    ///
    /// # Time complexity
    ///
    /// O(V log V)
    pub unsafe fn new<W, E>(csr: &CSR<W, E>, root: usize) -> Option<Self>
    where
        E: EdgeType,
    {
        let mut time = Vec::with_capacity(csr.num_nodes());
        let mut depth = Vec::with_capacity(csr.num_nodes() * 2);
        {
            let time = time.spare_capacity_mut();
            assert!(time.len() >= csr.num_nodes());
            let depth = depth.spare_capacity_mut();
            assert!(depth.len() >= csr.num_nodes() * 2);

            let mut t = 0;
            let mut d = 0;
            time[root * 2].write(t);
            depth[t].write((d, root));

            let mut visitor = Visitor::new(csr);
            let res = visitor.dfs(root, |e| {
                match e {
                    search::DFSTraversal::Descend(edge) => {
                        t += 1;
                        d += 1;
                        time[edge.target].write(t);
                        depth[t].write((d, edge.target));
                    }
                    search::DFSTraversal::Ascend(edge) => {
                        t += 1;
                        d -= 1;
                        depth[t].write((d, edge.source));
                    }
                    search::DFSTraversal::Glance(_) => {
                        // early return
                        return ControlFlow::Break(());
                    }
                }
                ControlFlow::Continue(())
            });
            if res.is_break() {
                return None;
            }
        }
        // SAFETY: Guaranteed by caller
        unsafe {
            time.set_len(csr.num_nodes());
            depth.set_len(csr.num_nodes() * 2);
        }

        let rmq = SparseTable::from(depth.into_boxed_slice());
        Some(Self {
            time: time.into_boxed_slice(),
            rmq,
        })
    }

    /// # Time complexity
    ///
    /// Θ(1)
    pub fn lca_pair(&self, x: usize, y: usize) -> Option<usize> {
        let mut l = self.time.get(x).copied()?;
        let mut r = self.time.get(y).copied()?;
        if l > r {
            std::mem::swap(&mut l, &mut r);
        }

        self.rmq.range_query(l..=r).map(|v| v.1)
    }

    /// # Time complexity
    ///
    /// Θ(`nodes.len()`)
    pub fn lca(&self, nodes: &[usize]) -> Option<usize> {
        let [mut l, mut r] = [usize::MAX, usize::MIN];
        for &i in nodes {
            let i = self.time.get(i).copied()?;
            l = l.min(i);
            r = r.max(i);
        }

        self.rmq.range_query(l..=r).map(|v| v.1)
    }
}
