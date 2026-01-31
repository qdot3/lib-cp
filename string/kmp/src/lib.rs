/// 文字列 `S` の接頭辞と `S[1..i]` の接尾辞の最長共通連続部分の長さを計算する。
///
/// # Time Complexity
///
/// *Θ*(*N*)
// TODO: online 化できる
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
    use super::kmp;

    #[test]
    fn test() {
        let len = kmp(b"abc abcd abcd");
        assert_eq!(len, vec![0, 0, 0, 0, 1, 2, 3, 0, 0, 1, 2, 3, 0])
    }
}
