/// 文字列 S と接尾辞 S[i..] の最長共通接頭辞の長さをもとめる。
///
/// # Time Complexity
///
/// *Θ*(*N*)
pub fn z_algorithm<T: Eq>(str: &[T]) -> Vec<usize> {
    let mut z = Vec::with_capacity(str.len());

    // プレフィックスと一致する部分文字列の内、最も右側まで続くもの。
    // 右端は広義単調増加なので、比較の回数は高々 2N 回
    let mut match_r = 0..0;
    // match_r は探索済みでなければならないので、
    z.push(str.len());
    for i in 1..str.len() {
        // 計算済みの結果を再利用。共通接頭辞の下界が決まる。
        z.push(if i < match_r.end {
            debug_assert!(match_r.contains(&i));
            z[i - match_r.start].min(match_r.end - i)
        } else {
            0
        });

        // 末尾も一致しているかも
        while i + z[i] < str.len() && str[i + z[i]] == str[z[i]] {
            z[i] += 1;
        }
        if i + z[i] > match_r.end {
            match_r = i..i + z[i]
        }
    }

    z
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use crate::z_algorithm;

    fn brute_force<T: Eq>(str: &[T]) -> Vec<usize> {
        let mut lcp = Vec::with_capacity(str.len());
        for i in 0..str.len() {
            lcp.push(
                str.iter()
                    .zip(str[i..].iter())
                    .take_while(|(a, b)| a == b)
                    .count(),
            );
        }

        lcp
    }

    #[test]
    fn random() {
        let mut rng = rand::rng();
        for n in 400..600 {
            let str = Vec::from_iter((0..n).map(|_| rng.random_range(0u8..4)));

            assert_eq!(z_algorithm(&str), brute_force(&str))
        }
    }

    #[test]
    fn handmade() {
        let str = b"abcdabcaba";
        let lcp = vec![10, 0, 0, 0, 3, 0, 0, 2, 0, 1];

        assert_eq!(z_algorithm(str), lcp)
    }
}
