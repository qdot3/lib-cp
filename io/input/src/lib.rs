use std::io::{self, BufRead, ErrorKind};

use from_bytes::FromBytes;

pub struct FastInput<R>
where
    R: BufRead,
{
    reader: R,
    // ページをまたぐことがある
    buf: Vec<u8>,
}

impl<R> FastInput<R>
where
    R: BufRead,
{
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buf: Vec::new(),
        }
    }

    /// 空白区切りのバイト列を１つパースして返す。
    pub fn next_token<T>(&mut self) -> Option<T>
    // TODO: anyhow を使う
    where
        T: FromBytes,
    {
        // trim prefix ascii whitespaces
        {
            let buf = self.reader.fill_buf().ok()?;
            let i = buf.iter().take_while(|b| b.is_ascii_whitespace()).count();
            self.reader.consume(i);
        }

        let buf = self.reader.fill_buf().ok()?;
        if let Some(n) = buf.iter().position(|b| b.is_ascii_whitespace()) {
            let token = T::from_bytes(&buf[..n]);
            // 空白文字は不要
            self.reader.consume(n + 1);

            token
        } else {
            let mut buf = std::mem::take(&mut self.buf);
            self.next_bytes(&mut buf).ok()?;

            let token = T::from_bytes(&buf);

            buf.clear();
            self.buf = buf;

            token
        }
        .ok()
    }

    pub fn next_token_vec<T>(&mut self, len: usize) -> Option<Vec<T>>
    where
        T: FromBytes,
    {
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(self.next_token()?)
        }
        Some(vec)
    }

    /// 空白区切りのバイト列を１つ書き込む。
    ///
    /// `ErrorKind::Interrupted`は無視する。
    pub fn next_bytes(&mut self, buf: &mut Vec<u8>) -> io::Result<()> {
        // trim prefix ascii whitespaces
        {
            let buf = self.reader.fill_buf()?;
            let i = buf.iter().take_while(|b| b.is_ascii_whitespace()).count();
            self.reader.consume(i);
        }

        loop {
            let available = match self.reader.fill_buf() {
                Ok(buf) => buf,
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            };

            let (done, used) =
                if let Some(n) = available.iter().position(|b| b.is_ascii_whitespace()) {
                    buf.extend_from_slice(&available[..n]);
                    (true, n + 1)
                } else {
                    buf.extend_from_slice(&available);
                    (false, available.len())
                };
            self.reader.consume(used);

            if done || (used == 0) {
                return Ok(());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_bytes() {
        let lorem_ipsum = br"
        Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation
        ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in
        reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur
        sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est
        laborum.";

        let mut input = FastInput::new(&lorem_ipsum[..]);

        let mut buf = Vec::new();
        for token in lorem_ipsum
            .split(|b| b.is_ascii_whitespace())
            .filter(|v| !v.is_empty())
        {
            input.next_bytes(&mut buf).unwrap();
            assert_eq!(buf, token);

            buf.clear();
        }
        assert!({
            input.next_bytes(&mut buf).unwrap();
            buf.is_empty()
        });
    }

    #[test]
    fn extract_token() {
        let mut input = FastInput::new(
            &br"1 2
11 22
1000000000000000000 1000000000000000000"[..],
        );

        let num: Vec<u64> = std::iter::from_fn(|| input.next_token()).collect();
        assert_eq!(
            num,
            vec![1, 2, 11, 22, 1000000000000000000, 1000000000000000000]
        );
        assert!(input.next_token::<u64>().is_none());
    }
}


/// Parse single `Bytes`, `String`s, primitive integers, `Vec`tor or tuple.
#[macro_export]
macro_rules! parse {
    (@source [$source:ident] @rest $(,)?) => {};

    // accept only single types
    (@source [$source:ident] @rest [$($item:tt)+] $(,)?) => {
        $crate::parse!(@vec @source [$source] @item [$($item)+] @rest)
    };
    (@source [$source:ident] @rest ($($item:tt)+) $(,)?) => {
        $crate::parse!(@tuple @source [$source] @item [$($item)+] @rest)
    };
    (@source [$source:ident] @rest Bytes $(with capacity $len:expr)? $(,)?) => {{
        let mut bytes = Vec::with_capacity(0 $(+ TryInto::<usize>::try_into($len).unwrap())?);
        $source.next_bytes(&mut bytes).unwrap();
        bytes
    }};
    (@source [$source:ident] @rest String $(with capacity $len:expr)? $(,)?) => {{
        String::from_utf8(
            $crate::parse!(@source [$source] @rest Bytes $(with capacity $len)?)
        ).unwrap()
    }};
    (@source [$source:ident] @rest $item:ty $(,)?) => {
        $source.next_token::<$item>().unwrap()
    };

    // parse vec items
    (@vec @source [$source:ident] @item [[$($item:tt)+] ; $len:expr] @rest) => {
        (0..$len).into_iter().map(|_|
            $crate::parse!(@source [$source] @rest [$($item)+])
        ).collect::<Vec<_>>()
    };
    (@vec @source [$source:ident] @item [($($item:tt)+) ; $len:expr] @rest) => {
        (0..$len).into_iter().map(|_|
            $crate::parse!(@source [$source] @rest ($($item)+))
        ).collect::<Vec<_>>()
    };
    (@vec @source [$source:ident] @item [$item:ty ; $len:expr] @rest) => {
        (0..$len).into_iter().map(|_|
            $crate::parse!(@source [$source] @rest $item)
        ).collect::<Vec<_>>()
    };

    // parse tuple items
    (@tuple @source [$source:ident] @item [$($item:tt),+  $(,)?] @rest) => {
        ($(
            $crate::parse!(@source [$source] @rest $item ),
        )+)
    };

    // bug!
    (@source [$source:ident] $( $rest:tt )*) => {
        std::compile_error!("failed to parse")
    };

    // interface
    ($source:ident >> $( $rest:tt )*) => {
        $crate::parse!(@source [$source] @rest $($rest)*)
    };
}

#[cfg(test)]
mod parse_single_value {
    use super::parse;

    use super::FastInput;

    #[test]
    fn vec() {
        let mut input = FastInput::new(&b"1 2"[..]);
        let x = parse!(input >> [u8; 2], );
        assert_eq!(x, vec![1, 2]);
    }

    #[test]
    fn vec_with_runtime_specified_len() {
        let mut input = FastInput::new(&b"3 1 2 3"[..]);
        let n = parse!(input >> u8);
        let x = parse!(input >> [u8; n], );
        assert_eq!(x, vec![1, 2, 3]);
    }

    #[test]
    fn nested_vec1() {
        let mut input = FastInput::new(&b"1 2 3 4"[..]);
        let x = parse!(input >> [[u8; 2]; 2], );
        assert_eq!(x, vec![vec![1, 2], vec![3, 4]]);
    }

    #[test]
    fn nested_vec2() {
        let mut input = FastInput::new(&b"1 2 3 4 5 6 7 8"[..]);
        let x = parse!(input >> [[[u8; 2]; 2]; 2], );
        assert_eq!(
            x,
            vec![vec![vec![1, 2], vec![3, 4]], vec![vec![5, 6], vec![7, 8]]]
        );
    }

    #[test]
    fn tuple() {
        let mut input = FastInput::new(&b"1 2"[..]);
        let x = parse!(input >> (u8, u8,),);
        assert_eq!(x, (1, 2));
    }

    #[test]
    fn nested_tuple1() {
        let mut input = FastInput::new(&b"1 2 3"[..]);
        let x = parse!(input >> (u8, (u8, u8),),);
        assert_eq!(x, (1, (2, 3)));
    }

    #[test]
    fn nested_tuple2() {
        let mut input = FastInput::new(&b"1 2 3"[..]);
        let x = parse!(input >> ((u8, u8), u8, ),);
        assert_eq!(x, ((1, 2), 3));
    }

    #[test]
    fn nested_tuple3() {
        let mut input = FastInput::new(&b"1 2 3 4 5 6"[..]);
        let x = parse!(input >> (((u8, u8), u8, (u8)), (u8, u8)),);
        assert_eq!(x, (((1, 2), 3, (4,)), (5, 6)));
    }

    #[test]
    fn tuple_in_vec() {
        let mut input = FastInput::new(&b"1 2 3 4"[..]);
        let x = parse!(input >> [(u8, u8); 2]);
        assert_eq!(x, vec![(1, 2), (3, 4)])
    }

    #[test]
    fn vec_in_tuple() {
        let mut input = FastInput::new(&b"1 2 3 4 5"[..]);
        let x = parse!(input >> ([u8; 2], [u8; 3]));
        assert_eq!(x, (vec![1, 2], vec![3, 4, 5]))
    }

    #[test]
    fn bytes() {
        let mut input = FastInput::new(&b"abcde"[..]);
        let x = parse!(input >> Bytes);
        assert_eq!(x, b"abcde")
    }

    #[test]
    fn bytes_with_capacity() {
        let mut input = FastInput::new(&b"5 abcde"[..]);
        let n = parse!(input >> u8);
        let x = parse!(input >> Bytes with capacity n);
        assert_eq!(x, b"abcde")
    }
    #[test]

    fn string() {
        let mut input = FastInput::new(&b"abcde"[..]);
        let x = parse!(input >> String);
        assert_eq!(x, "abcde".to_string())
    }
}

/// Parse one or more values.
#[macro_export]
macro_rules! bind {
    // terminator
    (@source [$source:ident] @rest) => {};

    // strip leading commas
    (@source [$source:ident] @rest , $($rest:tt)*) => {
        $crate::bind!(@source [$source] @rest $($rest)*)
    };

    // parse mutability
    (@source [$source:ident] @rest mut $( $rest:tt )*) => {
        $crate::bind!(@source [$source] @mut [mut] @rest $($rest)*)
    };
    (@source [$source:ident] @rest $( $rest:tt )*) => {
        $crate::bind!(@source [$source] @mut [] @rest $($rest)*)
    };

    // parse identifier
    (@source [$source:ident] @mut [$($mut:tt)?] @rest $ident:tt : $( $rest:tt )*) => {
        $crate::bind!(@source [$source] @mut [$($mut)?] @ident [$ident] @rest $($rest)*)
    };

    // parse types and values
    // tuple
    (@source [$source:ident] @mut [$($mut:tt)?] @ident [$ident:tt] @rest ($($t:tt)+) $( $rest:tt )*) => {
        let $($mut)? $ident = $crate::parse!($source >> ( $($t)+ ));

        $crate::bind!(@source [$source] @rest $($rest)*)
    };
    // vec
    (@source [$source:ident] @mut [$($mut:tt)?] @ident [$ident:tt] @rest [$($t:tt)+] $( $rest:tt )*) => {
        let $($mut)? $ident = $crate::parse!($source >> [ $($t)+ ]);

        $crate::bind!(@source [$source] @rest $($rest)*)
    };
    // single item
    (@source [$source:ident] @mut [$($mut:tt)?] @ident [$ident:tt] @rest $t:ty, $( $rest:tt )*) => {
        let $($mut)? $ident = $crate::parse!($source >> $t);

        $crate::bind!(@source [$source] @rest $($rest)*);
    };
    (@source [$source:ident] @mut [$($mut:tt)?] @ident [$ident:tt] @rest $t:ty)=> {
        let $($mut)? $ident = $crate::parse!($source >> $t);

        $crate::bind!(@source [$source] @rest);
    };

    // bug!
    (@source [$source:ident] $( $rest:tt )*) => {
        std::compile_error!("failed to parse")
    };

    ($source:ident >> $( $rest:tt )*) => {
        $crate::bind!(@source [$source] @rest $($rest)*)
    };
}

#[cfg(test)]
mod parses {
    use super::bind;

    use super::FastInput;

    #[test]
    fn single() {
        let mut input = FastInput::new(&b"1"[..]);
        bind!( input >> x: u8, );
        assert_eq!(x, 1)
    }

    #[test]
    fn mutability() {
        let mut input = FastInput::new(&b"1 2"[..]);
        bind!( input >> mut x: u8, y: u8);
        x += 1;
        assert_eq!(x, y)
    }

    #[test]
    fn vec() {
        let mut input = FastInput::new(&b"1 2"[..]);
        bind!( input >> x: [u8; 2], );
        assert_eq!(x, vec![1, 2])
    }

    #[test]
    fn tuple() {
        let mut input = FastInput::new(&b"1 2"[..]);
        bind!( input >> x: (u8, u8));
        assert_eq!(x, (1, 2))
    }

    #[test]
    fn multiple_variables() {
        let mut input = FastInput::new(&b"1 2 3 4 5 6 "[..]);
        bind!( input >> x: (u8, u8), y: u8, z: [u8; y]);

        assert_eq!(x, (1, 2));
        assert_eq!(y, 3);
        assert_eq!(z, vec![4, 5, 6])
    }
}
