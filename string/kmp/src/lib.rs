/// すべての部分文字列 `S[0..=i]` について、その接頭辞かつ接尾辞であるような真の部分文字列の長さの最大値をもとめる。
///
/// TODO: online 化できる
///
/// # Time Complexity
///
/// *Θ*(*N*)
pub fn kmp<T: Eq>(str: &[T]) -> Vec<usize> {
    let mut len = Vec::with_capacity(str.len());
    len.push(0);
    for i in 1..str.len() {
        let mut n = len[i - 1];
        // 共通部分列の長さは高々 n+1 である。末尾が一致しないなら最長共通部分列はより短い。
        while n > 0 && str[i] != str[n] {
            // str の接頭辞で考えればよい。
            // n は高々 N 増加し、ここで１以上小さくなるので、計算量は Θ(N)
            n = len[n - 1];
        }
        // 追加された１文字を確認
        if str[i] == str[n] {
            n += 1;
        }
        len.push(n);
    }

    len
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use crate::kmp;

    #[test]
    fn handmade() {
        let len = kmp(b"abc abcd abc ");
        assert_eq!(len, vec![0, 0, 0, 0, 1, 2, 3, 0, 0, 1, 2, 3, 4])
    }

    /// *Θ*(*N*^3)
    fn brute_force<T: Eq>(str: &[T]) -> Vec<usize> {
        let mut res = Vec::with_capacity(str.len());
        for i in 0..str.len() {
            let x = (1..=i)
                .rfind(|&len| str[..len] == str[i - len + 1..=i])
                .unwrap_or(0);
            res.push(x);
        }
        res
    }

    #[test]
    fn random() {
        let mut rng = rand::rng();
        for n in 50..100 {
            let str = Vec::from_iter((0..n).map(|_| rng.random_range(0u8..4)));

            assert_eq!(kmp(&str), brute_force(&str))
        }
    }
}
