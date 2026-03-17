use std::io::stdin;

use fps::FPS;
use input::{bind, FastInput};
use mint::Mint;
use proconio::fastout;

#[fastout]
fn main() {
    // let mut input: FastInput<std::io::StdinLock<'_>> = FastInput::new(stdin().lock());
    // bind! { input >> n: usize, }

    let poly =
        FPS::<Mint<998_244_353>>::from_iter([5, 4, 3, 2, 1].into_iter().map(|i| Mint::new(i)));
    let inv = poly.inv(10).unwrap();
    println!("{:?}", inv);
    println!("{:?}", poly * inv);
}


