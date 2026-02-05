pub trait SemiGroup {
    type Set;

    /// 結合法則を満たす閉じた演算
    fn op(lhs: Self::Set, rhs: Self::Set) -> Self::Set;
}

pub trait Identity: SemiGroup {
    /// 単位元
    fn id() -> Self::Set;
}

pub trait Monoid: SemiGroup + Identity {}

impl<T: SemiGroup + Identity> Monoid for T {}

pub trait Inverse: Monoid {
    /// 逆元
    fn inv(x: Self::Set) -> Self::Set;
}

pub trait Group: Monoid + Inverse {}

impl<T: Monoid + Inverse> Group for T {}

pub mod marker {
    use crate::SemiGroup;

    /// 二項演算が冪等性を満たす
    pub trait Idempotent: SemiGroup {}

    /// 二項演算が可換である
    pub trait Commutative: SemiGroup {}
}

macro_rules! tuple_impl {
    ( ($t0:tt, $lhs0:tt, $rhs0:tt, $x0:tt), $( ($t:tt, $lhs:tt, $rhs:tt, $x:tt) ),+ $(,)? ) => {
        impl< $t0 :SemiGroup, $( $t : SemiGroup),+> SemiGroup for ( $t0, $( $t ),+)
        {
            type Set = (<$t0>::Set, $( <$t>::Set ),+);

            fn op(($lhs0, $( $lhs ),+): Self::Set, ($rhs0, $( $rhs ),+): Self::Set) -> Self::Set {
                ( <$t0>::op($lhs0, $rhs0), $( <$t>::op($lhs, $rhs) ),+)
            }
        }

        impl< $t0: Identity, $( $t : Identity ),+> Identity for ( $t0, $( $t ),+)
        {
            fn id() -> Self::Set {
                (<$t0>::id(), $( <$t>::id() ),+)
            }
        }

        impl< $t0: Inverse, $( $t : Inverse ),+> Inverse for ( $t0, $( $t ),+)
        {
            fn inv(($x0, $( $x ),+): Self::Set) -> Self::Set {
                (<$t0>::inv($x0), $( <$t>::inv( $x ) ),+)
            }
        }

        impl< $t0: marker::Commutative, $( $t : marker::Commutative ),+> marker::Commutative for ( $t0, $( $t ),+)
        {}

        impl< $t0: marker::Idempotent, $( $t : marker::Idempotent ),+> marker::Idempotent for ( $t0, $( $t ),+)
        {}

        tuple_impl!( $( ($t, $lhs, $rhs, $x) ),+ );
    };
    // termination
    ( ($t0:tt, $lhs0:tt, $rhs0:tt, $x0:tt) ) => {}
}
tuple_impl!(
    (T1, lhs1, rhs1, x1),
    (T2, lhs2, rhs2, x2),
    (T3, lhs3, rhs3, x3),
    (T4, lhs4, rhs4, x4),
);

mod unit {
    use crate::marker::{Commutative, Idempotent};

    use super::*;

    impl SemiGroup for () {
        type Set = ();

        fn op(_: Self::Set, _: Self::Set) -> Self::Set {
            ()
        }
    }

    impl Identity for () {
        fn id() -> Self::Set {
            ()
        }
    }

    impl Inverse for () {
        fn inv(_: Self::Set) -> Self::Set {
            ()
        }
    }

    impl Commutative for () {}
    impl Idempotent for () {}
}

pub mod ops {
    use num::Zero;

    use super::*;

    use std::{
        marker::PhantomData,
        ops::{Add, Neg},
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Min<T>(PhantomData<T>);

    impl<T: Ord> SemiGroup for Min<T> {
        type Set = T;

        fn op(lhs: Self::Set, rhs: Self::Set) -> Self::Set {
            lhs.min(rhs)
        }
    }

    impl<T: Ord> marker::Idempotent for Min<T> {}
    impl<T: Ord> marker::Commutative for Min<T> {}

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Max<T>(PhantomData<T>);

    impl<T: Ord> SemiGroup for Max<T> {
        type Set = T;

        fn op(lhs: Self::Set, rhs: Self::Set) -> Self::Set {
            lhs.max(rhs)
        }
    }

    impl<T: Ord> marker::Idempotent for Max<T> {}
    impl<T: Ord> marker::Commutative for Max<T> {}

    macro_rules! primitive_min_max_identity_impl {
        ($( $t:ty )+) => {$(
            impl Identity for Min<$t> {
                fn id() -> Self::Set {
                    <$t>::MAX
                }
            }

            impl Identity for Max<$t> {
                fn id() -> Self::Set {
                    <$t>::MIN
                }
            }
        )+};
    }
    primitive_min_max_identity_impl!( u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize );

    /// `+`演算が定義された集合
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Additive<T>(PhantomData<T>);

    impl<T> SemiGroup for Additive<T>
    where
        T: Add<Output = T>,
    {
        type Set = T;

        fn op(lhs: Self::Set, rhs: Self::Set) -> Self::Set {
            lhs + rhs
        }
    }

    impl<T> Identity for Additive<T>
    where
        T: Zero,
        Additive<T>: SemiGroup<Set = T>,
    {
        fn id() -> Self::Set {
            T::zero()
        }
    }

    impl<T> Inverse for Additive<T>
    where
        T: Neg<Output = T>,
        Self: Monoid<Set = T>,
    {
        fn inv(x: Self::Set) -> Self::Set {
            -x
        }
    }

    impl<T> marker::Commutative for Additive<T> where Additive<T>: SemiGroup {}
}
