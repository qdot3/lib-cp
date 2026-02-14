use std::io::stdin;

use input::{bind, FastInput};
use proconio::fastout;

#[fastout]
fn main() {
    let mut input: FastInput<std::io::StdinLock<'_>> = FastInput::new(stdin().lock());
    bind! { input >> n: usize, mut k: u32, }
}
