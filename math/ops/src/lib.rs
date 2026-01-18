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

    impl<T1: Idempotent, T2: Idempotent> Idempotent for (T1, T2) {}
    impl<T1: Commutative, T2: Commutative> Commutative for (T1, T2) {}
}

// TODO: メタ変数式が安定化されたらタプルについても自動実装する。
// 参考：<https://doc.rust-lang.org/src/core/tuple.rs.html>
impl<T1: SemiGroup, T2: SemiGroup> SemiGroup for (T1, T2) {
    type Set = (T1::Set, T2::Set);

    fn op(lhs: Self::Set, rhs: Self::Set) -> Self::Set {
        (T1::op(lhs.0, rhs.0), T2::op(lhs.1, rhs.1))
    }
}

impl<T1: SemiGroup, T2: SemiGroup, T3: SemiGroup> SemiGroup for (T1, T2, T3) {
    type Set = (T1::Set, T2::Set, T3::Set);

    fn op(lhs: Self::Set, rhs: Self::Set) -> Self::Set {
        (
            T1::op(lhs.0, rhs.0),
            T2::op(lhs.1, rhs.1),
            T3::op(lhs.2, rhs.2),
        )
    }
}

impl<T1: Identity, T2: Identity> Identity for (T1, T2) {
    fn id() -> Self::Set {
        (T1::id(), T2::id())
    }
}

impl<T1: Identity, T2: Identity, T3: Identity> Identity for (T1, T2, T3) {
    fn id() -> Self::Set {
        (T1::id(), T2::id(), T3::id())
    }
}

impl<T1: Inverse, T2: Inverse> Inverse for (T1, T2) {
    fn inv(x: Self::Set) -> Self::Set {
        (T1::inv(x.0), T2::inv(x.1))
    }
}

impl<T1: Inverse, T2: Inverse, T3: Inverse> Inverse for (T1, T2, T3) {
    fn inv(x: Self::Set) -> Self::Set {
        (T1::inv(x.0), T2::inv(x.1), T3::inv(x.2))
    }
}

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
