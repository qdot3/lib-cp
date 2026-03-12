use std::io::stdin;

use input::{bind, FastInput};
use proconio::fastout;
use simd_bit::SimdBIT;

#[fastout]
fn main() {
    let mut input: FastInput<std::io::StdinLock<'_>> = FastInput::new(stdin().lock());
    bind! { input >> n: usize, q: usize, a: [i64; n], }

    let mut bit = SimdBIT::<i64, 8>::from(a);
    #[cfg(debug_assertions)]
    println!("{:?}", bit);

    for _ in 0..q {
        bind! { input >> t: u8, }

        if t == 0 {
            bind! { input >> p: usize, x: i64, }

            bit.point_add(p, x);

            #[cfg(debug_assertions)]
            println!("{:?}", bit);
        } else {
            bind! { input >> l: usize, r: usize, }

            let val = bit.range_sum(l..r);
            println!("{}", val)
        }
    }
}
