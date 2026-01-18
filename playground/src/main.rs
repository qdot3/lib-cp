use std::collections::VecDeque;

use proconio::{input, marker::Bytes};

fn main() {
    input! { n: usize, a: u64, b: u64, s: Bytes, }

    let mut min_cost = !0;
    let mut s = VecDeque::from(s);
    for i in 0..n {
        let mut cost = i as u64 * a;
        for (l, r) in (0..n).zip((0..n).rev()).take(n / 2) {
            if s[l] != s[r] {
                cost += b
            }
        }
        min_cost = min_cost.min(cost);
        s.rotate_left(1);
    }
    println!("{}", min_cost)
}
