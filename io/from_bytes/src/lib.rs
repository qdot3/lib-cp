use std::num::IntErrorKind;

/// `b"12345678"`を`12345678_u64`に変換する。
#[inline(always)]
const fn parse_8_digits(bytes: [u8; 8]) -> Result<u64, IntErrorKind> {
    // ascii コードの 0x30..=0x39 が数値に対応している
    let mut n = u64::from_le_bytes(bytes) ^ 0x3030_3030_3030_3030;

    // 上位４ビットが０で下位４ビットが９以下のとき、かつそのときに限り、等号成立
    // - 7 * 9 < 2^6 < 7 * 10, 7 * 15 = 105 < 2^7
    if (n & 0xf0f0_f0f0_f0f0_f0f0) | (n.wrapping_mul(7) & 0x4040_4040_4040_4040) == 0 {
        // [8, 7, 6, 5, 4, 3, 2, 1] -> [78, 56, 34, 12]
        n = (n.wrapping_mul((10 << 8) + 1) >> 8) & 0x00ff_00ff_00ff_00ff;
        // [78, 56, 34, 12] -> [5678, 1234]
        n = (n.wrapping_mul((100 << 16) + 1) >> 16) & 0x0000_ffff_0000_ffff;
        // [5678, 1234] -> [12345678]
        n = n.wrapping_mul((10000 << 32) + 1) >> 32;

        Ok(n)
    } else {
        Err(IntErrorKind::InvalidDigit)
    }
}

/// `b"1234"`を`1234_u32`に変換する。
#[inline(always)]
const fn parse_4_digits(bytes: [u8; 4]) -> Result<u32, IntErrorKind> {
    // ascii コードの 0x30..=0x39 が数値に対応している
    let mut n = u32::from_le_bytes(bytes) ^ 0x3030_3030;

    // 上位４ビットが０で下位４ビットが９以下のとき、かつそのときに限り、等号成立
    // - 7 * 9 < 2^6 < 7 * 10, 7 * 15 = 105 < 2^7
    if (n & 0xf0f0_f0f0) | (n.wrapping_mul(7) & 0x4040_4040) == 0 {
        // [4, 3, 2, 1] -> [34, 12]
        n = (n.wrapping_mul((10 << 8) + 1) >> 8) & 0x00ff_00ff;
        // [34, 12] -> \1234]
        n = n.wrapping_mul((100 << 16) + 1) >> 16;

        Ok(n)
    } else {
        Err(IntErrorKind::InvalidDigit)
    }
}

pub trait FromBytes: Sized {
    type Err;

    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Err>;
}

macro_rules! calc_digits {
    ( $t:ty, $digits:ident, $mul:tt, $add:ident $(, $overflow: path )* ) => {{
        let (chunks, remainder) = $digits.as_chunks::<8>();
        let mut n = 0 as $t;
        for chunk in chunks {
            n = n.$mul(#[allow(overflowing_literals)] 1_0000_0000) $(.ok_or($overflow)?)*;
            n = n.$add(parse_8_digits(*chunk)? as $t) $(.ok_or($overflow)?)*;
        }

        #[allow(overflowing_literals)]
        const POW10: [$t; 8] = [1, 10, 100, 1000, 1_0000, 10_0000, 100_0000, 1000_0000];
        n = n.$mul(POW10[remainder.len()]) $(.ok_or($overflow)?)*;
        if remainder.len() > 4 {
            let mut digits = [b'0'; 8];
            digits[8 - remainder.len()..].copy_from_slice(remainder);
            n = n.$add(parse_8_digits(digits)? as $t) $(.ok_or($overflow)?)*;
        } else {
            let mut digits = [b'0'; 4];
            digits[4 - remainder.len()..].copy_from_slice(remainder);
            n = n.$add(parse_4_digits(digits)? as $t) $(.ok_or($overflow)?)*;
        }

        n
    }};
}

macro_rules! from_bytes_impl {
    ($( $t:ty )*) => {$(
        impl FromBytes for $t {
            type Err = IntErrorKind;

            fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Err> {
                if bytes.is_empty() {
                    return Err(IntErrorKind::Empty);
                }

                #[allow(unused_comparisons)]
                let is_signed_ty = <$t>::MIN < 0;

                // 符号は高々１つ。先頭の b'0' を除去しても良いが、レアケース。
                let (is_positive, digits) = match bytes {
                    [b'+' | b'-'] => return Err(IntErrorKind::InvalidDigit),
                    [b'+', rest @ ..] => (true, rest),
                    [b'-', rest @ ..] if is_signed_ty => (false, rest),
                    _ => (true, bytes),
                };

                let never_overflow = {
                    // 符号に依らない
                    const MAX_DIGITS_LEN: usize = <$t>::MAX.ilog10() as usize + 1;
                    const LEADING_BYTE_OF_MAX: u8 =
                        (<$t>::MAX / (10 as $t).pow(MAX_DIGITS_LEN as u32 - 1)) as u8 + b'0';

                    (digits.len() < MAX_DIGITS_LEN)
                        || (digits.len() == MAX_DIGITS_LEN && digits[0] < LEADING_BYTE_OF_MAX)
                };

                let n = if never_overflow {
                    if is_positive {
                        calc_digits!($t, digits, wrapping_mul, wrapping_add)
                    } else {
                        calc_digits!($t, digits, wrapping_mul, wrapping_sub)
                    }
                } else {
                    if is_positive {
                        calc_digits!($t, digits, checked_mul, checked_add, IntErrorKind::PosOverflow)
                    } else {
                        calc_digits!($t, digits, checked_mul, checked_sub, IntErrorKind::NegOverflow)
                    }
                };

                Ok(n)
            }
        }
    )*};
}
from_bytes_impl!( i32 u32 i64 u64 i128 u128 isize usize );

// 最大桁数が８未満だとオーバーフローしてしまう（MIN.abs() = MAX + 1）
macro_rules! from_bytes_impl_small {
    ($( $t:ty )*) => {$(
        impl FromBytes for $t {
            type Err = IntErrorKind;

            fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Err> {
                let v = i32::from_bytes(bytes)?;

                if v < <$t>::MIN as i32 {
                    Err(IntErrorKind::NegOverflow)
                } else if v > <$t>::MAX as i32 {
                    Err(IntErrorKind::PosOverflow)
                } else{
                    Ok(v as $t)
                }
            }
        }
    )*};
}
from_bytes_impl_small!( i8 u8 i16 u16 );

#[cfg(test)]
mod tests {

    use rand::{rng, Rng};

    use super::*;

    #[test]
    fn random() {
        let mut rng = rng();
        for _ in 0..1000 {
            let n: i64 = rng.random();
            let s = n.to_string().clone();
            assert_eq!(i64::from_bytes(s.as_bytes()), Ok(n))
        }
    }

    #[test]
    fn min_max() {
        macro_rules! min_max {
            ( $( $t:ty )* ) => {$(
                let s = <$t>::MAX.to_string().clone();
                assert_eq!(<$t>::from_bytes(s.as_bytes()), Ok(<$t>::MAX));

                let s = <$t>::MIN.to_string().clone();
                assert_eq!(<$t>::from_bytes(s.as_bytes()), Ok(<$t>::MIN));
            )*};
        }

        min_max! { i8 u8 i16 u16 i32 u32 i64 u64 i128 u128 isize usize }
    }
}
