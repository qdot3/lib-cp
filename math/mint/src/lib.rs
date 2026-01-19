use std::{
    fmt::Display,
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use num::{integer::ExtendedGcd, Integer, One, Zero};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Mint<const MOD: u32>(u64);

impl<const MOD: u32> Mint<MOD> {
    pub const fn new(x: u32) -> Self {
        Self((x % MOD) as u64)
    }

    /// # Time Complexity
    ///
    /// *Θ*(log `exp`)
    pub const fn pow(mut self, mut exp: u32) -> Self {
        let mut res = 1;
        while exp > 0 {
            if exp & 1 == 1 {
                res = res * self.0 % MOD as u64;
            }
            self.0 = self.0 * self.0 % MOD as u64;
            exp >>= 1;
        }
        Self(res)
    }

    /// 乗法逆元をもとめる。
    ///
    /// # Time Complexity
    ///
    /// *O*(log *A*)
    pub fn inv(self) -> Option<Self> {
        // 内部的に self.0 は u32 に収まるので情報落ちはない
        let ExtendedGcd { gcd, x, y: _ } = (self.0 as i64).extended_gcd(&(MOD as i64));

        gcd.is_one()
            .then_some(Self(x.rem_euclid(MOD as i64) as u64 % MOD as u64))
    }

    pub const fn const_mul_assign(&mut self, other: Self){
        self.0 = self.0 * other.0 % MOD as u64;
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
        Self((self.0 + rhs.0) % MOD as u64)
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
        Self((self.0 + MOD as u64 - rhs.0) % MOD as u64)
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
        Self(self.0 * rhs.0 % MOD as u64)
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
        Self(MOD as u64 - self.0)
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
