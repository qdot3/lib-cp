use std::io::stdin;

use input::{bind, FastInput};
use proconio::fastout;

#[fastout]
fn main() {
    let mut input: FastInput<std::io::StdinLock<'_>> = FastInput::new(stdin().lock());
    bind! { input >> n: usize, mut a: [i32; n], }

    let a = {
        let sum: i32 = a.iter().sum();
        let n = n as i32;
        if sum % n != 0 {
            println!("-1");
            return;
        }
        let average = sum / n;
        a.iter_mut().for_each(|a| *a -= average);
        a
    };
    #[cfg(debug_assertions)]
    println!("{:?}", a);

    let mut dp = vec![(i32::MIN, i32::MIN); 1 << n];
    dp[0] = (0, 0);
    for set in 0..1 << n {
        let mut next: usize = !set & ((1 << n) - 1);
        while next != 0 {
            let i = next.trailing_zeros() as usize;
            next ^= 1 << i;

            dp[set | (1 << i)] = dp[set | (1 << i)].max(if dp[set].1 + a[i] == 0 {
                (dp[set].0 + 1, 0)
            } else {
                (dp[set].0, dp[set].1 + a[i])
            });
        }
    }
    #[cfg(debug_assertions)]
    println!("{:?}", dp);

    const PARTITION: usize = !0;
    let mut order = Vec::with_capacity(n * 2);
    let mut set = (1 << n) - 1;
    for mut subset in 1..(1 << n) {
        if set & subset == subset && dp[subset] == (1, 0) && dp[set ^ subset] == (dp[set].0 - 1, 0)
        {
            set ^= subset;
            while subset > 0 {
                let i = subset.trailing_zeros() as usize;
                subset ^= 1 << i;
                order.push(i);
            }
            order.push(PARTITION);
        }
    }
    #[cfg(debug_assertions)]
    println!("{:?}", order);

    let cost = order
        .split(|i| *i == PARTITION)
        .fold(0, |acc, v| acc + v.len().saturating_sub(1));
    println!("{}", cost);
    let mut a = a;
    for group in order.split_mut(|i| *i == PARTITION) {
        group.sort_unstable_by_key(|i| !a[*i]);
        for pair in group.windows(2) {
            let (i, j) = (pair[0], pair[1]);
            println!("{} {} {}", i + 1, j + 1, a[i]);
            a[j] += a[i]
        }
    }
}
