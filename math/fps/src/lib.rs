use std::ops::{Index, IndexMut, Mul};

use mint::Mint;

#[derive(Debug, Clone)]
pub struct FPS<T>(
    /// f(x) = Σ_i c[i] x^i
    Vec<T>,
);

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

impl<T> Index<usize> for FPS<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T> IndexMut<usize> for FPS<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
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
    /// # Panic
    ///
    /// `sef.len()` should be power of two
    ///
    /// # Time Complexity
    ///
    /// *Θ*(*N* log *N*)
    fn butterfly(f: &mut [Mint<P>]) {
        assert!(f.len().is_power_of_two());

        let mut w = f.len() >> 1;
        while w > 0 {
            let mut r = Mint::new(1);
            for (i, pair) in f.chunks_exact_mut(w << 1).enumerate() {
                let (prefix, suffix) = pair.split_at_mut(w);
                for i in 0..w {
                    (prefix[i], suffix[i]) = (prefix[i] + suffix[i] * r, prefix[i] - suffix[i] * r)
                }
                r *= Self::RATE[i.trailing_ones() as usize]
            }
            w >>= 1;
        }
    }

    /// ビット反転順序の入力を受け取り、正順で返す。
    /// 規格化しない。
    ///
    /// # Panic
    ///
    /// `sef.len()` should be power of two
    ///
    /// # Time Complexity
    ///
    /// *Θ*(*N* log *N*)
    fn butterfly_inv(f: &mut [Mint<P>]) {
        assert!(f.len().is_power_of_two());

        let mut w = 1;
        while w < f.len() {
            let mut r = Mint::new(1);
            // TODO: r = 1 の場合を特別扱いする。
            for (i, pair) in f.chunks_exact_mut(w << 1).enumerate() {
                let (prefix, suffix) = pair.split_at_mut(w);
                for i in 0..w {
                    (prefix[i], suffix[i]) = (prefix[i] + suffix[i], (prefix[i] - suffix[i]) * r)
                }
                r *= Self::INV_RATE[i.trailing_ones() as usize]
            }
            w <<= 1;
        }
    }

    pub fn ones(deg: usize) -> Self {
        Self(vec![Mint::new(1); deg + 1])
    }

    /// Calculates degree of the FPS.
    ///
    /// `deg(0)` is defined as `None`
    ///
    /// # Time Complexity
    ///
    /// *O*(*N*)
    pub fn degree(&self) -> Option<usize> {
        self.0.len().checked_sub(
            1 + self
                .0
                .iter()
                .rev()
                .take_while(|v| **v == Mint::new(0))
                .count(),
        )
    }

    const NORM: [Mint<P>; 32] = {
        let mut norm = [Mint::new(0); 32];

        let mut exp = 0;
        while exp <= P.ilog2() {
            norm[exp as usize] = Mint::new(2).pow(exp).pow(P - 2);
            exp += 1;
        }
        norm
    };

    #[inline]
    fn _inv(f: &[Mint<P>], k: usize, g: &mut [Mint<P>], buf: &mut [Mint<P>]) {
        assert!(k.is_power_of_two());
        assert!(f.len() >= k);
        assert!(g.len().min(buf.len()) >= 2 * k);

        let mut w = 2;
        g[0] = f[0].inv().unwrap();
        while w <= k {
            w *= 2;

            // deg(f) < w/2
            buf[..w / 2].copy_from_slice(&f[..w / 2]);
            let f = &mut buf[..w];
            f[w / 2..].fill(Mint::new(0));
            Self::butterfly(f);

            // deg(g) < w/4
            let g = &mut g[..w];
            g[w / 4..].fill(Mint::new(0));
            Self::butterfly(g);

            // deg(2g - fg^2) < w
            for i in 0..w {
                g[i] = g[i] * Mint::new(2) - f[i] * g[i] * g[i]
            }

            Self::butterfly_inv(g);
            g[..w / 2]
                .iter_mut()
                .for_each(|g| *g *= Self::NORM[w.ilog2() as usize]);
        }
    }

    /// Calculate a function `g` that satisfies `fg = 1 (mod x^k)`
    ///
    /// # Time Complexity
    ///
    /// *Θ*(*N* log *N*)
    pub fn inv(self, k: usize) -> Option<Self> {
        if self.0.get(0).is_some_and(|f0| *f0 != Mint::new(0)) {
            let k = k.next_power_of_two();

            let mut f = self.0;
            f.truncate(k);
            f.resize(k * 5, Mint::new(0));

            {
                let (f, rest) = f.split_at_mut(k);
                let (g, buf) = rest.split_at_mut(k * 2);

                Self::_inv(f, k, g, buf);
                f[..k].copy_from_slice(&g[..k]);
            }

            f.truncate(k);
            Some(Self(f))
        } else {
            None
        }
    }

    fn _log(f: &[Mint<P>], k: u32, g: &mut [Mint<P>], buf: &mut [Mint<P>], cache: &ModInvCache<P>) {
        let k = k as usize;

        assert!(k.is_power_of_two());
        assert!(f.len() >= k);
        assert_eq!(f[0], Mint::new(1));
        assert!(g.len().min(buf.len()) >= 2 * k);

        Self::_inv(f, k, g, buf);
        g[k..].fill(Mint::new(0));
        let g = &mut g[..k * 2];
        Self::butterfly(g);

        for i in 1..k {
            buf[i - 1] = Mint::new(i as u32) * f[i]
        }
        let f_prime = &mut buf[..k * 2];
        f_prime[k..].fill(Mint::new(0));
        Self::butterfly(f_prime);

        for i in 0..k * 2 {
            g[i] *= f_prime[i]
        }
        Self::butterfly_inv(g);
        let i = k.trailing_zeros() as usize + 1;
        g[..k].iter_mut().for_each(|g| *g *= Self::NORM[i]);

        for i in (1..k).rev() {
            g[i] = g[i - 1] * Mint::new(cache.get(i))
        }
        g[0] = Mint::new(0)
    }

    pub fn log(self, k: usize, cache: &mut ModInvCache<P>) -> Option<Self> {
        if self.0.get(0).is_some_and(|v| *v == Mint::new(1)) {
            cache.extend(k as u32);
            let mut f = self.0;

            let k = k.next_power_of_two();
            f.truncate(k);
            f.resize(k * 5, Mint::new(0));

            {
                let (f, rest) = f.split_at_mut(k);
                let (g, buf) = rest.split_at_mut(k * 2);
                Self::_log(f, k as u32, g, buf, cache);
                f.copy_from_slice(&g[..k]);
            }

            f.truncate(k);
            Some(Self(f))
        } else {
            None
        }
    }

    fn _exp(
        f: &[Mint<P>],
        k: u32,
        g: &mut [Mint<P>],
        buf1: &mut [Mint<P>],
        buf2: &mut [Mint<P>],
        cache: &ModInvCache<P>,
    ) {
        let k = k as usize;

        assert!(k.is_power_of_two());
        assert!(f.len() >= k);
        assert_eq!(f[0], Mint::new(0));
        assert!(g.len().min(buf1.len()).min(buf2.len()) >= 2 * k);

        let mut w = 1;
        g[0] = Mint::new(1);
        while w <= k {
            w *= 2;

            // deg(g) < w/2
            let g = &mut g[..w];
            g[w / 2..].fill(Mint::new(0));

            // deg(log g) < w/2
            let log_g = &mut buf1[..w];
            Self::_log(g, w as u32 / 2, log_g, buf2, cache);
            log_g[w / 2..].fill(Mint::new(0));

            // deg(f) < w/2
            buf2[..w / 2].copy_from_slice(&f[..w / 2]);
            let f = &mut buf2[..w];
            f[w / 2..].fill(Mint::new(0));

            Self::butterfly(g);
            Self::butterfly(log_g);
            Self::butterfly(f);
            for i in 0..w {
                // deg(g) * deg(1 - log g + f) < w
                g[i] = g[i] * (Mint::new(1) - log_g[i] + f[i]) // mod 2^(w/2)
            }

            Self::butterfly_inv(g);
            let i = w.trailing_zeros() as usize;
            g[..w / 2].iter_mut().for_each(|g| *g *= Self::NORM[i]);
        }
    }

    // pub fn exp()

    pub fn pow(mut self, exp: u32, k: u32, cache: &mut ModInvCache<P>) -> Self {
        let k = k as usize;

        if let Some(d) = self.0.iter().position(|v| *v != Mint::new(0)) {
            if let Some(k) = k.checked_sub(d.saturating_mul(exp as usize)) {
                let (mut f, k, offset) = {
                    let mut f = self.0;
                    f.drain(..d);

                    let k = k.next_power_of_two();
                    let offset = d * exp as usize;

                    f.truncate(k);
                    f.resize((k * 7).max(k + offset), Mint::new(0));

                    (f, k, offset)
                };

                cache.extend(k as u32);

                {
                    let (f, rest) = f.split_at_mut(k);
                    let (g, rest) = rest.split_at_mut(k * 2);
                    let (buf1, rest) = rest.split_at_mut(k * 2);
                    let buf2 = &mut rest[..k * 2];

                    let f0 = f[0].pow(exp);
                    let inv_f0 = f0.inv().unwrap();
                    f.iter_mut().for_each(|f| *f *= inv_f0);

                    Self::_log(f, k as u32, g, buf1, cache);
                    f.copy_from_slice(&g[..k]);
                    f.iter_mut().for_each(|f| *f *= Mint::new(exp));
                    Self::_exp(f, k as u32, g, buf1, buf2, cache);
                    g[..k].iter_mut().for_each(|g| *g *= f0);
                }

                f.copy_within(k..k * 2, offset);
                f.truncate(k + offset);
                f[..offset].fill(Mint::new(0));

                return Self(f);
            }
        }

        self.0.clear();
        self.0.extend(std::iter::repeat_n(Mint::new(0), k));
        self
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
        if let (Some(d0), Some(d1)) = (self.degree(), rhs.degree()) {
            let w = (d0 + d1 + 1).next_power_of_two();

            let mut lhs = self.0;
            lhs.resize(w, Mint::new(0));
            let mut rhs = rhs.0;
            rhs.resize(w, Mint::new(0));

            Self::butterfly(&mut lhs);
            Self::butterfly(&mut rhs);
            lhs.iter_mut()
                .zip(rhs.into_iter())
                .for_each(|(l, r)| *l *= r);
            Self::butterfly_inv(&mut lhs);

            let norm = if std::mem::size_of::<usize>() > std::mem::size_of_val(&P) {
                Mint::new((w % P as usize) as u32).inv().unwrap()
            } else {
                Mint::new(w as u32).inv().unwrap()
            };
            lhs.iter_mut().for_each(|v| *v *= norm);

            Self::from(lhs)
        } else {
            Self(Vec::new())
        }
    }
}

pub struct ModInvCache<const P: u32>(Vec<u32>);

impl<const P: u32> ModInvCache<P> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    fn extend(&mut self, n: u32) {
        assert!(n < P);

        self.0.reserve(n.min(P - 1) as usize);
        while self.0.len() < 2 {
            self.0.push(self.0.len() as u32);
        }

        for i in self.0.len() as u32..=n {
            let inv = P as u64 - (P / i) as u64 * self.0[(P % i) as usize] as u64 % P as u64;
            self.0.push(inv as u32);
        }
    }

    fn get(&self, i: usize) -> u32 {
        self.0[i]
    }
}
