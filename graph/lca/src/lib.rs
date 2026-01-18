use csr::{Index, UndirectedCSR, CSR};

#[derive(Debug, Clone)]
pub struct LCA<Idx: Index> {
    /// `row * col`サイズの表。i 行目は 2^i 個上の祖先を表す。
    ancestor: Box<[Idx]>,
    row: usize,
    col: usize,

    depth: Box<[Idx]>,
    dfs_order: Box<[Idx]>,
}

impl<Idx: Index> LCA<Idx> {
    /// # Time Complexity
    ///
    /// *O*(log *N*)
    pub fn lca(&self, i: Idx, j: Idx) -> Idx {
        let [mut i, mut j] = [i.into_usize(), j.into_usize()];

        // i と j の深さをそろえる
        {
            if self.depth[i] < self.depth[j] {
                std::mem::swap(&mut i, &mut j);
            }
            let mut diff = self.depth[i].into_usize() - self.depth[j].into_usize();
            while diff > 0 {
                let lz = diff.trailing_zeros();
                diff ^= 1 << lz;
                i = self.ancestor[lz as usize * self.row + i].into_usize();
            }
        }

        if i == j {
            return Idx::from_usize(i);
        }

        for k in (0..self.col).rev() {
            if self.ancestor[k * self.row + i] != self.ancestor[k * self.row + j] {
                i = self.ancestor[k * self.row + i].into_usize();
                j = self.ancestor[k * self.row + j].into_usize();
            }
        }

        return self.ancestor[i];
    }

    /// # Time Complexity
    ///
    /// *O*(*D* log *N* + *D* log *D*)
    pub fn distance(&self, nodes: &mut [Idx]) -> usize {
        let mut sum = 0;
        nodes.sort_unstable_by_key(|i| self.dfs_order[i.into_usize()]);
        for pair in nodes.windows(2) {
            let lca = self.lca(pair[0], pair[1]).into_usize();

            sum += self.depth[pair[0].into_usize()].into_usize()
                + self.depth[pair[1].into_usize()].into_usize()
                - 2 * self.depth[lca].into_usize();
        }
        {
            let [i, j] = [nodes[0], nodes[nodes.len() - 1]];
            let lca = self.lca(i, j);
            sum += self.depth[i.into_usize()].into_usize()
                + self.depth[j.into_usize()].into_usize()
                - 2 * self.depth[lca.into_usize()].into_usize();
        }

        sum >> 1
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BuildLCAError {
    #[error("given root index is too large.")]
    InvalidRoot,
    #[error("given CSR is not a tree.")]
    NotTree,
}

impl<Idx: Index, W> TryFrom<(Idx, &UndirectedCSR<Idx, W>)> for LCA<Idx> {
    type Error = BuildLCAError;

    fn try_from((root, csr): (Idx, &UndirectedCSR<Idx, W>)) -> Result<LCA<Idx>, Self::Error> {
        let row = csr.max_index() + 1;
        if root.into_usize() >= row {
            return Err(BuildLCAError::InvalidRoot);
        }
        // |E| = |V| - 1 より連結であることと木であることは同値
        if csr.max_index() * 2 != csr.num_edges() {
            return Err(BuildLCAError::NotTree);
        }

        let mut parent = Vec::with_capacity(row * (row.ilog2() + 1) as usize);
        // CSR に孤立点があると、親が存在しなので、初期化できない。また、未初期化か判定できない。
        // TODO: 実用上は問題ないが、try_from_usize() にするべき
        let null = Idx::from_usize(row);
        parent.extend(std::iter::repeat_n(null, row));
        let mut depth = Vec::with_capacity(row);
        let mut dfs_order = Vec::with_capacity(row);
        let mut stack = Vec::with_capacity(row);
        stack.push(root);
        {
            let depth = depth.spare_capacity_mut();
            let dfs_order = dfs_order.spare_capacity_mut();

            // root の連結成分で DFS
            parent[root.into_usize()] = root;
            depth[root.into_usize()].write(Idx::from_usize(0));
            let mut dfs_cnt = 0;
            while let Some(src) = stack.pop() {
                dfs_order[src.into_usize()].write(Idx::from_usize(dfs_cnt));
                dfs_cnt += 1;

                for &(tar, _) in csr.target(src) {
                    // 未訪問の頂点だけ探索
                    if parent[tar.into_usize()] == null {
                        parent[tar.into_usize()] = src;
                        // SAFETY: src は訪問済みなので、深さが分かっている
                        depth[tar.into_usize()].write(Idx::from_usize(
                            unsafe { depth[src.into_usize()].assume_init() }.into_usize() + 1,
                        ));
                        stack.push(tar);
                    }
                }
            }

            // 連結であることと、木であることは同値
            if dfs_cnt != row {
                return Err(BuildLCAError::NotTree);
            }
        }
        // SAFETY: n 個の異なる頂点を訪問したから。
        unsafe {
            depth.set_len(row);
            dfs_order.set_len(row)
        };

        // squaring で２冪個の祖先を求める。
        let mut offset = 0;
        let mut finished = parent.iter().all(|i| *i == root);
        while !finished {
            finished = true;
            for i in 0..row {
                let a = parent[offset + parent[offset + i].into_usize()];
                parent.push(a);
                finished &= a == root;
            }
            offset += row;
        }
        let col = parent.len() / row;

        Ok(Self {
            ancestor: parent.into_boxed_slice(),
            row,
            col,
            depth: depth.into_boxed_slice(),
            dfs_order: dfs_order.into_boxed_slice(),
        })
    }
}
