use std::{marker::PhantomData, ops::RangeBounds};

use lib_modulo::{Modulus64, Raw64};

#[derive(Debug, Clone)]
pub struct RollingHash<T>
where
    T: Into<u64>,
{
    // 1-based indexing. 0th node is `0`.
    hash: Vec<Raw64>,
    base: Vec<Raw64>,
    modulus: Modulus64,
    ch_ty: PhantomData<T>,
}

impl<T> RollingHash<T>
where
    T: Into<u64>,
{
    pub fn with_capacity(base: u64, odd_modulus: u64, capacity: usize) -> Self {
        let modulus = Modulus64::new(odd_modulus);

        let mut vec_base = Vec::with_capacity(capacity + 1);
        vec_base.push(modulus.residue(1).into_raw());
        vec_base.push(modulus.residue(base).into_raw());

        let mut hash = Vec::with_capacity(capacity + 1);
        hash.push(modulus.residue(0).into_raw());

        Self {
            hash,
            base: vec_base,
            modulus,
            ch_ty: PhantomData,
        }
    }

    pub fn push(&mut self, ch: T) {
        let ch = self.modulus.residue(ch.into());

        let hash = if let Some(&last) = self.hash.last() {
            self.base[1].into_residue(&self.modulus) * last + ch
        } else {
            ch
        };
        self.hash.push(hash.into_raw());
    }

    pub fn get_hash_of_slice<R>(&mut self, range: R) -> Option<Raw64>
    where
        R: RangeBounds<usize>,
    {
        // (Bound::Exclude(l), Bounds::Include(r))
        let l = match range.start_bound() {
            std::ops::Bound::Included(l) => *l,
            std::ops::Bound::Excluded(l) => l.checked_sub(1)?,
            std::ops::Bound::Unbounded => 0,
        };
        let r = match range.end_bound() {
            std::ops::Bound::Included(r) => r.checked_add(1)?,
            std::ops::Bound::Excluded(r) => *r,
            std::ops::Bound::Unbounded => self.hash.len().checked_sub(1)?,
        };

        let w = r.checked_sub(l)?;

        while self.base.len() <= w {
            static MSG: &str = "this is a bug. `self.base.len() >= 2` must hold";

            let last = self.base.last().expect(MSG).into_residue(&self.modulus);
            let next = last * *self.base.get(1).expect(MSG);
            self.base.push(next.into_raw());
        }
        let pow_base = self.base[w].into_residue(&self.modulus);

        let hash = *self.hash.get(r)? - *self.hash.get(l)? * pow_base;
        Some(hash.into_raw())
    }
}
