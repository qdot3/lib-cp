use std::io::stdin;

use output::IntBuffer;
use proconio::fastout;
use reader::FastBufReader;
use rustc_hash::FxHashSet;

#[fastout]
fn main() {
    let mut input = FastBufReader::<{ 1 << 16 }, _>::new(stdin().lock());
    let mut buf = IntBuffer::new();

    let n: usize = input.parse_next_token().unwrap();
}
