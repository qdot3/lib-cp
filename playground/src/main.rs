use std::io::stdin;

use input::{bind, FastInput};
use output::IntBuffer;
use proconio::fastout;

#[fastout]
fn main() {
    let mut input: FastInput<std::io::StdinLock<'_>> = FastInput::new(stdin().lock());
    let mut buf = IntBuffer::new();

    bind! { input >> n: usize, mut tx: [(i32, i32); n], }

    tx.iter_mut()
        .for_each(|(t, x)| (*t, *x) = (*x + *t, *x - *t));
    tx.sort_unstable_by_key(|(x, y)| (*x, -y));

    let mut robot = Vec::with_capacity(n);
    for (_, y) in tx {
        let i = robot.partition_point(|v| *v < y);
        if let Some(v) = robot.get_mut(i) {
            *v = y
        } else {
            robot.push(y);
        }
    }
    println!("{}", buf.format(robot.len()))
}
