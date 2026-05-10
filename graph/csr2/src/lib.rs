struct Edge<W> {
    source: usize,
    target: usize,
    weight: W,
}

pub struct CSRBuilder<W> {
    edges: Vec<Edge<W>>,
    max_node: usize,
}

impl<W> CSRBuilder<W> {
    pub fn with_capacity(capacity: usize) -> Self {
        todo!()
    }

    /// Append an directed edge.
    pub fn push(&mut self, source: usize, target: usize, weight: W) {
        self.edges.push(Edge {
            source,
            target,
            weight,
        });

        self.max_node = self.max_node.max(source).max(target)
    }

    pub fn build(self) -> CSR<W> {
        let edges = self.edges;
        let n = self.max_node.checked_add(1).unwrap();

        let mut cnt = vec![0; n];
        edges.iter().for_each(|e| {
            cnt[e.source] += 1;
        });
        for i in 1..cnt.len() {
            cnt[i] += cnt[i - 1]
        }
        debug_assert_eq!(cnt[n - 1], edges.len());

        let n_edges = edges.len();
        let mut target = Vec::with_capacity(edges.len());
        {
            let uninit = target.spare_capacity_mut();
            assert!(uninit.len() >= edges.len(), "bug");

            for Edge {
                source,
                target,
                weight,
            } in edges
            {
                cnt[source] -= 1;
                uninit[cnt[source] as usize].write((target, weight));
            }
        }
        // SAFETY: `target` has sufficient capacity, or the function would have already panicked.
        // The first `n_edges` elements have been initialized.
        unsafe { target.set_len(n_edges) };
        CSR {
            target,
            partition: cnt,
        }
    }
}

pub struct CSR<W> {
    target: Vec<(usize, W)>,
    partition: Vec<usize>,
}

impl<W> CSR<W> {
    pub fn targets(&self, source: usize) -> &[(usize, W)] {
        &&self.target[self.partition[source]..self.partition[source + 1]]
    }

    pub fn num_node(&self) -> usize {
        // `partition` has at least one element.
        self.partition.len() - 1
    }
}
