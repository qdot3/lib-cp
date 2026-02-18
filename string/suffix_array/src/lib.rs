// since max heap capacity is isize::MAX, msb of usize can be used for a counter flag.
// these constants are used for number of appearance of LMS-characters for each buckets
const COUNTER_FILTER: usize = 1usize.rotate_right(1);
// counters never overflow since `COUNT_ZERO + isize::MAX = usize::MAX`
const COUNT_ZERO: usize = COUNTER_FILTER;
const COUNT_ONE: usize = COUNT_ZERO + 1;

#[inline]
fn rename(str: &mut [usize], sa: &mut [usize]) {
    debug_assert_eq!(str.len(), sa.len());

    sa.fill(0);
    str.iter_mut().for_each(|s| sa[*s] += 1);
    (1..sa.len()).for_each(|i| sa[i] += sa[i - 1]);
    // now sa[i] is the right exclusive partition of i's bucket.
    // rename all i as sa[i]-1(sa[i-1]) if i is S(L)-type.
    {
        // penultimate is L-type
        let mut is_s_type = false;
        // sentinel will be 0
        let mut name = [0; 2];
        for i in (0..str.len()).rev().skip(1) {
            if str[i] != str[i + 1] {
                is_s_type = str[i] < str[i + 1]
            }
            if let Some(s) = str.get_mut(i + 2) {
                *s = name[i % 2]
            }
            name[i % 2] = if is_s_type {
                sa[str[i]] - 1
            } else {
                // we can use msb of usize as a type flag
                sa[str[i] - 1] | 1usize.rotate_right(1)
            };
        }
        str[..2].copy_from_slice(&name);
    }
}

#[inline]
fn str_to_ptr(s: usize) -> usize {
    // if msb is 0, then RF-pointer for S-type. otherwise, LF-pointer for L-type
    const MASK: usize = !0 >> 1;
    s & MASK
}

#[inline]
fn is_s_type(s: usize) -> bool {
    const MASK: usize = !0 >> 1;
    s & !MASK == 0
}

#[inline]
fn sort_lms_chars(str: &[usize], sa: &mut [usize]) {
    // count LMS-characters
    sa.fill(COUNT_ZERO);
    str.windows(2).for_each(|s| {
        if !is_s_type(s[0]) && is_s_type(s[1]) {
            sa[str_to_ptr(s[1])] += 1
        }
    });
    // put LMS-characters
    str.windows(2).enumerate().for_each(|(i, s)| {
        // s[1] is LMS-type
        if !is_s_type(s[0]) && is_s_type(s[1]) {
            let ptr = str_to_ptr(s[1]);
            debug_assert!(sa[ptr] >= COUNT_ONE);

            if sa[ptr] == COUNT_ONE {
                sa[ptr] = i + 1
            } else if sa[ptr - 1] == COUNT_ZERO {
                // first LMS-character in the bucket
                sa[ptr - 1] = i + 1;
                sa[ptr] = COUNT_ZERO + 2;
            } else {
                let c = sa[ptr] - COUNT_ZERO;
                if sa
                    .get(ptr.wrapping_sub(c))
                    .is_some_and(|sa| *sa == COUNT_ZERO)
                {
                    sa[ptr - c] = i + 1;
                    sa[ptr] += 1;
                } else {
                    // last LMS-character in the bucket
                    sa[ptr] = i + 1;
                }
            }
        }
    });

    // remove counter and shift LMC-characters right by 1
    let mut i = sa.len();
    while i > 0 {
        i -= 1;
        if sa[i] > COUNT_ONE {
            // counter indicates next free position
            let c = sa[i] - COUNT_ZERO - 1;
            sa[i] = sa[i - c];
            sa[i - c] = COUNT_ZERO;

            i -= c;
        }
    }
}

