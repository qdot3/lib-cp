/// `Σ_{0 <= i < n} ⌊ (a i + b) / m ⌋`を計算する
///
/// # Time Complexity
///
/// *O*(log )
pub fn floor_sum(mut n: i64, mut m: i64, mut a: i64, mut b: i64) -> i64 {
    assert_ne!(m, 0);
    assert!(n >= 0);

    let mut res = 0;
    loop {
        // f(n, m, a, b) = n(n-1)/2 * d_a + n d_b + f(n, m, r_a, r_b)
        let (div_a, rem_a) = (a.div_euclid(m), a.rem_euclid(m));
        let (div_b, rem_b) = (b.div_euclid(m), b.rem_euclid(m));
        res += n * (n - 1) / 2 * div_a + n * div_b;

        // f(n, m, 0, r_b) = 0
        if rem_a == 0 {
            break res;
        }

        // k := ⌊ (r_a(n-1) + b) / m ⌋ <= n-1（床関数の最大値）
        // f(n, m, r_a, r_b) = n k + f(k, r_a, m, r_b - m k)
        let k = (rem_a * (n - 1) + rem_b).div_euclid(m);
        res += n * k;
        (n, m, a, b) = (k, rem_a, m, rem_b - m * k)
    }
}
