use std::ops::Index;

pub struct MinPrimeFactor {
    mpf: Box<[usize]>,
}

impl MinPrimeFactor {
    /// # Time Complexity
    ///
    /// *O*(*N* log log *N*)
    pub fn new(n: usize) -> Self {
        let mut mpf: Box<[usize]> = (0..=n).collect();
        mpf.iter_mut().step_by(2).skip(1).for_each(|p| *p = 2);
        for i in (3..=n).step_by(2) {
            let i = i;
            if mpf[i] == i {
                // i は素数
                for j in (i * i..=n).step_by(i) {
                    mpf[j] = mpf[j].min(i)
                }
            }
        }

        Self { mpf }
    }

    /// `n`を素因数分解する。
    ///
    /// # Time Complexity
    ///
    /// *O*(log *K*) where K is the number of prime factors
    pub fn prime_factors(&self, mut x: usize) -> impl Iterator<Item = (usize, usize)> + '_ {
        std::iter::from_fn(move || {
            if x <= 1 {
                None
            } else {
                let p = self.mpf[x];
                let mut n = 0;
                while self.mpf[x] == p {
                    n += 1;
                    x /= self.mpf[x]
                }
                Some((p, n))
            }
        })
    }

    /// # Time Complexity
    ///
    /// *O*(1)
    pub fn is_prime(&self, x: usize) -> bool {
        self.mpf[x] == x
    }
}

impl Index<usize> for MinPrimeFactor {
    type Output = usize;

    fn index(&self, index: usize) -> &Self::Output {
        &self.mpf[index]
    }
}
