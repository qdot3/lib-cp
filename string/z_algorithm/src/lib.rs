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
