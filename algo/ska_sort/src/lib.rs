pub fn ska_sort_by_key<T, F, K>(values: &mut [T], f: F)
where
    F: Fn(&T) -> K,
    K: RadixKey,
{
    ska_sort_rec(values, &f, K::MAX_LEVEL);
}

fn ska_sort_rec<T, F, K>(values: &mut [T], f: &F, level: usize)
where
    F: Fn(&T) -> K,

    K: RadixKey,
{
    if values.len() < K::FALLBACK_THRESHOLD_UNSTABLE {
        values.sort_unstable_by_key(|v| f(v).full_key());
        return;
    }

    let (end, largest) = {
        let mut hist = [0; 256];
        for v in values.iter() {
            hist[f(v).extract_key(level) as usize] += 1
        }
        let largest = (0..256).max_by_key(|i| hist[*i]).unwrap();

        for i in 1..256 {
            hist[i] += hist[i - 1]
        }

        (hist, largest)
    };

    {
        let mut next = end;
        next.rotate_right(1);
        next[0] = 0;

        // MSB radix sort
        for b in 0..256 {
            if b == largest {
                continue;
            }

            let mut i = next[b];
            while i < end[b] {
                let key = f(&values[i]).extract_key(level) as usize;

                if key == b {
                    i += 1;
                } else {
                    values.swap(i, next[key]);
                    next[key] += 1;
                }
            }
        }
    }

    if level > 0 {
        let mut rest = values;

        let mut start = 0;
        for i in 0..256 {
            let (pre, suf) = rest.split_at_mut(end[i] - start);
            rest = suf;
            start = end[i];

            if pre.len() > 1 {
                ska_sort_rec(pre, f, level - 1);
            }
        }
    }
}

pub trait RadixKey: Sized {
    const MAX_LEVEL: usize = { std::mem::size_of::<Self>().checked_sub(1).unwrap() };
    const FALLBACK_THRESHOLD_UNSTABLE: usize = { 1 << 10 };

    type Key: Ord;

    fn extract_key(&self, level: usize) -> u8;
    fn full_key(&self) -> Self::Key;
}

macro_rules! impl_radix_key_signed {
    ($( $t:ty )*) => {$(
        impl RadixKey for $t {
            type Key = $t;

            fn extract_key(&self, level: usize) -> u8 {
                let filter = const {
                    (1 as $t).cast_unsigned().rotate_right(1)
                };
                let key = (self.cast_unsigned() ^ filter) >> (level * 8);
                key as u8
            }

            fn full_key(&self) -> Self::Key {
                *self
            }
        }
    )*};
}
impl_radix_key_signed!( i8 i16 i32 i64 i128 isize );
