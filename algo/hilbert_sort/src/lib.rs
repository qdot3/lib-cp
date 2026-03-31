use std::ops::{BitOr, Not};

pub trait Unsigned: Copy + BitOr<Output = Self> + Not<Output = Self> {
    const BITS: u32;
    const ZERO: Self;

    fn bit_width(self) -> u32;
    fn test(self, n: u8) -> bool;
}

macro_rules! unsigned_impl {
    ($( $t:ty )*) => {$(
        impl Unsigned for $t {
            const BITS: u32 = <$t>::BITS;
            const ZERO: $t = 0;

            fn bit_width(self) -> u32 {
                Self::BITS - self.leading_zeros()
            }

            fn test(self, n: u8) -> bool {
                (self >> n) & 1 == 1
            }
        }
    )*};
}
unsigned_impl!( u8 u16 u32 u64 u128 usize );

#[derive(Debug)]
pub struct Query<U, T>
where
    U: Unsigned,
{
    pub x: U,
    pub y: U,

    pub data: T,
}

/// # Time Complexity
///
/// *O*(N log max A_i)
pub fn hilbert_sort_2d<U, T>(slice: &mut [Query<U, T>])
where
    U: Unsigned,
{
    let order = slice
        .iter()
        .fold(U::ZERO, |acc, q| acc | q.x | q.y)
        .bit_width();

    fn inner<U, T>(slice: &mut [Query<U, T>], order: u8)
    where
        U: Unsigned,
    {
        if slice.len() <= 1 {
            return;
        }

        // #[rustfmt::skip]
        macro_rules! partition {
            // test = false -> asc
            // test = true  -> des
            ( $slice:expr, $dir:ident, $shift:expr, $test:expr ) => {{
                let (mut l, mut r) = (0, ($slice).len());
                loop {
                    l += ($slice)[l..r]
                        .iter()
                        .take_while(|q| q.$dir.test($shift) == $test)
                        .count();
                    r -= ($slice)[l..r]
                        .iter()
                        .rev()
                        .take_while(|q| q.$dir.test($shift) == !$test)
                        .count();

                    if r - l > 1 {
                        ($slice).swap(l, r - 1);
                    } else {
                        break ($slice).split_at_mut(l);
                    }
                }
            }};
        }

        // quadrant
        // ┃ 0  3 ▲  ┏━▶ y
        // ┃ 1  2 ┃  ▼
        // ┗━━━━━━┛  x
        let (l, r) = partition!(slice, y, order, false);
        {
            let (quad0, quad1) = partition!(l, x, order, false);

            if order > 0 {
                quad0.iter_mut().for_each(|q| (q.x, q.y) = (q.y, q.x));
                inner(quad0, order - 1);
                quad0.iter_mut().for_each(|q| (q.x, q.y) = (q.y, q.x));

                inner(quad1, order - 1);
            }
        }
        {
            let (quad2, quad3) = partition!(r, x, order, true);

            if order > 0 {
                inner(quad2, order - 1);

                quad3.iter_mut().for_each(|q| (q.x, q.y) = (!q.y, !q.x));
                inner(quad3, order - 1);
                quad3.iter_mut().for_each(|q| (q.x, q.y) = (!q.y, !q.x));
            }
        }
    }

    inner(slice, order as u8);
}

#[test]
fn hilbert_8x8() {
    let mut vec = Vec::with_capacity(8 * 8);
    for x in 0..8 {
        for y in 0..8 {
            vec.push(Query { x, y, data: () });
        }
    }

    hilbert_sort_2d::<u32, ()>(&mut vec);

    let mut order = [[!0; 8]; 8];
    for (i, q) in vec.into_iter().enumerate() {
        order[q.y as usize][q.x as usize] = i;
    }

    for row in order {
        println!("{:?}", row)
    }
}
