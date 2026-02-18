pub trait RadixSort {
    fn radix_sort(&mut self);
}

macro_rules! radix_sort_impl_uint {
    ($( $t:ty )*) => {$(
        impl RadixSort for &mut [$t] {
            /// # Time Complexity
            ///
            /// *Θ*(*N*)
            fn radix_sort(&mut self) {
                if let Some(max) = self.iter().max().copied() {
                    // １バイトごとに基数ソート
                    const MASK: $t = 0xff;
                    let mut bucket = [0; 256];
                    let mut buf = Vec::new();
                    for shift in (0..u32::BITS - max.leading_zeros()).step_by(8) {
                        // バケットの左端を求める
                        for i in self.iter() {
                            let i = (i >> shift) & MASK;
                            bucket[i as usize] += 1;
                        }
                        let mut sum = 0;
                        bucket.iter_mut().for_each(|n| {
                            sum += *n;
                            *n = sum - *n;
                        });

                        // 安定ソート
                        buf.extend_from_slice(&self);
                        for i in buf.drain(..) {
                            let j = (i >> shift) & MASK;
                            self[bucket[j as usize]] = i;
                            bucket[j as usize] += 1;
                        }

                        bucket.fill(0);
                    }
                }
            }
        }
    )*};
}

radix_sort_impl_uint!( u8 u16 u32 u64 u128 usize );