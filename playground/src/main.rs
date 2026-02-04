use std::io::stdin;

use input::{bind, FastInput};
use itertools::Itertools;
use proconio::fastout;

#[fastout]
fn main() {
    let mut input = FastInput::new(stdin().lock());
    bind! { input >> t: usize, }

    let mut coin = Vec::new();
    let mut digits = Vec::new();
    for _ in 0..t {
        bind! { input >> n: usize, m: u64, }

        coin.reserve(n + 10);
        for _ in 0..n {
            bind! { input >> a: u64, }
            coin.push(a);
        }
        coin.extend_from_slice(&[0; 10]);

        for i in (0..coin.len()).rev() {
            let max = coin[i.saturating_sub(10)..=i]
                .iter()
                .fold(0, |acc, c| acc / 10 + c);
            let max = max / m * m;

            if max <= coin[i] {
                coin[i] /= m;
                continue;
            }

            let mut need = max - coin[i];
            coin[i] = max / m;
            for c in coin[..i].iter_mut().rev() {
                need *= 10;

                if *c >= need {
                    *c -= need;
                    break;
                } else {
                    need -= *c;
                    *c = 0;
                }
            }
        }
        #[cfg(debug_assertions)]
        println!("{:?}", coin);

        digits.reserve(coin.len() + 100);
        let mut carry = 0;
        for coin in coin.drain(..) {
            carry += coin;

            digits.push(carry % 10);
            carry /= 10;
        }
        while carry > 0 {
            digits.push(carry % 10);
            carry /= 10;
        }
        while digits.pop_if(|d| *d == 0).is_some() {}

        if digits.is_empty() {
            println!("0")
        } else {
            println!("{}", digits.drain(..).rev().join(""))
        }
    }
}
