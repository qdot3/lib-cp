use std::ops::RangeBounds;

pub trait SupportedPrime {}

pub struct Prime<const P: u64>;

impl<const P: u64> Prime<P>
where
    Self: SupportedPrime,
{
    #[inline]
    const fn mul_mod(a: u64, b: u64) -> u64 {
        let exp = P.ilog2() as u64 + 1;
        let diff = (1 << exp) - P;
        assert!(2 + diff < 1 << (63 - exp));

        let l_size = exp.div_ceil(2);
        let l_mask = (1 << l_size) - 1;

        let (ua, la) = (a >> l_size, a & l_mask);
        let (ub, lb) = (b >> l_size, b & l_mask);
        let m = ua * lb + ub * la;
        let (um, lm) = (m >> l_size, m & l_mask);

        let res = (ua * ub + um) * (1 + (exp & 1)) * diff + (lm << l_size) + la * lb;
        res % P
    }
}

macro_rules! supported_prime_impl {
    ($n:literal; $( (1 << $exp:literal) - $diff:literal),*$(,)?) => {
        /// Large prime numbers that is suitable for [`RollingHasher`].
        pub const PRIMES: [u64; $n] = [$( { (1 << $exp) - $diff } ),*];

        $(
            impl SupportedPrime for Prime<{ (1 << $exp) - $diff }> {}
        )*
    };
}

supported_prime_impl! {
    // the number of prime numbers. 10 will be sufficient.
    5;
    // # Constraints
    //
    // - P = 2^EXP - DIFF >= 2^56
    // - EXP <= 61
    // - DIFF + 2 < 2^(63 - EXP)
    //
    // 2^57 - x, x < 2^8 = 64
    (1 << 57) - 49,
    (1 << 57) - 25,
    (1 << 57) - 13,
    // 2^58 - x, x < 2^5 = 32
    (1 << 58) - 27,
    // the largest prime number
    (1 << 61) - 1,
}

#[derive(Debug, Clone)]
pub struct RollingHash<const P: u64>
where
    Prime<P>: SupportedPrime,
{
    /// prefix[i] = hash(S[..i]) とする。とくに、prefix[0] = 0（加法単位元）
    prefix: Vec<u64>,
    /// pow_base[i] = base.pow(i)
    pow_base: Vec<u64>,
}

impl<const P: u64> RollingHash<P>
where
    Prime<P>: SupportedPrime,
{
    /// 基数には十分大きな乱数を選ぶこと。
    ///
    /// # Time Complexity
    ///
    /// *Θ*(*N*)
    pub fn new<T>(base: u64, str: &[T]) -> Self
    where
        T: Into<u64> + Clone,
    {
        let mut prefix = Vec::with_capacity(str.len() + 1);
        let mut pow_base = prefix.clone();

        prefix.push(0);
        pow_base.push(1);
        let base = base % P;
        for (i, s) in str.into_iter().enumerate() {
            prefix.push((Prime::<P>::mul_mod(prefix[i], base) + s.clone().into()) % P);
            pow_base.push(Prime::<P>::mul_mod(pow_base[i], base));
        }

        Self { prefix, pow_base }
    }

    /// 部分文字列のハッシュ値を返す
    ///
    /// # Time Complexity
    ///
    /// *Θ*(1)
    pub fn get_hash_value<R>(&self, range: R) -> u64
    where
        R: RangeBounds<usize>,
    {
        let l = match range.start_bound() {
            std::ops::Bound::Included(l) => *l,
            std::ops::Bound::Excluded(l) => l + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let r = match range.end_bound() {
            std::ops::Bound::Included(r) => r + 1,
            std::ops::Bound::Excluded(r) => *r,
            std::ops::Bound::Unbounded => self.prefix.len() - 1,
        };

        // P < 2^61
        let hash = self.prefix[r] + P - Prime::<P>::mul_mod(self.prefix[l], self.pow_base[r - l]);
        hash % P
    }
}