/// ソート済みのLMS型からL型をソートする
#[inline]
fn induce_l_type(str: &[usize], sa: &mut [usize], remove_counter: bool) {
    // count L-types
    str.iter().for_each(|s| {
        if !is_s_type(*s) {
            sa[str_to_ptr(*s)] += 1
        }
    });
    // put L-suffixes
    let mut i = 0;
    while i < sa.len() {
        if sa[i] > 0 && sa[i] & COUNTER_FILTER == 0 {
            let s = str[sa[i] - 1];
            let ptr = str_to_ptr(s);
            if !is_s_type(s) && sa[ptr] & COUNTER_FILTER > 0 {
                if sa[ptr] == COUNT_ONE {
                    sa[ptr] = sa[i] - 1;
                } else if sa[ptr + 1] == COUNT_ZERO {
                    // first L-character in the bucket
                    sa[ptr + 1] = sa[i] - 1;
                    sa[ptr] = COUNT_ZERO + 2;
                } else {
                    let c = sa[ptr] - COUNT_ZERO;
                    if sa.get(ptr + c).is_some_and(|sa| *sa == COUNT_ZERO) {
                        sa[ptr + c] = sa[i] - 1;
                        sa[ptr] += 1
                    } else {
                        // the bucket is filled
                        sa[ptr] = sa[i] - 1;
                        sa[ptr..ptr + c].rotate_left(1);
                        // current bucket may be rotated
                        continue;
                    }
                }
            }
        }
        i += 1;
    }

    // remove counter and shift L-suffixes left by 1
    if remove_counter {
        i = 0;
        while i < sa.len() {
            if sa[i] > COUNT_ZERO {
                // counter indicates next free position
                let c = sa[i] - COUNT_ZERO;
                sa[i] = COUNT_ZERO;
                sa[i..i + c].rotate_left(1);

                i += c
            } else {
                i += 1
            }
        }
    }
}

/// ソート済みのLML型からS型をソートする
#[inline]
fn induce_s_type(str: &[usize], sa: &mut [usize], remove_counter: bool) {
    // count S-suffixes except for the sentinel
    str.iter().rev().skip(1).for_each(|s| {
        if is_s_type(*s) {
            sa[str_to_ptr(*s)] += 1
        }
    });
    // put S-suffixes
    let mut i = sa.len();
    while i > 0 {
        i -= 1;
        if sa[i] > 0 && sa[i] & COUNTER_FILTER == 0 {
            let s = str[sa[i] - 1];
            let ptr = str_to_ptr(s);
            if is_s_type(s) && sa[ptr] & COUNTER_FILTER > 0 {
                if sa[ptr] == COUNT_ONE {
                    sa[ptr] = sa[i] - 1;
                } else if sa[ptr - 1] == COUNT_ZERO {
                    // first L-character in the bucket
                    sa[ptr - 1] = sa[i] - 1;
                    sa[ptr] = COUNT_ZERO + 2;
                } else {
                    let c = sa[ptr] - COUNT_ZERO;
                    if sa
                        .get(ptr.wrapping_sub(c))
                        .is_some_and(|sa| *sa == COUNT_ZERO)
                    {
                        sa[ptr - c] = sa[i] - 1;
                        sa[ptr] += 1
                    } else {
                        // last L-character in the bucket
                        sa[ptr] = sa[i] - 1;
                        sa[ptr + 1 - c..=ptr].rotate_right(1);
                        // current bucket may be rotated
                        i += 1;
                        continue;
                    }
                }
            }
        }
    }

    if remove_counter {
        todo!()
    }
}

#[inline]
fn induced_sort_lms(str: &[usize], sa: &mut [usize]) {
    induce_l_type(str, sa, true);

    // remove LMS-substrings except for the sentinel
    sa.iter_mut().skip(1).for_each(|i| {
        // LMS-suffix is placed
        if str.get(*i).is_some_and(|s| is_s_type(*s)) {
            *i = COUNT_ZERO
        }
    });

    induce_s_type(str, sa, false);
}

/// ソート済みのLMS文字からLMS部分文字列をソートする
#[inline]
fn sort_lms_substrings<'a>(
    str: &[usize],
    sa: &'a mut [usize],
) -> (&'a mut [usize], &'a mut [usize]) {
    // all LMS-character are sorted
    let n_lms = {
        // sort LMS-prefixes
        induced_sort_lms(str, sa);

        // collect LMS-substrings to the tail
        let mut n_lms = 0;
        for i in (0..sa.len()).rev() {
            if sa[i] > 0 {
                // str[sa[i]] is LMS-type
                if is_s_type(str[sa[i]]) && !is_s_type(str[sa[i] - 1]) {
                    n_lms += 1;
                    sa[sa.len() - n_lms] = sa[i]
                }
            }
        }
        {
            let n = sa.len();
            sa[..n - n_lms].fill(COUNT_ZERO);
        }

        n_lms
    };

    // rename LMS-substrings
    {
        // sentinel is a LMS-substring
        sa[sa[sa.len() - n_lms] / 2] = 0;
        let mut kind_lms = 0;
        for i in sa.len() - n_lms + 1..sa.len() {
            let n1 = str[sa[i - 1]..]
                .windows(2)
                .take_while(|s| !(!is_s_type(s[0]) && is_s_type(s[1])))
                .count();
            let n2 = str[sa[i]..]
                .windows(2)
                .take_while(|s| !(!is_s_type(s[0]) && is_s_type(s[1])))
                .count();
            if str[sa[i - 1]..=sa[i - 1] + n1] != str[sa[i]..=sa[i] + n2] {
                kind_lms += 1
            }
            sa[sa[i] / 2] = kind_lms
        }
    };

    // collect renamed ones to the head
    {
        let mut n = 0;
        for i in 0..sa.len() - n_lms {
            if sa[i] & COUNTER_FILTER == 0 {
                sa[n] = std::mem::replace(&mut sa[i], COUNT_ZERO);
                n += 1
            }
        }
    }
    let (pre, sa) = sa.split_at_mut(sa.len() - n_lms);
    let (str, _) = pre.split_at_mut(n_lms);
    (str, sa)
}

