use csr::{Index, CSR};
use fixedbitset::FixedBitSet;

/// CSR は無効グラフでなければならない
///
/// TODO: 最大独立集合を求めるためには連結成分が複数ある場合を正しく取り扱うように修正する必要がある。
pub fn try_into_bipartite<Idx, W>(csr: &impl CSR<Idx, W>) -> Option<BipartiteGraph<Idx>>
where
    Idx: Index,
{
    let n = csr.max_index() + 1;
    let mut is_black = FixedBitSet::with_capacity(n);

    let mut stack = Vec::with_capacity(n);
    let mut checked = FixedBitSet::with_capacity(n);
    for i in 0..n {
        if checked[i] {
            continue;
        }

        // color of i-th node is white
        checked.insert(i);
        stack.push(Idx::from_usize(i));
        while let Some(i) = stack.pop() {
            for (j, _) in csr.target(i) {
                if checked[j.into_usize()] {
                    // conflict!
                    if is_black[i.into_usize()] == is_black[j.into_usize()] {
                        return None;
                    }
                } else {
                    checked.insert(j.into_usize());
                    stack.push(*j);

                    is_black.set(j.into_usize(), !is_black[i.into_usize()]);
                }
            }
        }
    }

    let mut nodes = Vec::with_capacity(n);
    let partition = {
        let nodes = nodes.spare_capacity_mut();
        let (mut l, mut r) = (0, n);
        for i in 0..n {
            if is_black[i] {
                nodes[l].write(Idx::from_usize(i));
                l += 1;
            } else {
                r -= 1;
                nodes[r].write(Idx::from_usize(i));
            }
        }
        assert_eq!(l, r, "portion of `Vec<Idx>` is NOT initialized");

        l
    };
    // SAFETY
    //
    // n 以上の容量を確保し、n 個の要素を [0, n) に書き込んだから。
    unsafe {
        nodes.set_len(n);
    }

    Some(BipartiteGraph {
        nodes: nodes.into_boxed_slice(),
        partition,
    })
}

pub struct BipartiteGraph<Idx: Index> {
    nodes: Box<[Idx]>,
    partition: usize,
}

impl<Idx: Index> BipartiteGraph<Idx> {
    pub fn blacks(&self) -> &[Idx] {
        &self.nodes[..self.partition]
    }

    pub fn whites(&self) -> &[Idx] {
        &self.nodes[self.partition..]
    }
}
