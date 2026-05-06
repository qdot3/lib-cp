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

    const fn clear(&mut self) {
        self.len = self.buf.len()
    }

    const fn is_empty(&self) -> bool {
        self.len == self.buf.len()
    }

    /// - `10 <= n <= 99`
    /// - 最初に書き込む前に`self.len`を初期化する必要がある
    #[inline(always)]
    fn write_2_digits(&mut self, n: usize) {
        static LUT: &[u8; 200] = b"00010203040506070809101112131415161718192021222324252627282930313233343536373839404142434445464748495051525354555657585960616263646566676869707172737475767778798081828384858687888990919293949596979899";

        self.len -= 2;
        self.buf[self.len..self.len + 2].copy_from_slice(&LUT[n * 2..n * 2 + 2]);
    }

    pub fn format<T>(&mut self, n: T) -> &str
    where
        T: Format<Buffer = Self>,
    {
        T::format(n, self)
    }
}

pub trait Format {
    type Buffer;

    fn format(self, buf: &mut Self::Buffer) -> &str;
}

macro_rules! impl_format_uint {
    ($( $t:ty )*) => {$(
        impl Format for $t {
            type Buffer = IntBuffer;

            fn format(mut self, buf: &mut Self::Buffer) -> &str {
                buf.clear();

                while self >= 10 {
                    let rem = self % 100;
                    self /= 100;

                    buf.write_2_digits(rem as usize);
                }
                if self == 0 || buf.is_empty() {
                    buf.len -= 1;
                    buf.buf[buf.len] = self as u8 + b'0';
                }

                unsafe { str::from_utf8_unchecked(&buf.buf[buf.len..]) }
            }
        }
    )*};
}
impl_format_uint!( u8 u16 u32 u64 );

macro_rules! impl_format_usize {
    ( $u:ty ) => {
        impl Format for usize {
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
        impl Format for $t {
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
impl_format_int!( i8 i16 i32 i64 isize);
