use std::io::stdin;

use floor_sum::floor_sum;
use input::{bind, FastInput};
use proconio::fastout;

#[fastout]
fn main() {
    let mut input = FastInput::new(stdin().lock());
    bind! { input >> t: u32, }

    for _ in 0..t {
        bind! { input >> n: i64, m: i64, a: i64, b: i64, }

        let x = floor_sum(n, m, a, b);
        let y = floor_sum(n, m, a - 1, b - 1);

        println!("{}", n - (x - y))
    }
}
