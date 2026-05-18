use std::io::{stdin, stdout, BufWriter, Write};

use output::IntBuffer;
use reader::FastBufReader;

fn main() {
    let mut input = FastBufReader::<{ 1 << 16 }, _>::new(stdin().lock());
    let mut output = BufWriter::with_capacity(1 << 18, stdout().lock());
    let mut buf = IntBuffer::new();

    let t: usize = input.parse_next_token().unwrap();
    for _ in 0..t {
        let a: u64 = input.parse_next_token().unwrap();
        let b: u64 = input.parse_next_token().unwrap();

        output.write(buf.format(a + b).as_bytes()).unwrap();
        output.write(b"\n").unwrap();
    }
}
