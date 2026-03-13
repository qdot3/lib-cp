use std::ops::Index;

pub struct MinPrimeFactor {
    mpf: Box<[u32]>,
}

impl MinPrimeFactor {
    /// # Time Complexity
    ///
    /// *O*(*N* log log *N*)
    pub fn new(n: u32) -> Self {
        if std::mem::size_of::<usize>() < std::mem::size_of::<u32>() {
            assert!(n <= usize::MAX as u32);
        }

        let mut mpf: Box<[u32]> = (0..=n).collect();
        mpf.iter_mut().step_by(2).skip(1).for_each(|p| *p = 2);
        for i in (3..=n as usize).step_by(2) {
            let i = i;
            if mpf[i] == i as u32 {
                // i は素数
                for j in (i * i..=n as usize).step_by(i) {
                    mpf[j] = mpf[j].min(i as u32)
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
    pub fn prime_factors(&self, x: u32) -> impl Iterator<Item = (u32, u8)> + '_ {
        let mut x = x as usize;
        std::iter::from_fn(move || {
            if x <= 1 {
                None
            } else {
                let p = self.mpf[x];
                let mut n = 0;
                while self.mpf[x] == p {
                    n += 1;
                    x /= self.mpf[x] as usize
                }
                Some((p, n))
            }
        })
    }

    /// # Time Complexity
    ///
    /// *O*(1)
    pub fn is_prime(&self, x: u32) -> bool {
        self.mpf[x as usize] == x
    }
}

impl Index<usize> for MinPrimeFactor {
    type Output = u32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.mpf[index]
    }
}
