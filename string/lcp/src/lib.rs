/// 接尾辞配列からLCP配列を計算する。
/// 返り値を`lcp`とかくと、`lcp[i]`は`text[i..]`とその次に小さな接尾辞のLCPの長さになっている。
/// とくに、`lcp[0] = 0`である。
///
/// # Complexity
///
/// - *Θ*(*N*) in time
/// - *O*(1) in working space
pub fn lcp_array<T: Eq>(text: &[T], sa: &[usize]) -> Vec<usize> {
    let mut lcp = Vec::with_capacity(sa.len());
    {
        let lcp = lcp.spare_capacity_mut();
        (0..sa.len()).for_each(|i| {
            lcp[sa[i]].write(i);
        });
    }
    // 接尾辞配列での順序
    unsafe { lcp.set_len(sa.len()) };

    // LCPの長さの下限
    let mut l = 0;
    for i in 0..sa.len() {
        // 次に小さな接尾辞が存在しないので、LCPは 0 でよい。
        if lcp[i] > 0 {
            let j = sa[lcp[i] - 1];
            // 少なくとも一方は Some を返す。
            while text.get(i + l) == text.get(j + l) {
                l += 1
            }
            // 不要なのでLCPの長さを書き込む
            lcp[i] = l;

            l = l.saturating_sub(1)
        }
    }

    lcp
}
