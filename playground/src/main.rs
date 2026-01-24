use disjoint_sparse_table::DisjointSparseTable;
use itertools::Itertools;
use ops::ops::Additive;
use point::Point2D;
use proconio::{fastout, input, marker::Usize1};

#[fastout]
fn main() {
    input! { n: usize, q: usize, xy: [(i64, i64); n], }
    let xy = xy
        .into_iter()
        .map(|(x, y)| Point2D::new(x, y))
        .collect_vec();

    let (count, angle): (Vec<_>, Vec<_>) = {
        let mut xy = xy.clone();
        xy.sort_unstable_by(|a, b| a.arg_cmp(*b));
        xy.into_iter()
            .dedup_by_with_count(|a, b| a.arg_cmp(*b).is_eq())
            .unzip()
    };

    let sum = DisjointSparseTable::<Additive<usize>>::from(count);
    for _ in 0..q {
        input! { a: Usize1, b: Usize1, }

        let end = angle.binary_search_by(|v| v.arg_cmp(xy[b])).unwrap();
        let start = angle.binary_search_by(|v| v.arg_cmp(xy[a])).unwrap();

        if start <= end {
            println!("{}", sum.range_query(start..=end).unwrap())
        } else {
            println!("{}", n - sum.range_query(end + 1..start).unwrap_or(0))
        }
    }
}
