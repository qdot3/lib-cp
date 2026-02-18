/// 座標圧縮後の配列と復元用の配列を返す。
///
/// # Time Complexity
///
/// *Θ*(*N*)
pub fn compress(values: &[usize]) -> (Vec<usize>, Vec<usize>) {
    if values.is_empty() {
        // アロケーションは遅延されるので、悪影響はない
        return (Vec::new(), Vec::new());
    }

    // １バイトごとに基数ソート
    const MASK: usize = 0b1111_1111;
    let mut bucket = [0; 256];
    let mut result1 = Vec::from_iter(0..values.len());
    let mut result2 = vec![0; values.len()];
    for shift in (0..usize::BITS).step_by(8) {
        bucket.fill(0);
        for i in result1.iter() {
            bucket[(values[*i] >> shift) & MASK] += 1
        }

        // バケットの左端を計算
        let mut sum = 0;
        bucket.iter_mut().for_each(|n| {
            sum += *n;
            *n = sum - *n
        });

        // 安定ソート
        for i in result1.iter() {
            let j = (values[*i] >> shift) & MASK;
            result2[bucket[j]] = *i;
            bucket[j] += 1
        }

        std::mem::swap(&mut result1, &mut result2);
    }

    // 座標圧縮。隣接二項を比較する。
    result2[result1[0]] = 0;
    result1
        .windows(2)
        .for_each(|i| result2[i[1]] = result2[i[0]] + (values[i[0]] != values[i[1]]) as usize);
    let compressed = result2;

    result1.iter_mut().for_each(|i| *i = values[*i]);
    result1.dedup();
    let restore = result1;

    (compressed, restore)
}
