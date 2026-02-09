use std::{ io::stdin};

use input::{bind, FastInput};
use proconio::fastout;

#[fastout]
fn main() {
    let mut input = FastInput::new(stdin().lock());
    bind! { input >> n: usize, }

}
