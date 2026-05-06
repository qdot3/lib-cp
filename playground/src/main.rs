use std::io::{stdin, stdout, BufWriter, Write};

use input::{bind, FastInput};
use output::IntBuffer;

fn main() {
    let mut input: FastInput<std::io::StdinLock<'_>> = FastInput::new(stdin().lock());
    bind! { input >> t: usize, }

    let mut output = BufWriter::with_capacity(1 << 16, stdout().lock());
    let mut buf = IntBuffer::new();

    for _ in 0..t {
        bind! { input >> a: u64, b: u64, }

        output.write(buf.format(a + b).as_bytes()).unwrap();
        output.write(b"\n").unwrap();
    }
}
