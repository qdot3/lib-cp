use csr2::CSR;

pub struct LCA {
    in_out: Vec<usize>,
}

impl LCA {
    pub fn new<W, G>(graph: CSR<W, G>, root: usize) -> (Self, CSR<W, G>) {
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
