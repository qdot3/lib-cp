use csr2::{Directed, OutEdge, CSR};

pub struct SCC {
    scc: Vec<usize>,
    partition: Vec<usize>,
}

impl SCC {
    #[must_use]
    pub fn new<W>(csr: &CSR<W, Directed>) -> Self {
        const INF: usize = !0;

        let mut stack = Vec::with_capacity(csr.num_nodes());
        let mut ord_low = vec![(INF, INF); csr.num_nodes()];
        let mut removed = BitSet::new(csr.num_nodes());

        let mut scc = Vec::with_capacity(csr.num_nodes());
        let mut partition = Vec::with_capacity(csr.num_nodes());

        let mut order = 0;
        let mut cursor = csr.num_nodes();
        for i in 0..csr.num_nodes() {
            if ord_low[i].0 != INF {
                continue;
            }

            stack.push((i, i, 0));
            ord_low[i] = (order, order);
            order += 1;
            scc.push(i);

            while let Some((v, p, nth)) = stack.pop() {
                if let Some(OutEdge { target: c, .. }) = csr.nth_edge(v, nth) {
                    stack.push((v, p, nth + 1));

                    if ord_low[c].0 == INF {
                        ord_low[c] = (order, order);
                        order += 1;
                        scc.push(c);

                        stack.push((c, v, 0));
                    } else if removed.is_false(c) {
                        ord_low[v].1 = ord_low[v].1.min(ord_low[c].0)
                    }
                } else {
                    ord_low[p].1 = ord_low[p].1.min(ord_low[v].1);

                    if removed.is_false(v) && {
                        let (ord, low) = ord_low[v];
                        ord == low
                    } {
                        // `v` is the representative node of the scc

                        partition.push(cursor);
                        while let Some(u) = scc.pop() {
                            cursor -= 1;
                            removed.set(u);

                            let i = cursor - scc.len();
                            scc.spare_capacity_mut()[i].write(u);

                            if u == v {
                                break;
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(cursor, 0);

        // SAFETY: all elements have been initialized
        unsafe { scc.set_len(csr.num_nodes()) };
        partition.push(0);

        Self { scc, partition }
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.partition.len() - 1
    }

    #[must_use]
    pub fn topological_iter(&self) -> impl DoubleEndedIterator<Item = &[usize]> {
        self.partition
            .windows(2)
            .rev()
            .map(|w| &self.scc[w[1]..w[0]])
    }
}

struct BitSet {
    block: Vec<usize>,
}

impl BitSet {
    const B: usize = usize::BITS as usize;

    fn new(n: usize) -> Self {
        Self {
            block: vec![0; n.div_ceil(Self::B)],
        }
    }

    fn is_false(&self, i: usize) -> bool {
        let (b, i) = (i / Self::B, i % Self::B);
        (self.block[b] >> i) & 1 == 0
    }

    fn set(&mut self, i: usize) {
        let (b, i) = (i / Self::B, i % Self::B);
        self.block[b] |= 1 << i
    }
}
