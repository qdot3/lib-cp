use std::io::{stdin, stdout, BufWriter};

use input::{bind, FastInput};
use output::IntBuffer;

fn main() {
    let mut input: FastInput<std::io::StdinLock<'_>> = FastInput::new(stdin().lock());
    let mut output = BufWriter::with_capacity(1 << 16, stdout().lock());
    let mut buf = IntBuffer::new();

    bind! { input >> n: usize, }
}
