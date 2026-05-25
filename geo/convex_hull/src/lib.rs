use point::Point2D;

pub struct ConvexHull {
    pub upper: Vec<Point2D>,
    pub lower: Vec<Point2D>,
}

impl ConvexHull {
    /// Computes the convex hull of `points`.
    ///
    /// The endpoints of `upper` and `lower` may overlap.
    ///
    /// # Time complexity
    ///
    /// O(N log N)
    pub fn new(mut points: Vec<Point2D>) -> Self {
        // Verified with <https://judge.yosupo.jp/problem/static_convex_hull>
        if points.is_empty() {
            return ConvexHull {
                upper: Vec::new(),
                lower: Vec::new(),
            };
        }

        let mut upper = Vec::with_capacity(points.len());
        let mut lower = Vec::with_capacity(points.len());

        points.sort_unstable_by_key(|p| p.x);
        for [u, l] in points.chunk_by(|a, b| a.x == b.x).map(|chunk| {
            let [mut u, mut l] = [chunk[0]; 2];
            for p in chunk.iter().skip(1) {
                if u.y < p.y {
                    u = *p
                } else if l.y > p.y {
                    l = *p
                }
            }
            [u, l]
        }) {
            while upper.len() >= 2 && {
                let n = upper.len();
                Point2D::direction(&upper[n - 2], &upper[n - 1], &u).is_ge()
            } {
                upper.pop();
            }
            upper.push(u);

            while lower.len() >= 2 && {
                let n = lower.len();
                Point2D::direction(&lower[n - 2], &lower[n - 1], &l).is_le()
            } {
                lower.pop();
            }

            lower.push(l);
        }

        ConvexHull { upper, lower }
    }
}
