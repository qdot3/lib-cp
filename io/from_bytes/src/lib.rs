//! 参考サイト
//! - <https://zenn.dev/mizar/articles/fc87d667153080>

/// FIXME: use std::hint::cold_path();
#[cold]
const fn cold_path() {}

/// `b"1234567890123456"`を`1234567890123456_u64`にパースする。
#[inline(always)]
const fn parse_16_digits(bytes: [u8; 16]) -> Option<u64> {
    let (mut hi, mut lo) = {
        let bytes = bytes.as_chunks::<8>().0;
        // ascii コードの 0x30..=0x39 が数値に対応している
        (
            u64::from_le_bytes(bytes[0]) ^ 0x3030_3030_3030_3030,
            u64::from_le_bytes(bytes[1]) ^ 0x3030_3030_3030_3030,
        )
    };

    // 上位４ビットが０で下位４ビットが９以下のとき、かつそのときに限り、等号成立
    // - 7 * 9 < 2^6 < 7 * 10, 7 * 15 = 105 < 2^7
    if (hi | lo | (hi + 0x0606_0606_0606_0606) | (lo + 0x0606_0606_0606_0606))
        & 0xf0f0_f0f0_f0f0_f0f0
        == 0
    {
        // [8, 7, 6, 5, 4, 3, 2, 1] -> [78, 56, 34, 12]
        hi = (hi.wrapping_mul((10 << 8) + 1) >> 8) & 0x00ff_00ff_00ff_00ff;
        lo = (lo.wrapping_mul((10 << 8) + 1) >> 8) & 0x00ff_00ff_00ff_00ff;
        // [78, 56, 34, 12] -> [5678, 1234]
        hi = (hi.wrapping_mul((100 << 16) + 1) >> 16) & 0x0000_ffff_0000_ffff;
        lo = (lo.wrapping_mul((100 << 16) + 1) >> 16) & 0x0000_ffff_0000_ffff;
        // [5678, 1234] -> [12345678]
        hi = hi.wrapping_mul((10000 << 32) + 1) >> 32;
        lo = lo.wrapping_mul((10000 << 32) + 1) >> 32;

        Some(hi * 1_0000_0000 + lo)
    } else {
        cold_path();
        None
    }
}

/// `b"12345678"`を`12345678_u64`に変換する。
// inlined due to frequent calls
#[inline(always)]
const fn parse_8_digits(bytes: [u8; 8]) -> Option<u64> {
    // ascii コードの 0x30..=0x39 が数値に対応している
    let mut n = u64::from_le_bytes(bytes) ^ 0x3030_3030_3030_3030;

    // 上位４ビットが０で下位４ビットが９以下のとき、かつそのときに限り、等号成立
    // - 7 * 9 < 2^6 < 7 * 10, 7 * 15 = 105 < 2^7
    if (n | (n + 0x0606_0606_0606_0606)) & 0xf0f0_f0f0_f0f0_f0f0 == 0 {
        // [8, 7, 6, 5, 4, 3, 2, 1] -> [78, 56, 34, 12]
        n = (n.wrapping_mul((10 << 8) + 1) >> 8) & 0x00ff_00ff_00ff_00ff;
        // [78, 56, 34, 12] -> [5678, 1234]
        n = (n.wrapping_mul((100 << 16) + 1) >> 16) & 0x0000_ffff_0000_ffff;
        // [5678, 1234] -> [12345678]
        n = n.wrapping_mul((10000 << 32) + 1) >> 32;

        Some(n)
    } else {
        cold_path();
        None
    }
}

/// `b"1234"`を`1234_u32`に変換する。
#[inline(always)]
const fn parse_4_digits(bytes: [u8; 4]) -> Option<u32> {
    // ascii コードの 0x30..=0x39 が数値に対応している
    let mut n = u32::from_le_bytes(bytes) ^ 0x3030_3030;

    // 上位４ビットが０で下位４ビットが９以下のとき、かつそのときに限り、等号成立
    // - 7 * 9 < 2^6 < 7 * 10, 7 * 15 = 105 < 2^7
    if (n | n + 0x0606_0606) & 0xf0f0_f0f0 == 0 {
        // [4, 3, 2, 1] -> [34, 12]
        n = (n.wrapping_mul((10 << 8) + 1) >> 8) & 0x00ff_00ff;
        // [34, 12] -> [1234]
        n = n.wrapping_mul((100 << 16) + 1) >> 16;

        Some(n)
    } else {
        cold_path();
        None
    }
}

pub trait FromBytes: Sized {
    type Err;

    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Err>;
}

