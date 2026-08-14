use std::{io::stdin, num::NonZero};

use min_prime_factor::MinPrimeFactor;
use num_integer::Integer;
use output::IntBuffer;
use proconio::fastout;
use reader::FastBufReader;

#[fastout]
fn main() {
    let mut input = FastBufReader::<{ 1 << 16 }, _>::new(stdin().lock());
    let mut buf = IntBuffer::new();

    let n: usize = input.parse_next_token().unwrap();
    let s = input.parse_next_token_vec::<i64>(n).unwrap();

    let mut max_score = 0;

    let step_sum = {
        let mut buf = Vec::with_capacity(n.isqrt() + 1);
        buf.push(Vec::new());

        for step in 1..=n.isqrt() {
            buf.push(s.clone());
            for i in (step..s.len()).rev() {
                buf[step][i - step] += buf[step][i]
            }
        }

        buf
    };

    let mpf = MinPrimeFactor::new(n as u32);
    let mut divisor = Vec::with_capacity(n.isqrt() * 2);
    for b in 1..n - 1 {
        mpf.append_divisors(NonZero::new((n - b - 1) as u32).unwrap(), &mut divisor);

        for d in divisor.drain(..).map(|d| d as usize) {
            let a = b + d;
            if a >= n - 1 {
                continue;
            }

            let (div, rem) = a.div_rem(&d);
            if rem == 0 && div * d + b < n - 1 {
                continue;
            }

            println!("{} {}", a, b);

            if d < step_sum.len() {
                let score = step_sum[d][0] + step_sum[d][a];
                max_score = max_score.max(score)
            } else {
                let mut score = 0;
                let mut i = 0;
                loop {
                    i += a;
                    score += s[i] as i64;
                    if i == n - 1 {
                        break;
                    }

                    i -= b;
                    score += s[i] as i64;
                    if i == n - 1 {
                        break;
                    }
                }
                max_score = max_score.max(score)
            }
        }
    }

    println!("{}", buf.format(max_score))
}
