/// 文字列`S`中の回文をすべて求める。
///
/// 返り値を`len`とすると、
///
/// - `len[2i]`は`S[i]`を中心とする奇数長の回文の長さの最大値
/// - `len[2i+1]`は`S[i..i+2]`を中心とする偶数長の回文の長さの最大値
///
/// にそれぞれ対応している。
pub fn manacher<T: Eq>(str: &[T]) -> Vec<usize> {
    // str は [s[0], #, s[1], ..., #] と解釈される
    let mut radius = Vec::with_capacity(str.len() * 2);
    radius.push(1);

    // 開区間 (l, r) が回文（1-based indexing）
    let mut l = 1;
    let mut r = 2;
    for i in 2..str.len() * 2 {
        // 計算済みの半径を再利用する。未調査の区間があればカットする。
        // c+r >= i より、アンダーフローしない
        let mut ri = (r - i).min(radius[l + (r - i) - 1]).max(1);

        // 未調査の区間まで回文を延長する。
        // i ± ri が偶数のときは区切り文字なので一致し、奇数のときは文字を比較する。
        // 偶数の場合は内側に丸めればよい。
        ri = (ri..i)
            .find(|ri| str.get((i - ri) / 2) != str.get((i + ri - 1) / 2))
            .unwrap_or(i);

        radius.push(ri);
        if i + ri > r {
            (l, r) = (i - ri, i + ri)
        }
    }

    // 先頭に区切り文字`#`を追加すると、長さ 2r-1 の回文のうち栗切り文字は r 個ある。
    // 区切り文字を除いた回文の長さは、r-1 である。
    for i in 0..radius.len() {
        if radius[i] != i + 1 {
            radius[i] -= 1
        }
    }

    radius
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{self, Rng};

    fn assert<T: Eq>(str: &[T]) {
        let brute_force = |str: &[T]| {
            let mut res = Vec::with_capacity(str.len() * 2);
            for i in 0..str.len() {
                let odd_r = (0..=i)
                    .take_while(|r| str.get(i - r) == str.get(i + r))
                    .count();
                res.push(odd_r * 2 - 1);

                let even_r = (0..=i)
                    .take_while(|r| str.get(i - r) == str.get(i + r + 1))
                    .count();
                res.push(even_r * 2);
            }
            res.pop();

            res
        };

        assert_eq!(manacher(str), brute_force(str))
    }

    #[test]
    fn random() {
        let mut rng = rand::rng();
        for _ in 0..100 {
            let s: Vec<u8> = (0..500).map(|_| rng.random_range(0..4)).collect();

            assert(&s);
        }
    }
}
