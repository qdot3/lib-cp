use csr::{Index, CSR};

/// 有向グラフの強連結成分（SCC）を求める
///
/// # Time Complexity
///
/// *O*(*V* + *E*)
pub fn tarjan_scc<Idx, W>(csr: &impl CSR<Idx, W>) -> SCC<Idx>
where
    Idx: Index,
{
    let n = csr.max_index() + 1;
    let inf = Idx::from_usize(n);

    let mut dfs_stack = Vec::with_capacity(n);
    let mut dfs_order = vec![inf; n];

    let mut scc_stack = Vec::with_capacity(n);
    let mut on_scc_stack = vec![false; n];
    let mut scc = Vec::with_capacity(n);
    let mut scc_partition = Vec::with_capacity(n);
    scc_partition.push(Idx::from_usize(0));

    // SCC 内で自身より dfs 順序が小さなものがあれば、その値を記録する。
    // なければ自身の dfs 順序を記録し、このときに限り scc_stack の子孫と SCC を構成する。
    let mut low_link = vec![inf; n];

    let mut order = 0;
    for i in 0..n {
        if dfs_order[i] != inf {
            continue;
        }

        dfs_stack.push((Idx::from_usize(i), inf));
        while let Some((i, pi)) = dfs_stack.last().copied() {
            if dfs_order[i.into_usize()] == inf {
                // 行きがけ順序
                dfs_order[i.into_usize()] = Idx::from_usize(order);
                low_link[i.into_usize()] = Idx::from_usize(order);
                order += 1;
                scc_stack.push(i);
                on_scc_stack[i.into_usize()] = true;

                for &(j, _) in csr.target(i) {
                    if dfs_order[j.into_usize()] == inf {
                        dfs_stack.push((j, i));
                    } else if on_scc_stack[j.into_usize()] {
                        // ループができる
                        low_link[i.into_usize()] =
                            low_link[i.into_usize()].min(low_link[j.into_usize()])
                    }
                }
            } else {
                // 帰りがけ順序
                dfs_stack.pop();

                if pi != inf {
                    low_link[pi.into_usize()] =
                        low_link[pi.into_usize()].min(low_link[i.into_usize()])
                }

                // SCC の中で最初に訪問したノードなら、scc_stack に残っている子孫たちと SCC をつくる
                if on_scc_stack[i.into_usize()] // 多重辺があるとバグる
                    && low_link[i.into_usize()] == dfs_order[i.into_usize()]
                {
                    // TODO: scc の末尾から埋めることで、Vec を１つ減らすことができる。unsafe
                    while {
                        let j = scc_stack.pop().unwrap();
                        scc.push(j);
                        on_scc_stack[j.into_usize()] = false;

                        j != i
                    } {}
                    scc_partition.push(Idx::from_usize(scc.len()));
                }
            }
        }
    }

    SCC {
        scc: scc.into_boxed_slice(),
        partition: scc_partition.into_boxed_slice(),
    }
}

pub struct SCC<Idx: Index> {
    scc: Box<[Idx]>,
    partition: Box<[Idx]>,
}

impl<Idx: Index> SCC<Idx> {
    // TODO: cargo-equipが対応したら、`+ use<'a, Idx>` とする
    pub fn iter_in_topological_order(
        &self,
    ) -> impl Iterator<Item = &[Idx]> + DoubleEndedIterator + '_ {
        self.partition
            .windows(2)
            .map(|lr| &self.scc[lr[0].into_usize()..lr[1].into_usize()])
            .rev()
    }
}
