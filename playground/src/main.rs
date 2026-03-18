use std::io::stdin;

use fps::{ModInvCache, FPS};
use input::{bind, FastInput};
use mint::Mint;
use proconio::fastout;

#[fastout]
fn main() {
    // let mut input: FastInput<std::io::StdinLock<'_>> = FastInput::new(stdin().lock());
    // bind! { input >> n: usize, }

    let poly = FPS::<Mint<998_244_353>>::from_iter([1, 1, 1].into_iter().map(|i| Mint::new(i)));
    println!("{:?}", poly.pow(2, 5, &mut ModInvCache::new()));
}
