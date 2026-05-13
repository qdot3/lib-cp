use std::io::stdin;

use input::{bind, FastInput};
use output::IntBuffer;
use proconio::fastout;

#[fastout]
fn main() {
    let mut input: FastInput<std::io::StdinLock<'_>> = FastInput::new(stdin().lock());
    let mut buf = IntBuffer::new();

    bind! { input >> n: usize, }

}
