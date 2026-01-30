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
    // TODO: anyhow を使う
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
