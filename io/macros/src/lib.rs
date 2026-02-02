// テストに必要
#[allow(unused_imports)]
use input::FastInput;

#[macro_export]
macro_rules! f_read_value {
    (@source [$source:expr] @rest $(,)?) => {};

    // accept only single types
    (@source [$source:expr] @rest [$($item:tt)+] $(,)?) => {
        $crate::f_read_value!(@vec @source [$source] @item [$($item)+] @rest)
    };
    (@source [$source:expr] @rest ($($item:tt)+) $(,)?) => {
        $crate::f_read_value!(@tuple @source [$source] @item [$($item)+] @rest)
    };
    (@source [$source:expr] @rest Bytes $(with capacity $len:expr)? $(,)?) => {{
        let mut bytes = Vec::with_capacity(0 $(+ TryInto::<usize>::try_into($len).unwrap())?);
        $source.next_bytes(&mut bytes).unwrap();
        bytes
    }};
    (@source [$source:expr] @rest String $(with capacity $len:expr)? $(,)?) => {{
        String::from_utf8(
            $crate::f_read_value!(@source [$source] @rest Bytes $(with capacity $len)?)
        ).unwrap()
    }};
    (@source [$source:expr] @rest $item:ty $(,)?) => {
        $source.next_token::<$item>().unwrap()
    };

    // parse vec items
    (@vec @source [$source:expr] @item [[$($item:tt)+] ; $len:expr] @rest) => {
        (0..$len).into_iter().map(|_|
            $crate::f_read_value!(@source [$source] @rest [$($item)+])
        ).collect::<Vec<_>>()
    };
    (@vec @source [$source:expr] @item [($($item:tt)+) ; $len:expr] @rest) => {
        (0..$len).into_iter().map(|_|
            $crate::f_read_value!(@source [$source] @rest ($($item)+))
        ).collect::<Vec<_>>()
    };
    (@vec @source [$source:expr] @item [$item:ty ; $len:expr] @rest) => {
        (0..$len).into_iter().map(|_|
            $crate::f_read_value!(@source [$source] @rest $item)
        ).collect::<Vec<_>>()
    };

    // parse tuple items
    (@tuple @source [$source:expr] @item [$($item:tt),+  $(,)?] @rest) => {
        ($(
            $crate::f_read_value!(@source [$source] @rest $item ),
        )+)
    };

    // bug!
    (@source [$source:expr] $( $rest:tt )*) => {
        std::compile_error!("failed to parse")
    };

    // interface
    ($source:expr => $( $rest:tt )*) => {
        $crate::f_read_value!(@source [$source] @rest $($rest)*)
    };
}

#[cfg(test)]
mod parse_single_value {
    use super::f_read_value;

    use input::FastInput;

    #[test]
    fn vec() {
        let mut input = FastInput::new(&b"1 2"[..]);
        let x = f_read_value!(input => [u8; 2], );
        assert_eq!(x, vec![1, 2]);
    }

    #[test]
    fn vec_with_runtime_specified_len() {
        let mut input = FastInput::new(&b"3 1 2 3"[..]);
        let n = f_read_value!(input => u8);
        let x = f_read_value!(input => [u8; n], );
        assert_eq!(x, vec![1, 2, 3]);
    }

    #[test]
    fn nested_vec1() {
        let mut input = FastInput::new(&b"1 2 3 4"[..]);
        let x = f_read_value!(input => [[u8; 2]; 2], );
        assert_eq!(x, vec![vec![1, 2], vec![3, 4]]);
    }

    #[test]
    fn nested_vec2() {
        let mut input = FastInput::new(&b"1 2 3 4 5 6 7 8"[..]);
        let x = f_read_value!(input => [[[u8; 2]; 2]; 2], );
        assert_eq!(
            x,
            vec![vec![vec![1, 2], vec![3, 4]], vec![vec![5, 6], vec![7, 8]]]
        );
    }

    #[test]
    fn tuple() {
        let mut input = FastInput::new(&b"1 2"[..]);
        let x = f_read_value!(input => (u8, u8,),);
        assert_eq!(x, (1, 2));
    }

    #[test]
    fn nested_tuple1() {
        let mut input = FastInput::new(&b"1 2 3"[..]);
        let x = f_read_value!(input => (u8, (u8, u8),),);
        assert_eq!(x, (1, (2, 3)));
    }

    #[test]
    fn nested_tuple2() {
        let mut input = FastInput::new(&b"1 2 3"[..]);
        let x = f_read_value!(input => ((u8, u8), u8, ),);
        assert_eq!(x, ((1, 2), 3));
    }

    #[test]
    fn nested_tuple3() {
        let mut input = FastInput::new(&b"1 2 3 4 5 6"[..]);
        let x = f_read_value!(input => (((u8, u8), u8, (u8)), (u8, u8)),);
        assert_eq!(x, (((1, 2), 3, (4,)), (5, 6)));
    }