impl FromBytes for u64 {
    type Err = ();

    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Err> {
        let digits = bytes.strip_prefix(b"+").unwrap_or(bytes);

        if digits.is_empty() {
            cold_path();
            return Err(());
        }

        let (pre, suf) = digits.as_rchunks::<8>();
        let mut n = {
            let mut bytes = [b'0'; 8];
            // memcpy回避
            match pre.len() {
                1 => bytes[7..].copy_from_slice(&pre),
                2 => bytes[6..].copy_from_slice(&pre),
                3 => bytes[5..].copy_from_slice(&pre),
                4 => bytes[4..].copy_from_slice(&pre),
                5 => bytes[3..].copy_from_slice(&pre),
                6 => bytes[2..].copy_from_slice(&pre),
                7 => bytes[1..].copy_from_slice(&pre),
                _ => {}
            };
            match pre.len() {
                1 | 2 | 3 | 4 => parse_4_digits(bytes.as_chunks::<4>().0[1]).ok_or(())? as u64,
                5 | 6 | 7 => parse_8_digits(bytes).ok_or(())?,
                _ => 0,
            }
        };

        let mut of = false;
        for chunk in suf {
            let x = n.overflowing_mul(1_0000_0000);
            n = x.0;
            of |= x.1;

            let x = n.overflowing_add(parse_8_digits(*chunk).ok_or(())?);
            n = x.0;
            of |= x.1;
        }

        if of {
            cold_path();
            Err(())
        } else {
            Ok(n)
        }
    }
}

impl FromBytes for u128 {
    type Err = ();

    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Err> {
        let digits = bytes.strip_prefix(b"+").unwrap_or(bytes);

        if digits.is_empty() {
            cold_path();
            return Err(());
        }

        let (pre, suf) = digits.as_rchunks::<16>();
        let mut n = {
            let mut bytes = [b'0'; 16];
            // memcpy回避
            match pre.len() {
                1 => bytes[15..].copy_from_slice(&pre),
                2 => bytes[14..].copy_from_slice(&pre),
                3 => bytes[13..].copy_from_slice(&pre),
                4 => bytes[12..].copy_from_slice(&pre),
                5 => bytes[11..].copy_from_slice(&pre),
                6 => bytes[10..].copy_from_slice(&pre),
                7 => bytes[9..].copy_from_slice(&pre),
                8 => bytes[8..].copy_from_slice(&pre),
                9 => bytes[7..].copy_from_slice(&pre),
                10 => bytes[6..].copy_from_slice(&pre),
                11 => bytes[5..].copy_from_slice(&pre),
                12 => bytes[4..].copy_from_slice(&pre),
                13 => bytes[3..].copy_from_slice(&pre),
                14 => bytes[2..].copy_from_slice(&pre),
                15 => bytes[1..].copy_from_slice(&pre),
                _ => {}
            };
            match pre.len() {
                1 | 2 | 3 | 4 => parse_4_digits(bytes.as_chunks::<4>().0[3]).ok_or(())? as u128,
                5 | 6 | 7 | 8 => parse_8_digits(bytes.as_chunks::<8>().0[1]).ok_or(())? as u128,
                9 | 10 | 11 | 12 | 13 | 14 | 15 => parse_16_digits(bytes).ok_or(())? as u128,
                _ => 0,
            }
        };

        let mut of = false;
        for chunk in suf {
            let x = n.overflowing_mul(1_0000_0000_0000_0000);
            n = x.0;
            of |= x.1;

            let x = n.overflowing_add(parse_16_digits(*chunk).ok_or(())? as u128);
            n = x.0;
            of |= x.1;
        }

        if of {
            cold_path();
            Err(())
        } else {
            Ok(n)
        }
    }
}

impl FromBytes for i64 {
    type Err = ();

    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Err> {
        let (is_positive, digits) = match bytes {
            [b'-', rest @ ..] => (false, rest),
            [b'+', rest @ ..] | rest => (true, rest),
        };

        if digits.is_empty() {
            cold_path();
            return Err(());
        }

        let (pre, suf) = digits.as_rchunks::<8>();
        let mut n = {
            let mut bytes = [b'0'; 8];
            // memcpy回避
            match pre.len() {
                1 => bytes[7..].copy_from_slice(&pre),
                2 => bytes[6..].copy_from_slice(&pre),
                3 => bytes[5..].copy_from_slice(&pre),
                4 => bytes[4..].copy_from_slice(&pre),
                5 => bytes[3..].copy_from_slice(&pre),
                6 => bytes[2..].copy_from_slice(&pre),
                7 => bytes[1..].copy_from_slice(&pre),
                _ => {}
            };
            match pre.len() {
                1 | 2 | 3 | 4 => parse_4_digits(bytes.as_chunks::<4>().0[1]).ok_or(())? as i64,
                5 | 6 | 7 => parse_8_digits(bytes).ok_or(())? as i64,
                _ => 0,
            }
        };

        let mut of = false;
        if is_positive {
            for chunk in suf {
                let x = n.overflowing_mul(1_0000_0000);
                n = x.0;
                of |= x.1;

                let x = n.overflowing_add(parse_8_digits(*chunk).ok_or(())? as i64);
                n = x.0;
                of |= x.1;
            }
        } else {
            n = -n;
            for chunk in suf {
                let x = n.overflowing_mul(1_0000_0000);
                n = x.0;
                of |= x.1;

                let x = n.overflowing_sub(parse_8_digits(*chunk).ok_or(())? as i64);
                n = x.0;
                of |= x.1;
            }
        }

        if of {
            cold_path();
            Err(())
        } else {
            Ok(n)
        }
    }
}

