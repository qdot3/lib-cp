use std::{
    fmt::Debug,
    ops::{Add, AddAssign, Div, DivAssign, Neg, Sub, SubAssign},
};

use num::{Num, Signed};

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
    /// - `= 0`: 平行
    pub fn det(self, other: Self) -> T {
        self.x * other.y - self.y * other.x
    }

    /// ２つのベクトルの内積をとる
    pub fn dot(self, other: Self) -> T {
        self.x * other.x + self.y * other.y
    }
}

/// # Time Complexity
///
/// *O*(*N* log *N*)
///
/// # Constraints
///
/// - 除算を用いないので、整数型でもよい
/// - ２点の掛け算でオーバーフローしないこと
pub fn convex_hull<T: Signed + Num + Copy + Ord>(mut points: Vec<Point2D<T>>) -> ConvexHull<T> {
    if points.is_empty() {
        return ConvexHull {
            upper: Vec::new(),
            lower: Vec::new(),
        };
    }

    // 凸多角形を適当に作り、その内部の点を削除する（Akl–Toussaint heuristic）
    {
        // 右回りの凸多角形。点は全て異なる。
        let mut outer = {
            let mut octet = [points[0]; 8];
            for p in points.iter().skip(1) {
                if octet[0].y < p.y {
                    octet[0] = *p // top
                } else if octet[4].y > p.y {
                    octet[4] = *p // bottom
                }

                if octet[2].x < p.x {
                    octet[2] = *p // right
                } else if octet[6].x > p.x {
                    octet[6] = *p // left
                }

                if octet[1].x + octet[1].y < p.x + p.y {
                    octet[1] = *p // top right
                } else if octet[5].x + octet[5].y > p.x + p.y {
                    octet[5] = *p // bottom left
                }

                if octet[3].x - octet[3].y < p.x - p.y {
                    octet[3] = *p // bottom right
                } else if octet[7].x - octet[7].y > p.x - p.y {
                    octet[7] = *p // top left
                }
            }
            let mut octet = octet.to_vec();
            octet.dedup();
            octet
        };

        // 凸多角形内部の点を削除
        if outer.len() > 2 {
            outer.extend_from_within(0..1);
            points.retain(|p| {
                // outer の頂点は p から見て時計回りに並んでいる <=> p は outer の内部
                !outer
                    .windows(2)
                    .all(|pts| (pts[0] - *p).det(pts[1] - *p).is_negative())
            });
        }
    }

    // 凸包を求める。双対変換 + CHT の組み合わせで、上下の凸包を求める。
    // (a, b) -> y = ax + b と双対変換すると、上（下）側の包絡線が上（下）部凸包になる。
    let points = {
        points.sort_unstable();
        points
    };
    let mut upper: Vec<Point2D<T>> = Vec::with_capacity(points.len());
    let mut lower = upper.clone();
    for (pl, pu) in points
        .chunk_by(|a, b| a.x == b.x)
        // y 切片が大きいもの
        .map(|chunk| (chunk[0], chunk.last().copied().unwrap()))
    {
        while upper.len() >= 2 && {
            let n = upper.len();
            (upper[n - 1] - upper[n - 2])
                .det(pu - upper[n - 2])
                .is_positive()
        } {
            upper.pop();
        }
        upper.push(pu);

        while lower.len() >= 2 && {
            let n = lower.len();
            (lower[n - 1] - lower[n - 2])
                .det(pl - lower[n - 2])
                .is_negative()
        } {
            lower.pop();
        }
        lower.push(pl);
    }

    ConvexHull { upper, lower }
}

#[derive(Debug, Clone)]
pub struct ConvexHull<T> {
    // x について昇順に点をもつ
    pub upper: Vec<Point2D<T>>,
    pub lower: Vec<Point2D<T>>,
}
