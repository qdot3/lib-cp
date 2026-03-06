/// 座標圧縮後の配列と復元用の配列を返す。
///
/// # Time Complexity
///
/// *Θ*(*N*)
pub fn compress<const MIN: u32>(text: &[u32]) -> (Vec<u32>, Vec<u32>) {
    if std::mem::size_of::<usize>() > 4 {
        assert!(text.len() <= (1 << 32));
    }

    // step 1. radix sort
    let (sorted, buf) = {
        // assume the size of TLB >= 2^8
        const B: u32 = 8;
        const MASK: u32 = (1 << B) - 1;

        let mut bucket = Box::new([0; 4 << B]);
        for v in text.iter() {
            let v = v.to_le_bytes();
            bucket[(v[0] as usize) | (0 << B)] += 1;
            bucket[(v[1] as usize) | (1 << B)] += 1;
            bucket[(v[2] as usize) | (2 << B)] += 1;
            bucket[(v[3] as usize) | (3 << B)] += 1;
        }

        // keep locality to reduce TLB and cache misses
        let mut buf1 = Vec::from_iter(text.iter().enumerate().map(|(i, v)| [i as u32, *v]));
        let mut buf2 = vec![[0; 2]; buf1.len()];
        let mut shift = 0;
        for bucket in bucket.chunks_mut(1 << B) {
            // calculate left-free pointer for each bucket
            let mut sum = 0;
            bucket.iter_mut().for_each(|n| {
                sum += *n;
                *n = sum - *n;
            });

            for [i, v] in buf1.iter() {
                let j = ((v >> shift) & MASK) as usize;

                buf2[bucket[j] as usize] = [*i, *v];
                bucket[j] += 1;
            }

            std::mem::swap(&mut buf1, &mut buf2);
            shift += B;
        }

        (buf1, buf2)
    };

    // step 2. rename characters
    let mut compressed = buf.into_flattened();
    compressed.truncate(text.len());

    if let Some([i, _]) = sorted.first() {
        compressed[*i as usize] = MIN;
    }
    for iv in sorted.windows(2) {
        compressed[iv[1][0] as usize] =
            compressed[iv[0][0] as usize] + (iv[1][1] != iv[0][1]) as u32
    }

    // step3. make map to restore character
    let mut restore = sorted;
    restore.dedup_by_key(|iv| iv[1]);
    let mut restore = restore.into_flattened();
    for i in (1..restore.len()).step_by(2) {
        restore[i / 2] = restore[i];
    }
    restore.truncate(restore.len() / 2);

    (compressed, restore)
}