impl FromBytes for i128 {
    type Err = ();

    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Err> {
        let (is_positive, digits) = match bytes {
            [b'-', rest @ ..] => (false, rest),
            [b'+', rest @ ..] | rest => (true, rest),
        };

        if digits.is_empty() {
            cold_path();
            return Err(());
        }

        let (pre, suf) = digits.as_rchunks::<16>();
        let mut n = {
            let mut bytes = [b'0'; 16];
            // memcpy回避
            match pre.len() {
                1 => bytes[15..].copy_from_slice(&pre),
                2 => bytes[14..].copy_from_slice(&pre),
                3 => bytes[13..].copy_from_slice(&pre),
                4 => bytes[12..].copy_from_slice(&pre),
                5 => bytes[11..].copy_from_slice(&pre),
                6 => bytes[10..].copy_from_slice(&pre),
                7 => bytes[9..].copy_from_slice(&pre),
                8 => bytes[8..].copy_from_slice(&pre),
                9 => bytes[7..].copy_from_slice(&pre),
                10 => bytes[6..].copy_from_slice(&pre),
                11 => bytes[5..].copy_from_slice(&pre),
                12 => bytes[4..].copy_from_slice(&pre),
                13 => bytes[3..].copy_from_slice(&pre),
                14 => bytes[2..].copy_from_slice(&pre),
                15 => bytes[1..].copy_from_slice(&pre),
                _ => {}
            };
            match pre.len() {
                1 | 2 | 3 | 4 => parse_4_digits(bytes.as_chunks::<4>().0[3]).ok_or(())? as i128,
                5 | 6 | 7 | 8 => parse_8_digits(bytes.as_chunks::<8>().0[1]).ok_or(())? as i128,
                9 | 10 | 11 | 12 | 13 | 14 | 15 => parse_16_digits(bytes).ok_or(())? as i128,
                _ => 0,
            }
        };
        let mut of = false;
        if is_positive {
            for chunk in suf {
                let x = n.overflowing_mul(1_0000_0000_0000_0000);
                n = x.0;
                of |= x.1;

                let x = n.overflowing_add(parse_16_digits(*chunk).ok_or(())? as i128);
                n = x.0;
                of |= x.1;
            }
        } else {
            n = -n;
            for chunk in suf {
                let x = n.overflowing_mul(1_0000_0000_0000_0000);
                n = x.0;
                of |= x.1;

                let x = n.overflowing_sub(parse_16_digits(*chunk).ok_or(())? as i128);
                n = x.0;
                of |= x.1;
            }
        }

        if of {
            cold_path();
            Err(())
        } else {
            Ok(n)
        }
    }
}

macro_rules! from_bytes_derive {
    ( $tar:ty as $src:ty) => {
        impl FromBytes for $tar {
            type Err = ();

            fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Err> {
                let wide = <$src>::from_bytes(bytes)?;
                <$tar>::try_from(wide).map_err(|_| ())
            }
        }
    };
}

from_bytes_derive!(u8 as u64);
from_bytes_derive!(u16 as u64);
from_bytes_derive!(u32 as u64);
from_bytes_derive!(usize as u64);

from_bytes_derive!(i8 as i64);
from_bytes_derive!(i16 as i64);
from_bytes_derive!(i32 as i64);
from_bytes_derive!(isize as i64);

#[cfg(test)]
mod tests {
    use rand::{rng, Rng};

    use super::*;

    #[test]
    fn random() {
        let mut rng = rng();
        for _ in 0..1 << 20 {
            let n: u64 = rng.random();
            let s = n.to_string().clone();
            assert_eq!(u64::from_bytes(s.as_bytes()).ok(), Some(n))
        }
    }

    #[test]
    fn empty() {
        assert_eq!(u64::from_bytes(b"").ok(), None)
    }

    #[test]
    fn overflow() {
        assert!(u64::from_bytes("1".repeat(1000).as_bytes()).is_err());
        assert!(i64::from_bytes("1".repeat(1000).as_bytes()).is_err());
    }

    #[test]
    fn min_max() {
        macro_rules! min_max {
            ( $( $t:ty )* ) => {$(
                let s = <$t>::MAX.to_string().clone();
                assert_eq!(<$t>::from_bytes(s.as_bytes()).ok(), Some(<$t>::MAX));

                let s = <$t>::MIN.to_string().clone();
                assert_eq!(<$t>::from_bytes(s.as_bytes()).ok(), Some(<$t>::MIN));
            )*};
        }

        min_max! { i8 u8 i16 u16 i32 u32 i64 u64 i128 u128 isize usize }
    }
}
