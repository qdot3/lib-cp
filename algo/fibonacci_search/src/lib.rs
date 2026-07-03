use std::ops::Range;

/// Returns the maximum value and the corresponding parameter.
///
/// # Precondition
///
/// `key` must be strictly quasiconvex over the `range`.
///
/// # Time complexity
///
/// O(log B) where B is `usize::BITS`
pub fn fibonacci_search_by_key<F, K>(range: Range<usize>, mut key: F) -> Option<(usize, K)>
where
    F: FnMut(usize) -> K,
    K: Ord,
{
    let mut offset = range.start;

    let mut fib = [1, 2, 3];
    while fib[2] < range.end - range.start {
        if let Some(sum) = fib[1].checked_add(fib[2]) {
            fib = [fib[1], fib[2], sum];
        } else {
            break;
        }
    }

    let mut cached_key = [const { None }; 4];
    while fib[2] > 3 {
        if let Some(i) = offset.checked_add(fib[1]) {
            cached_key[1].get_or_insert_with(|| key(i));
        }
        if let Some(i) = offset.checked_add(fib[2]) {
            cached_key[2].get_or_insert_with(|| key(i));
        }

        if cached_key[1] < cached_key[2] {
            offset += fib[1];

            cached_key[0] = cached_key[1].take();
            cached_key[1] = cached_key[2].take();
            cached_key[2].take();
        } else {
            cached_key[3] = cached_key[2].take();
            cached_key[2] = cached_key[1].take();
            cached_key[1].take();
        }

        fib = [fib[1] - fib[0], fib[0], fib[1]];
    }

    std::iter::zip(offset..range.end, cached_key)
        .map(|(i, v)| (i, v.unwrap_or_else(|| key(i))))
        .max_by(|a, b| a.1.cmp(&b.1))
}

/// # Precondition
pub fn golden_section_search_by_key<F, K>(range: Range<f64>, key: F)
where
    F: FnMut(f64) -> K,
    K: Ord,
{
    let Range { start, end } = range;

    // reject invalid range
    if start.is_nan() || end.is_nan() {
        todo!("return None")
    }

    const SHIFT: usize = std::mem::size_of::<f64>() * 8 - 1;

    // convert `f64` to `i64`, keeping the order
    let f2i = |f: f64| {
        let mut i = f.to_bits().cast_signed();
        i ^= ((i >> SHIFT).cast_unsigned() >> 1).cast_signed();
        i
    };
    // inverse of `f2i`
    let i2f = |mut i: i64| {
        i ^= ((i >> SHIFT).cast_unsigned() >> 1).cast_signed();
        f64::from_bits(i.cast_unsigned())
    };

    todo!("fibonacci search on i64")
}
