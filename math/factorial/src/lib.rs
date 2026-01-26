use mint::Mint;

pub struct Factorial<const MOD: u32> {
    fact: Vec<Mint<MOD>>,
    inv_fact: Vec<Mint<MOD>>,
}

impl<const MOD: u32> Factorial<MOD> {
    /// # Time Complexity
    ///
    /// *Θ*(*N*)
    pub fn with_inverse(n: usize) -> Self {
        assert_eq!(n as u32 as usize, n);

        let mut fact = Vec::with_capacity(n + 1);
        fact.push(Mint::new(1));
        for i in 1..=n {
            fact.push(fact[i - 1] * Mint::new(i as u32));
        }

        let mut inv_fact = Vec::with_capacity(n + 1);
        {
            let uninit = inv_fact.spare_capacity_mut();
            uninit[n].write(fact[n].inv().unwrap());

            for i in (1..=n).rev() {
                // SAFETY: 末尾から初期化する
                uninit[i - 1].write(unsafe { uninit[i].assume_init() * Mint::new(i as u32) });
            }
        }
        // SAFETY: `0..=n`を末尾から初期化した。容量不足の場合は panic している。
        unsafe { inv_fact.set_len(n + 1) };

        Self { fact, inv_fact }
    }

    /// `nCk`を計算する。
    /// `n > k`のときは`0`を返す。
    ///
    /// # Time Complexity
    ///
    /// *Θ*(1)
    pub fn choose(&self, n: usize, k: usize) -> Mint<MOD> {
        if n >= k {
            self.fact[n] * self.inv_fact[k] * self.inv_fact[n - k]
        } else {
            Mint::new(0)
        }
    }
}
