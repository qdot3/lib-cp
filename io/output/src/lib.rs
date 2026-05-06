pub struct IntBuffer {
    buf: [u8; 40],
    len: usize,
}

impl IntBuffer {
    pub const fn new() -> Self {
        Self {
            buf: [0; _],
            len: 40,
        }
    }

    pub fn format<T>(&mut self, n: T) -> &str
    where
        T: BufFormat<Buffer = Self>,
    {
        T::format(n, self)
    }
}

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

pub trait BufFormat {
    type Buffer;

    fn format(self, buf: &mut Self::Buffer) -> &str;
}

macro_rules! impl_format_uint {
    ($( $t:ty )*) => {$(
        impl BufFormat for $t {
            type Buffer = IntBuffer;

            fn format(mut self, buf: &mut Self::Buffer) -> &str {
                buf.len = buf.buf.len();

                while self >= 1000 {
                    let rem = self % 10000;
                    self /= 10000;

                    buf.len -= 4;
                    buf.buf[buf.len..buf.len + 4].copy_from_slice(&LUT4[rem as usize])
                }
                while self > 0 {
                    let rem = self % 10;
                    self /= 10;

                    buf.len -= 1;
                    buf.buf[buf.len] = rem as u8 + b'0';
                }
                if buf.len == buf.buf.len() {
                    buf.len -= 1;
                    buf.buf[buf.len] = b'0'
                }

                unsafe { str::from_utf8_unchecked(&buf.buf[buf.len..]) }
            }
        }
    )*};
}
impl_format_uint!( u16 u32 u64 );

impl BufFormat for u8 {
    type Buffer = IntBuffer;

    fn format(mut self, buf: &mut Self::Buffer) -> &str {
        buf.len = buf.buf.len();

        while {
            let rem = self % 10;
            self /= 10;

            buf.len -= 1;
            buf.buf[buf.len] = rem + b'0';

            self > 0
        } {}

        unsafe { str::from_utf8_unchecked(&buf.buf[buf.len..]) }
    }
}

macro_rules! impl_format_usize {
    ( $u:ty ) => {
        impl BufFormat for usize {
            type Buffer = IntBuffer;

            fn format(self, buf: &mut Self::Buffer) -> &str {
                buf.format(self as $u)
            }
        }
    };
}
#[cfg(target_pointer_width = "64")]
impl_format_usize!(u64);
#[cfg(target_pointer_width = "32")]
impl_format_usize!(u32);
#[cfg(target_pointer_width = "16")]
impl_format_usize!(u16);

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

        assert_eq!(buf.format(-0i8), "0");
        assert_eq!(buf.format(-1i8), "-1");
        assert_eq!(buf.format(i32::MIN), i32::MIN.to_string().as_str());
        assert_eq!(buf.format(i32::MAX), i32::MAX.to_string().as_str());
    }
}
