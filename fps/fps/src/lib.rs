use mint::Mint;

pub struct FPS<T>(
    /// f(x) = Σ_i c[i] x^i
    Vec<T>,
);

impl<T> From<Vec<T>> for FPS<T> {
    fn from(value: Vec<T>) -> Self {
        Self(value)
    }
}

pub trait NTTFriendlyPrime: Sized {
    /// p = A * 2^N + 1
    const A: u32;
    const N: u32;
    const PRIMITIVE_ROOT: u32;

    fn next_rotating_factor(self, i: usize) -> Self {
        let rate = const {
            let mut rate = [0_u64; 32];
            let mut i = 0;
            while i < Self::N {
                i += 1;
            }
        };
        todo!()
    }

    fn next_rotating_factor_inv(self, i: usize) -> Self {
        todo!()
    }
}

impl<const P: u32> FPS<Mint<P>>
where
    Mint<P>: NTTFriendlyPrime,
{
    /// 正順の入力を受け取り、ビット反転順序で返す。
    ///
    /// # Time Complexity
    ///
    /// *Θ*(*N* log *N*)
    pub fn ntt_t(&mut self, deg: usize, inverse: bool) {
        self.0.resize(deg.next_power_of_two(), Mint::new(0));
        let mut w = self.0.len() >> 1;
        while w > 0 {
            let mut r = Mint::new(1);
            for (i, pair) in self.0.chunks_exact_mut(w << 1).enumerate() {
                let (prefix, suffix) = pair.split_at_mut(w);
                for i in 0..w {
                    (prefix[i], suffix[i]) = (prefix[i] + suffix[i] * r, prefix[i] - suffix[i] * r)
                }
                r = if inverse {
                    r.next_rotating_factor_inv(i)
                } else {
                    r.next_rotating_factor(i)
                };
            }
            w >>= 1;
        }
    }

    /// ビット反転順序の入力を受け取り、正順で返す。
    ///
    /// # Time Complexity
    ///
    /// *Θ*(*N* log *N*)
    pub fn ntt_f(&mut self, deg: usize, inverse: bool) {
        self.0.resize(deg.next_power_of_two(), Mint::new(0));
        let mut w = 1;
        while w < self.0.len() {
            let mut r = Mint::new(1);
            for (i, pair) in self.0.chunks_exact_mut(w << 1).enumerate() {
                let (prefix, suffix) = pair.split_at_mut(w);
                for i in 0..w {
                    (prefix[i], suffix[i]) = (prefix[i] + suffix[i], (prefix[i] - suffix[i]) * r)
                }
                r = if inverse {
                    r.next_rotating_factor_inv(i)
                } else {
                    r.next_rotating_factor(i)
                };
            }
            w <<= 1;
        }
        todo!("規格化する")
    }
}
