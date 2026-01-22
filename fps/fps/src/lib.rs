use std::ops::Mul;

use mint::Mint;

pub struct FPS<T>(
    /// f(x) = Σ_i c[i] x^i
    Vec<T>,
);

impl<T> FPS<T> {
    // TODO: capacity と degree を区別したい。iter.size_hint() ?
    const fn degree(&self) -> usize {
        self.0.len()
    }

    pub fn coefficients(&self, n: usize) -> &[T] {
        &self.0[..n]
    }
}

impl<T> From<Vec<T>> for FPS<T> {
    fn from(value: Vec<T>) -> Self {
        Self(value)
    }
}

impl<T> FromIterator<T> for FPS<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(Vec::from_iter(iter))
    }
}

pub trait NTTFriendlyPrime: Sized {
    const PRIMITIVE_ROOT: u32;
}

impl NTTFriendlyPrime for Mint<998_244_353> {
    const PRIMITIVE_ROOT: u32 = 3;
}

impl<const P: u32> FPS<Mint<P>>
where
    Mint<P>: NTTFriendlyPrime,
{
    /// NTTにおける回転因子の差分
    const RATE: [Mint<P>; 32] = {
        // P = a * 2^n + 1
        let n = (P - 1).trailing_zeros() as usize;
        let a = (P - 1) >> n;

        let mut rate = [Mint::new(0); 32];
        let mut w = Mint::new(Mint::<P>::PRIMITIVE_ROOT).pow(a);
        let mut iw = w.pow(P - 2);

        let mut i = 2;
        while i <= n {
            rate[n - i] = w;

            let mut j = n - i;
            while j + 2 < n {
                j += 1;
                rate[j].const_mul_assign(iw);
            }

            w = w.pow(2);
            iw = iw.pow(2);
            i += 1
        }

        rate
    };

    /// 逆NTTにおける回転因子の差分
    const INV_RATE: [Mint<P>; 32] = {
        // P = a * 2^n + 1
        let n = (P - 1).trailing_zeros() as usize;
        let a = (P - 1) >> n;

        let mut rate = [Mint::new(0); 32];
        let mut iw = Mint::new(Mint::<P>::PRIMITIVE_ROOT).pow(a);
        let mut w = iw.pow(P - 2);

        let mut i = 2;
        while i <= n {
            rate[n - i] = w;

            let mut j = n - i;
            while j + 2 < n {
                j += 1;
                rate[j].const_mul_assign(iw);
            }

            w = w.pow(2);
            iw = iw.pow(2);
            i += 1
        }

        rate
    };

    /// 正順の入力を受け取り、ビット反転順序で返す。
    /// 規格化しない。
    ///
    /// # Time Complexity
    ///
    /// *Θ*(*N* log *N*)
    pub fn ntt_t(mut self, deg: usize, inverse: bool) -> Vec<Mint<P>> {
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
                    r * Self::INV_RATE[i.trailing_ones() as usize]
                } else {
                    r * Self::RATE[i.trailing_ones() as usize]
                };
            }
            w >>= 1;
        }

        self.0
    }

    /// ビット反転順序の入力を受け取り、正順で返す。
    /// 規格化する。
    ///
    /// # Time Complexity
    ///
    /// *Θ*(*N* log *N*)
    pub fn ntt_f(mut self, deg: usize, inverse: bool) -> Vec<Mint<P>> {
        self.0.resize(deg.next_power_of_two(), Mint::new(0));
        let mut w = 1;
        while w < self.0.len() {
            let mut r = Mint::new(1);
            // TODO: r = 1 の場合を特別扱いする。
            for (i, pair) in self.0.chunks_exact_mut(w << 1).enumerate() {
                let (prefix, suffix) = pair.split_at_mut(w);
                for i in 0..w {
                    // TODO: 並列化する
                    (prefix[i], suffix[i]) = (prefix[i] + suffix[i], (prefix[i] - suffix[i]) * r)
                }
                r = if inverse {
                    r * Self::INV_RATE[i.trailing_ones() as usize]
                } else {
                    r * Self::RATE[i.trailing_ones() as usize]
                };
            }
            w <<= 1;
        }

        // 規格化
        let frac_1_n = Mint::new((self.0.len() % P as usize) as u32).inv().unwrap();
        self.0.iter_mut().for_each(|v| *v *= frac_1_n);

        self.0
    }
}

impl<const P: u32> Mul for FPS<Mint<P>>
where
    Mint<P>: NTTFriendlyPrime,
{
    type Output = Self;

    /// # Time Complexity
    ///
    /// *O*(*N* log *N*)
    fn mul(self, rhs: Self) -> Self::Output {
        let deg = self.degree() + rhs.degree() - 1;

        let mut a = self.ntt_t(deg, false);
        let b = rhs.ntt_t(deg, false);

        for (a, b) in a.iter_mut().zip(b.into_iter()) {
            *a *= b
        }

        let c = Self::from(a).ntt_f(deg, true);
        Self::from(c)
    }
}

#[cfg(test)]
mod tests {
    use mint::Mint;

    use crate::FPS;

    #[test]
    fn check_rate() {
        assert_eq!(
            FPS::<Mint<998_244_353>>::RATE[..22],
            [
                Mint::new(0x3656d65b),
                Mint::new(0x1e5ea9e6),
                Mint::new(0x16038782),
                Mint::new(0x13caac90),
                Mint::new(0x3a9a4cfa),
                Mint::new(0x761af21),
                Mint::new(0xe372007),
                Mint::new(0x3a2be7d4),
                Mint::new(0x23fe18b2),
                Mint::new(0x330f5b68),
                Mint::new(0x7d37cf9),
                Mint::new(0x3239edef),
                Mint::new(0x2b8ea5c3),
                Mint::new(0x382d2452),
                Mint::new(0x300e9be2),
                Mint::new(0x908b3f5),
                Mint::new(0x1e726cd9),
                Mint::new(0x1e02c2f0),
                Mint::new(0x2c49629c),
                Mint::new(0x2c2b7c93),
                Mint::new(0x35a5081),
                Mint::new(0x33b69d8b),
            ]
        )
    }
}
