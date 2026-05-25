#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Point2D {
    pub x: i32,
    pub y: i32,
}

impl Point2D {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub const fn dot(&self, other: &Self) -> i64 {
        self.x as i64 * other.x as i64 + self.y as i64 * other.y as i64
    }

    pub const fn cross(&self, other: &Self) -> i64 {
        self.x as i64 * other.y as i64 - self.y as i64 * other.x as i64
    }

    /// Returns the turning direction of the path `a -> b -> c`.
    ///
    /// | `Ordering` | Orientation        |
    /// |------------|--------------------|
    /// | `Greater`  | counterclockwise   |
    /// | `Equal`    | collinear          |
    /// | `Less`     | clockwise          |
    pub fn direction(a: &Self, b: &Self, c: &Self) -> std::cmp::Ordering {
        // Verified with <https://judge.yosupo.jp/problem/count_points_in_triangle>
        let p = {
            let lhs = b.x as i64 - a.x as i64;
            let rhs = c.y as i64 - a.y as i64;
            lhs as i128 * rhs as i128
        };
        let q = {
            let lhs = c.x as i64 - a.x as i64;
            let rhs = b.y as i64 - a.y as i64;
            lhs as i128 * rhs as i128
        };

        p.cmp(&q)
    }

    pub fn cmp_by_atan2(&self, other: &Self) -> std::cmp::Ordering {
        // Verified with <https://judge.yosupo.jp/problem/sort_points_by_argument>
        (self.y.cmp(&0).then(0.cmp(&self.x)))
            .cmp(&other.y.cmp(&0).then(0.cmp(&other.x)))
            .then_with(|| (other.x as i64 * self.y as i64).cmp(&(other.y as i64 * self.x as i64)))
    }
}