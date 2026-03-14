pub struct MinPrimeFactor {
    mpf: Box<[[u32; 8]]>,
}

impl MinPrimeFactor {
    /// `self.mpf[k][i]` stores the minimum prime factor of `30k + WHEEL[i]`
    const WHEEL: [u32; 8] = [1, 7, 11, 13, 17, 19, 23, 29];

    /// let `(k, i)` to be `mul(l, r)`, then `(30 k_l + WHEEL[i_l]) * (30 k_r + WHEEL[i_r]) = (30k + WHEEL[i])` holds.
    #[inline]
    const fn mul(lhs: u32, rhs: u32) -> (usize, usize) {
        // `WHEEL[i] * WHEEL[j] % 30` -> position
        let lut = const {
            let mut table = [[0; 8]; 8];

            let mut i = 0;
            while i < 8 {
                let mut j = 0;
                while j < 8 {
                    let rem = Self::WHEEL[i] * Self::WHEEL[j] % 30;
                    let mut k = 0;
                    while rem != Self::WHEEL[k] {
                        k += 1
                    }
                    table[i][j] = k as u32;
                    j += 1;
                }
                i += 1;
            }

            table
        };

        let (kl, il) = (lhs / 8, lhs as usize % 8);
        let (kr, ir) = (rhs / 8, rhs as usize % 8);

        let k = kl * (30 * kr + Self::WHEEL[ir])
            + kr * Self::WHEEL[il]
            + Self::WHEEL[il] * Self::WHEEL[ir] / 30;
        let i = lut[il][ir];

        (k as usize, i as usize)
    }

    pub fn new(max: u32) -> Self {
        let mut mpf = Vec::from_iter(
            std::iter::successors(Some(Self::WHEEL), |prev| {
                Some(std::array::from_fn(|i| prev[i] + 30))
            })
            .take(max.div_ceil(30).max(1) as usize),
        )
        .into_boxed_slice();
        // `0` is equivalent to `None::<NonZeroU32>`
        mpf[0][0] = 0;

        let max_p = (30 * mpf.len()).isqrt() as u32;
        let mut small_primes = Vec::new();

        let chunk_size: usize = const {
            const CACHE_SIZE: usize = 32 << 10;
            CACHE_SIZE / std::mem::size_of_val(&Self::WHEEL)
        };

        let n = chunk_size.min(mpf.len());
        for lhs in 1..n * 8 {
            let (kl, il) = (lhs / 8, lhs % 8);

            let num = 30 * kl as u32 + Self::WHEEL[il];
            if mpf[kl][il] == num {
                let lhs = lhs as u32;
                let mut rhs = lhs;
                let (mut k, mut i) = Self::mul(lhs, rhs);
                while k < n {
                    mpf[k][i % 8] = mpf[k][i % 8].min(num);

                    rhs += 1;
                    (k, i) = Self::mul(lhs, rhs);
                }

                small_primes.push([lhs, rhs]);
            }
        }

        // Segmented Sieve
        let mut offset = chunk_size;
        for mpf in mpf.chunks_mut(chunk_size).skip(1) {
            small_primes.iter_mut().for_each(|[lhs, rhs]| {
                let prime = 30 * (*lhs / 8) + Self::WHEEL[*lhs as usize % 8];

                let (mut k, mut i) = Self::mul(*lhs, *rhs);
                k -= offset;
                while k < mpf.len() {
                    mpf[k][i % 8] = mpf[k][i % 8].min(prime);

                    *rhs += 1;
                    (k, i) = Self::mul(*lhs, *rhs);
                    k -= offset;
                }
            });

            for lhs in offset * 8..(offset + mpf.len()) * 8 {
                let (kl, il) = (lhs / 8, lhs % 8);

                let num = 30 * kl as u32 + Self::WHEEL[il];
                if mpf[kl - offset][il] == num && num <= max_p {
                    let lhs = lhs as u32;
                    let mut rhs = lhs;
                    let (mut k, mut i) = Self::mul(lhs, rhs);
                    k -= offset;
                    while k < mpf.len() {
                        mpf[k][i % 8] = mpf[k][i % 8].min(num);

                        rhs += 1;
                        (k, i) = Self::mul(lhs, rhs);
                        k -= offset;
                    }

                    small_primes.push([lhs, rhs]);
                }
            }

            offset += chunk_size;
        }

        Self { mpf }
    }

    fn inner_index(n: u32) -> Option<(usize, usize)> {
        let lut = const {
            let mut map = [None; 30];

            let mut i = 0;
            while i < 8 {
                map[Self::WHEEL[i] as usize] = Some(i);
                i += 1;
            }
            map
        };

        let n = n as usize;
        lut[n % 30].map(|i| (n / 30, i))
    }

    pub fn is_prime(&self, n: u32) -> bool {
        if let Some((k, i)) = Self::inner_index(n) {
            self.mpf[k][i] == n
        } else {
            [2, 3, 5].contains(&n)
        }
    }

    pub fn factorize(&self, mut n: u32) -> Factorize<impl Iterator<Item = u32> + '_> {
        assert_ne!(n, 0);

        const LUT: [Option<u32>; 30] = {
            let mut mpf = [None; 30];

            let mut i = 0;
            while i < 30 {
                if i % 2 == 0 {
                    mpf[i] = Some(2)
                } else if i % 3 == 0 {
                    mpf[i] = Some(3)
                } else if i % 5 == 0 {
                    mpf[i] = Some(5);
                }

                i += 1
            }

            mpf
        };

        let iter = std::iter::from_fn(move || {
            if let Some(p) = LUT[n as usize % 30] {
                n /= p;
                Some(p)
            } else if let Some((k, i)) = Self::inner_index(n) {
                let p = self.mpf[k][i];
                (p > 0).then(|| {
                    n /= p;
                    p
                })
            } else {
                let v = Some(n);
                n = 1;
                v
            }
        });

        Factorize::new(iter)
    }
}

pub struct Factorize<I>
where
    I: Iterator<Item: Eq>,
{
    iter: I,
    next: Option<I::Item>,
}

impl<I> Factorize<I>
where
    I: Iterator<Item: Eq>,
{
    fn new(mut iter: I) -> Self {
        Self {
            next: iter.next(),
            iter,
        }
    }
}

impl<I> Iterator for Factorize<I>
where
    I: Iterator<Item: Eq>,
{
    type Item = (I::Item, u8);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(p) = self.next.take() {
            let mut count = 1;
            while let Some(v) = self.iter.next() {
                if v == p {
                    count += 1;
                } else {
                    let _ = self.next.insert(v);
                    break;
                }
            }

            Some((p, count))
        } else {
            None
        }
    }
}
