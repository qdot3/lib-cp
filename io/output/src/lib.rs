use std::io::Write;

pub struct IntBuffer {
    buf: [u8; 40],
    len: usize,
}

impl IntBuffer {
    pub const fn new() -> Self {
        let buf = [0; _];
        let len = buf.len();

        Self { buf, len }
    }

    pub fn format<T>(&mut self, n: T) -> &str
    where
        T: BufFormat<Buffer = Self>,
    {
        T::format(n, self)
    }

    pub fn format_iter<T>(
        &mut self,
        buf: &mut impl Write,
        mut iter: impl Iterator<Item = T>,
        sep: impl AsRef<[u8]>
    ) -> std::io::Result<()>
    where
        T: BufFormat<Buffer = Self>,
    {
        if let Some(v) = iter.next() {
            buf.write(T::format(v, self).as_bytes())?;
        }
        while let Some(v) = iter.next() {
            buf.write(sep.as_ref())?;
            buf.write(T::format(v, self).as_bytes())?;
        }

        Ok(())
    }
}

/// format 4 digits at once. 40 MB.
static LUT4: [[u8; 4]; 10000] = const {
    let mut lut = [[0; 4]; 10000];

    let mut i = 0;
    while i < 10000 {
        lut[i][3] = (i / 0001 % 10) as u8 + b'0';
        lut[i][2] = (i / 0010 % 10) as u8 + b'0';
        lut[i][1] = (i / 0100 % 10) as u8 + b'0';
        lut[i][0] = (i / 1000 % 10) as u8 + b'0';

        i += 1;
    }

    lut
};

mod sealed {
    pub trait Sealed {}

    macro_rules! seal {
        ($( $t:ty )*) => {$(
            impl Sealed for $t {}
        )*};
    }
    seal!( i8 u8 i16 u16 i32 u32 i64 u64 isize usize );

    impl<T> Sealed for &T where T: Sealed {}
    impl<T> Sealed for &mut T where T: Sealed {}
}

pub trait BufFormat: sealed::Sealed {
    type Buffer;

    fn format(self, buf: &mut Self::Buffer) -> &str;
}

impl<T> BufFormat for &T
where
    T: Copy + BufFormat,
{
    type Buffer = T::Buffer;

    #[inline(always)]
    fn format(self, buf: &mut Self::Buffer) -> &str {
        (*self).format(buf)
    }
}

impl<T> BufFormat for &mut T
where
    T: Copy + BufFormat,
{
    type Buffer = T::Buffer;

    #[inline(always)]
    fn format(self, buf: &mut Self::Buffer) -> &str {
        (*self).format(buf)
    }
}

macro_rules! impl_format_uint {
    ($( $t:ty )*) => {$(
        impl BufFormat for $t {
            type Buffer = IntBuffer;

            fn format(mut self, buf: &mut Self::Buffer) -> &str {
                buf.len = buf.buf.len();

                while {
                    let rem = self % 10000;
                    self /= 10000;

                    buf.len -= 4;
                    buf.buf[buf.len..buf.len + 4].copy_from_slice(&LUT4[rem as usize]);

                    self > 0
                } {}
                let n = u32::from_le_bytes(buf.buf[buf.len..].as_chunks::<4>().0[0]) ^ 0x0030_3030;
                let offset = n.trailing_zeros() as usize / 8;
                buf.len += offset;

                // SAFETY: ASCII graphic characters only
                unsafe { str::from_utf8_unchecked(&buf.buf[buf.len..]) }
            }
        }
    )*};
}
impl_format_uint!( u16 u32 u64 usize );

impl BufFormat for u8 {
    type Buffer = IntBuffer;

    fn format(self, buf: &mut Self::Buffer) -> &str {
        buf.len = buf.buf.len();
        buf.format(self as u16)
    }
}

macro_rules! impl_format_int {
    ($( $t:ty )*) => {$(
        impl BufFormat for $t {
            type Buffer = IntBuffer;

            fn format(self, buf: &mut Self::Buffer) -> &str {
                buf.format(self.unsigned_abs());
                if self.is_negative() {
                    buf.len -= 1;
                    buf.buf[buf.len] = b'-';
                }

                // SAFETY: ASCII graphic characters only
                unsafe { str::from_utf8_unchecked(&buf.buf[buf.len..]) }
            }
        }
    )*};
}
impl_format_int!( i8 i16 i32 i64 isize );

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed() {
        let mut buf = IntBuffer::new();

        assert_eq!(buf.format(-00i8), "0");
        assert_eq!(buf.format(-1i8), "-1");
        assert_eq!(buf.format(i32::MIN), i32::MIN.to_string().as_str());
        assert_eq!(buf.format(i32::MAX), i32::MAX.to_string().as_str());
    }
}
