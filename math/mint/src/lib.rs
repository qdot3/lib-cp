use std::{
    fmt::Display,
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use num_integer::{ExtendedGcd, Integer};
use num_traits::{One, Zero};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Mint<const MOD: u32>(u32);

impl<const MOD: u32> Mint<MOD> {
    pub const fn new(x: u32) -> Self {
        Self(x % MOD)
    }

    /// # Time Complexity
    ///
    /// *Θ*(log `exp`)
    pub const fn pow(self, mut exp: u32) -> Self {
        let mut val = self.0 as u64;
        let mut res = 1 as u64;
        while exp > 0 {
            if exp & 1 == 1 {
                res = res * val % MOD as u64;
            }
            val = val * val % MOD as u64;
            exp >>= 1;
        }
        Self(res as u32)
    }

    /// 乗法逆元をもとめる。
    ///
    /// # Time Complexity
    ///
    /// *O*(log *A*)
    pub fn inv(self) -> Option<Self> {
        // 内部的に self.0 は u32 に収まるので情報落ちはない
        let ExtendedGcd { gcd, x, y: _ } = (self.0 as i64).extended_gcd(&(MOD as i64));

        gcd.is_one().then_some({
            let inv = x.rem_euclid(MOD as i64) as u64 % MOD as u64;
            Self(inv as u32)
        })
    }

    pub const fn const_mul_assign(&mut self, other: Self) {
        self.0 = (self.0 as u64 * other.0 as u64 % MOD as u64) as u32;
    }
}

impl<const MOD: u32> Display for Mint<MOD> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<const MOD: u32> Add for Mint<MOD> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let (mut x, b) = self.0.overflowing_add(rhs.0);
        if b || x >= MOD {
            x = x.wrapping_sub(MOD);
        }
        Self(x as u32)
    }
}

impl<const MOD: u32> AddAssign for Mint<MOD> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl<const MOD: u32> Sub for Mint<MOD> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let (mut x, b) = self.0.overflowing_sub(rhs.0);
        if b {
            x = x.wrapping_add(MOD);
        }
        Self(x as u32)
    }
}

impl<const MOD: u32> SubAssign for Mint<MOD> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl<const MOD: u32> Mul for Mint<MOD> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let x = self.0 as u64 * rhs.0 as u64 % MOD as u64;
        Self(x as u32)
    }
}

impl<const MOD: u32> MulAssign for Mint<MOD> {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}

impl<const MOD: u32> Neg for Mint<MOD> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(if self.0 == 0 { 0 } else { MOD - self.0 })
    }
}

impl<const MOD: u32> Zero for Mint<MOD> {
    fn zero() -> Self {
        Self(0)
    }

    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl<const MOD: u32> One for Mint<MOD> {
    fn one() -> Self {
        Self(1)
    }
}

impl<T, const MOD: u32> From<T> for Mint<MOD>
where
    T: Into<u32>,
{
    fn from(value: T) -> Self {
        Self::new(value.into())
    }
}
