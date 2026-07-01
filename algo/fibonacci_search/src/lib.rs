/// Returns the index of maximum node.
///
/// # Precondition
///
/// `src` must be strictly unimodal by `key`.
///
/// # Time complexity
///
/// O(log N)
pub fn fibonacci_search_by_key<T, U, F>(src: &[T], mut key: F) -> Option<usize>
where
    F: FnMut(&T) -> U,
    U: Ord,
{
    let mut f = [1, 2, 3];
    while f[2] < src.len() {
        f = [f[1], f[2], f[1] + f[2]];
    }

    let mut cached_key = [const { None }; 4];
    let mut offset = 0;
    while f[2] > 3 {
        src.get(offset + f[0])
            .map(|v| cached_key[1].get_or_insert(key(v)));
        src.get(offset + f[1])
            .map(|v| cached_key[2].get_or_insert(key(v)));

        if cached_key[1] < cached_key[2] {
            offset += f[1];

            cached_key[0] = cached_key[1].take();
            cached_key[1] = cached_key[2].take();
            cached_key[2].take();
        } else {
            cached_key[3] = cached_key[2].take();
            cached_key[2] = cached_key[1].take();
            cached_key[1].take();
        }

        f = [f[1] - f[0], f[0], f[1]];
    }

    for i in 0..4 {
        src.get(offset + i)
            .map(|v| cached_key[i].get_or_insert(key(v)));
    }
    (0..src.len() - offset)
        .max_by_key(|&i| &cached_key[i])
        .map(|i| i + offset)
}