    #[test]
    fn tuple_in_vec() {
        let mut input = FastInput::new(&b"1 2 3 4"[..]);
        let x = f_read_value!(input => [(u8, u8); 2]);
        assert_eq!(x, vec![(1, 2), (3, 4)])
    }

    #[test]
    fn vec_in_tuple() {
        let mut input = FastInput::new(&b"1 2 3 4 5"[..]);
        let x = f_read_value!(input => ([u8; 2], [u8; 3]));
        assert_eq!(x, (vec![1, 2], vec![3, 4, 5]))
    }

    #[test]
    fn bytes() {
        let mut input = FastInput::new(&b"abcde"[..]);
        let x = f_read_value!(input => Bytes);
        assert_eq!(x, b"abcde")
    }

    #[test]
    fn bytes_with_capacity() {
        let mut input = FastInput::new(&b"5 abcde"[..]);
        let n = f_read_value!(input => u8);
        let x = f_read_value!(input => Bytes with capacity n);
        assert_eq!(x, b"abcde")
    }
    #[test]

    fn string() {
        let mut input = FastInput::new(&b"abcde"[..]);
        let x = f_read_value!(input => String);
        assert_eq!(x, "abcde".to_string())
    }
}

#[macro_export]
macro_rules! f_input {
    // terminator
    (@source [$source:expr] @rest) => {};

    // strip leading commas
    (@source [$source:expr] @rest , $($rest:tt)*) => {
        $crate::f_input!(@source [$source] @rest $($rest)*)
    };

    // parse mutability
    (@source [$source:expr] @rest mut $( $rest:tt )*) => {
        $crate::f_input!(@source [$source] @mut [mut] @rest $($rest)*)
    };
    (@source [$source:expr] @rest $( $rest:tt )*) => {
        $crate::f_input!(@source [$source] @mut [] @rest $($rest)*)
    };

    // parse identifier
    (@source [$source:expr] @mut [$($mut:tt)?] @rest $ident:tt : $( $rest:tt )*) => {
        $crate::f_input!(@source [$source] @mut [$($mut)?] @ident [$ident] @rest $($rest)*)
    };

    // parse types and values
    // tuple
    (@source [$source:expr] @mut [$($mut:tt)?] @ident [$ident:tt] @rest ($($t:tt)+) $( $rest:tt )*) => {
        let $($mut)? $ident = $crate::f_read_value!($source => ( $($t)+ ));

        $crate::f_input!(@source [$source] @rest $($rest)*)
    };
    // vec
    (@source [$source:expr] @mut [$($mut:tt)?] @ident [$ident:tt] @rest [$($t:tt)+] $( $rest:tt )*) => {
        let $($mut)? $ident = $crate::f_read_value!($source => [ $($t)+ ]);

        $crate::f_input!(@source [$source] @rest $($rest)*)
    };
    // single item
    (@source [$source:expr] @mut [$($mut:tt)?] @ident [$ident:tt] @rest $t:ty, $( $rest:tt )*) => {
        let $($mut)? $ident = $crate::f_read_value!($source => $t);

        $crate::f_input!(@source [$source] @rest $($rest)*);
    };
    (@source [$source:expr] @mut [$($mut:tt)?] @ident [$ident:tt] @rest $t:ty $(,)?)=> {
        let $($mut)? $ident = $crate::f_read_value!($source => $t);

        $crate::f_input!(@source [$source] @rest);
    };

    // bug!
    (@source [$source:expr] $( $rest:tt )*) => {
        std::compile_error!("failed to parse")
    };

    ($source:expr => $( $rest:tt )*) => {
        $crate::f_input!(@source [$source] @rest $($rest)*)
    };
}

#[cfg(test)]
mod parse_multiple_values {
    use super::f_input;

    use input::FastInput;

    #[test]
    fn single() {
        let mut input = FastInput::new(&b"1"[..]);
        f_input!( input => x: u8, );
        assert_eq!(x, 1)
    }

    #[test]
    fn mutability() {
        let mut input = FastInput::new(&b"1 2"[..]);
        f_input!( input => mut x: u8, y: u8);
        x += 1;
        assert_eq!(x, y)
    }

    #[test]
    fn vec() {
        let mut input = FastInput::new(&b"1 2"[..]);
        f_input!( input => x: [u8; 2], );
        assert_eq!(x, vec![1, 2])
    }

    #[test]
    fn tuple() {
        let mut input = FastInput::new(&b"1 2"[..]);
        f_input!( input => x: (u8, u8));
        assert_eq!(x, (1, 2))
    }

    #[test]
    fn multiple_variables() {
        let mut input = FastInput::new(&b"1 2 3 4 5 6 "[..]);
        f_input!( input => x: (u8, u8), y: u8, z: [u8; y]);

        assert_eq!(x, (1, 2));
        assert_eq!(y, 3);
        assert_eq!(z, vec![4, 5, 6])
    }
}
