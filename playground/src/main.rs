use proconio::{fastout, input};

#[fastout]
fn main() {
    input! { t: usize, }

    for _ in 0..t {
        input! { mut n: usize, mut wp: [(u64, u64); n], }
        wp.sort_unstable_by_key(|(w, p)| w + p);

        let mut sum_w = wp.iter().fold(0, |acc, (w, _)| acc + w);
        let mut sum_p = 0;
        for (w, p) in wp.into_iter().rev() {
            sum_w -= w;
            sum_p += p;
            n -= 1;
            if sum_p >= sum_w {
                break;
            }
        }
        println!("{}", n)
    }
}