#[inline]
fn sort_lms_suffixes(str: &[usize], sa: &mut [usize]) {
    // 部分問題をLMS型接尾辞で登場順に上書きする
    let mut n = 0;
    str.windows(2).enumerate().for_each(|(i, s)| {
        if !is_s_type(s[0]) && is_s_type(s[1]) {
            sa[n] = i + 1;
            n += 1;
        }
    });

    // 部分問題の解から、LMS型接尾辞をソートする。
    let l = sa.len() - n;
    for i in l..sa.len() {
        sa[i] = sa[sa[i]];
    }
    sa[..n].fill(COUNT_ZERO);

    // LMS型接尾辞をバケットに書き込む
    {
        n = 0;
        // 番兵は最小の接尾辞
        let mut s = COUNTER_FILTER;
        for i in l..sa.len() {
            // バケット内でのオフセット
            if str[sa[i]] == s {
                n += 1
            } else {
                sa[str_to_ptr(s) - n..=str_to_ptr(s)].reverse();
                n = 0
            };
            s = str[sa[i]];
            // 同じ場所に書き込むことがあるので、初期化してから上書きする。
            sa[str_to_ptr(s) - n] = std::mem::replace(&mut sa[i], COUNT_ZERO);
        }
        sa[str_to_ptr(s) - n..=str_to_ptr(s)].reverse();
    }
}

/// Returns the suffix array.
///
/// The input will be modified and useless.
///
/// # Constraints
///
/// - `str.len() == sa.len()`
/// - only the last character is `0` (sentinel)
///
/// # Time Complexity
///
/// *Θ*(*N*)
pub fn suffix_array(str: &mut [usize], sa: &mut [usize]) {
    assert_eq!(str.len(), sa.len());
    debug_assert!(
        str.iter().rev().skip(1).all(|s| 1 <= *s && *s < str.len()),
        "all characters except for the sentinel should be within [1, {})",
        str.len()
    );
    debug_assert_eq!(
        str[str.len() - 1],
        0,
        "`str` should end with the sentinel 0"
    );

    if sa.len() == 1 {
        sa[0] = 0;
        return;
    }

    rename(str, sa);
    sort_lms_chars(str, sa);
    {
        let (str, sa) = sort_lms_substrings(str, sa);
        suffix_array(str, sa);
    }
    sort_lms_suffixes(str, sa);
    induced_sort_lms(str, sa);
}

/// # Constraints
///
/// `str.len() == sa.len()`
///
/// # Time Complexity
///
/// *O*(*N*^2 log *N*)
pub fn suffix_array_brute_force<T: Ord>(str: &[T], sa: &mut [usize]) {
    assert_eq!(str.len(), sa.len());

    sa.iter_mut().enumerate().for_each(|(i, sa)| *sa = i);
    sa.sort_unstable_by_key(|i| &str[*i..]);
}

#[cfg(test)]
mod compact {
    use rand::Rng;

    use super::*;

    fn assert(str: &mut [usize]) {
        let mut sa2 = vec![0; str.len()];
        suffix_array_brute_force(&str, &mut sa2);

        let mut sa1 = vec![0; str.len()];
        suffix_array(str, &mut sa1);

        assert_eq!(sa1, sa2)
    }

    /// DOI: <10.1016/j.ic.2021.104818>
    #[test]
    fn example() {
        let mut str = [2, 1, 1, 3, 3, 1, 1, 3, 3, 1, 2, 1, 0];
        assert(&mut str);
    }

    #[test]
    fn random() {
        let mut rng = rand::rng();
        for n in 300..600 {
            let mut str = Vec::from_iter((1..n).map(|_| rng.random_range(1..n)));
            str.push(0);
            assert(&mut str);
        }
    }
}

