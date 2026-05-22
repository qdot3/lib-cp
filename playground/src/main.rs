use std::io::stdin;

use output::IntBuffer;
use proconio::fastout;
use reader::FastBufReader;

#[fastout]
fn main() {
    let mut input = FastBufReader::<{ 1 << 16 }, _>::new(stdin().lock());
    let mut buf = IntBuffer::new();

    let t: usize = input.parse_next_token().unwrap();
}
