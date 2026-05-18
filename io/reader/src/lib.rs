use core::slice;
use std::{borrow::Cow, io::Read, mem::MaybeUninit};

use from_bytes::FromBytes;

pub struct FastBufReader<const N: usize, R>
where
    R: Read,
{
    reader: R,

    buf: Box<[MaybeUninit<u8>; N]>,
    filled: usize,
    cursor: usize,
}

impl<const N: usize, R: Read> FastBufReader<N, R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buf: Box::new([const { MaybeUninit::uninit() }; _]),
            filled: 0,
            cursor: 0,
        }
    }

    fn fill_buf(&mut self) -> std::io::Result<usize> {
        if self.cursor < self.filled {
            self.buf.copy_within(self.cursor..self.filled, 0);
        }
        self.filled -= self.cursor;
        self.cursor = 0;

        let uninit = &mut self.buf[self.filled..];

        // SAFETY:
        // - `uninit` provides valid ptr and has sufficient capacity
        // - `buf` has exclusive access to `uninit` during its lifetime
        let buf =
            unsafe { slice::from_raw_parts_mut(uninit.as_mut_ptr().cast::<u8>(), uninit.len()) };
        // FIXME: EOF
        let n = self.reader.read(buf)?;
        self.filled += n;

        Ok(n)
    }

    fn skip_until_ascii_whitespace(&mut self) {
        self.cursor += self.buf[self.cursor..self.filled]
            .iter()
            .take_while(|b| unsafe { b.assume_init() }.is_ascii_whitespace())
            .count();
    }

    fn position_ascii_whitespace(&self) -> usize {
        const D: usize = std::mem::size_of::<usize>();

        const ASCII_WHITESPACE: [usize; 3] = [
            // usize::from_ne_bytes([b'\t'; D]),
            // usize::from_ne_bytes([b'\x0C'; D]),
            usize::from_ne_bytes([b'\n'; D]),
            usize::from_ne_bytes([b'\r'; D]),
            usize::from_ne_bytes([b' '; D]),
        ];

        // SAFETY:
        // - `init` are initialized and provides valid ptr
        // - `buf` has exclusive access to `init` during its lifetime
        let init = &self.buf[self.cursor..self.filled];
        let buf = unsafe { slice::from_raw_parts(init.as_ptr().cast::<u8>(), init.len()) };
        let (chunks, remainder) = buf.as_chunks::<D>();

        let mut n = 0;
        for chunk in chunks {
            let packed = usize::from_le_bytes(*chunk);

            let one = const { usize::from_ne_bytes([0x01; D]) };

            // SWAG to find first ascii whitespace
            let acc = ASCII_WHITESPACE.iter().fold(0, |acc, tar| {
                let v = (packed ^ tar).wrapping_sub(one);
                acc | v
            }) & !packed
                & const { usize::from_ne_bytes([0x80; D]) };
            let pos = acc.trailing_zeros() as usize / 8;

            n += pos;

            if pos < 8 {
                return n;
            }
        }
        n += remainder
            .iter()
            .position(|b| b.is_ascii_whitespace())
            .unwrap_or(remainder.len());

        n
    }

    /// ` `, `\r`, `\n`を区切り文字とする
    pub fn next_token(&mut self) -> std::io::Result<Cow<'_, [u8]>> {
        self.skip_until_ascii_whitespace();

        if self.filled - self.cursor < 40 {
            self.fill_buf()?;
            self.skip_until_ascii_whitespace();
        }

        let n = self.position_ascii_whitespace();
        self.cursor += n;

        if self.cursor < self.filled {
            let token = &self.buf[self.cursor - n..self.cursor];
            // SAFETY
            // - `token` is initialized and provides valid ptr
            // - `token` will not be modified during the lifetime of `ret`
            let ret = unsafe { slice::from_raw_parts(token.as_ptr().cast::<u8>(), token.len()) };
            return Ok(Cow::from(ret));
        }

        self.cursor -= n;
        // FIXME: this is optimized for ArCoder
        let mut ret = Vec::with_capacity(1 << 19);
        loop {
            {
                let token = &self.buf[self.cursor..self.filled];
                // SAFETY
                // - `token` is initialized and provides valid ptr
                // - slice will be dropped immediately after creation
                ret.extend_from_slice(unsafe {
                    slice::from_raw_parts(token.as_ptr().cast::<u8>(), token.len())
                });
            }
            self.cursor = self.filled;

            self.fill_buf()?;
            debug_assert_eq!(
                self.cursor, 0,
                "bug: `self.cursor` should be initialized to `0`"
            );

            let n = self.position_ascii_whitespace();
            if n < self.filled {
                {
                    let token = &self.buf[self.cursor..self.cursor + n];
                    // SAFETY: see above
                    ret.extend_from_slice(unsafe {
                        slice::from_raw_parts(token.as_ptr().cast::<u8>(), token.len())
                    });
                }
                self.cursor = n;

                break;
            }
        }

        Ok(Cow::from(ret))
    }

    #[inline(always)]
    pub fn parse_next_token<T>(&mut self) -> Option<T>
    where
        T: FromBytes,
    {
        let token = self.next_token().ok()?;
        T::from_bytes(&token).ok()
    }

    #[inline(always)]
    pub fn parse_next_token_vec<T>(&mut self, n: usize) -> Option<Vec<T>>
    where
        T: FromBytes,
    {
        let mut ret = Vec::with_capacity(n);
        for _ in 0..n {
            let token = self.next_token().ok()?;
            ret.push(T::from_bytes(&token).ok()?)
        }

        Some(ret)
    }
}
