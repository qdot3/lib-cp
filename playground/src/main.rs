use std::io::{stdin, stdout, BufWriter, Write};

use input::FastInput;

fn main() {
    let mut input = FastInput::new(stdin().lock());
    let mut output = BufWriter::new(stdout().lock());

    let t: usize = input.next_token().unwrap();
    for _ in 0..t {
        let a: u64 = input.next_token().unwrap();
        let b: u64 = input.next_token().unwrap();

        let _ = writeln!(&mut output, "{}", a + b);
    }
}
