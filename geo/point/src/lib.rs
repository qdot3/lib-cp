use std::{
    cmp::Ordering,
    fmt::Debug,
    ops::{Add, AddAssign, Div, DivAssign, Neg, Sub, SubAssign},
};

use num::{Integer, Num, Signed};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point2D<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point2D<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T: Add<Output = T>> Add for Point2D<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T: AddAssign> AddAssign for Point2D<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<T: Sub<Output = T>> Sub for Point2D<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T: SubAssign> SubAssign for Point2D<T> {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl<T: Div<Output = T> + Clone> Div<T> for Point2D<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self {
            x: self.x / rhs.clone(),
            y: self.y / rhs,
        }
    }
}

impl<T: DivAssign + Clone> DivAssign<T> for Point2D<T> {
    fn div_assign(&mut self, rhs: T) {
        self.x /= rhs.clone();
        self.y /= rhs;
    }
}

impl<T: Neg<Output = T>> Neg for Point2D<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl<T: Num + Signed> Point2D<T> {
    /// ２つのベクトルがつくる平行四辺形の符号付き面積を計算する
    ///
    /// - `> 0`: 反時計回り
    /// - `< 0`: 時計回り
    /// - `= 0`: 平行・反平行
    pub fn det(self, other: Self) -> T {
        self.x * other.y - self.y * other.x
    }

    /// ２つのベクトルの内積をとる
    pub fn dot(self, other: Self) -> T {
        self.x * other.x + self.y * other.y
    }
}

impl<T: Integer + Signed + Copy> Point2D<T> {
    /// 偏角で比較する。
    pub fn arg_cmp(self, other: Self) -> Ordering {
        ((self.y, self.x) < (T::zero(), T::zero()))
            .cmp(&((other.y, other.x) < (T::zero(), T::zero())))
            .then(self.det(other).cmp(&T::zero()))
    }
}
