use proconio::{fastout, input};

#[fastout]
fn main() {
    input! { t: usize, }

    let mut stack = Vec::new();
    'trial: for _ in 0..t {
        input! { n: usize, p: [u32; n], }

        // 最適であるための必要条件
        if p[0] != n as u32 {
            println!("No");
            continue;
        }

        // 木の存在
        {
            stack.reserve(n);
            let mut min_p = p[0];
            for p in p.iter().skip(1).copied() {
                if p < min_p {
                    stack.extend((p + 1..min_p).rev());
                    min_p = p;
                } else if !stack.pop().is_some_and(|v| v == p) {
                    println!("No");

                    stack.clear();
                    continue 'trial;
                }
            }
        };

        // 最適か？

        println!("Yes");
        assert!(stack.is_empty());
    }
}
