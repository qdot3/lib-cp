use floor_sum::floor_sum;
use proconio::{fastout, input};

#[fastout]
fn main() {
    input! { t: usize, }

    for _ in 0..t {
        input! { n: i64, m: i64, a: i64, b: i64, }

        let x = floor_sum(n, m, a, b);
        let y = floor_sum(n, m, a - 1, b-1);

        println!("{}", n - (x - y))
    }
}
