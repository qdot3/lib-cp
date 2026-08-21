use std::num::NonZero;

use num_integer::{ExtendedGcd, Integer};

pub fn crt(mod_rem: &[(i64, i64)]) -> Option<(i64, NonZero<i64>)> {
    // `x = 0 (mod 1)` for any integer `x`
    let mut m0 = 1;
    let mut r0 = 0;

    for (mi, ri) in mod_rem.into_iter().map(|(m, r)| {
        let m = m.abs();
        let r = r.rem_euclid(m);
        (m, r)
    }) {
        // r0 + t[i] m0 = r[i] (mod m[i])
        let (ExtendedGcd { gcd, x: _, y: im0 }, lcm) = i64::extended_gcd_lcm(&mi, &m0);
        let (div, rem) = (ri - r0).div_rem(&gcd);
        if rem != 0 {
            return None;
        }
        // t[i] (m0 / g) = (r[i] - r0) / g (mod (m[i] / g))
        let t = (div.rem_euclid(mi / gcd) * im0).rem_euclid(mi / gcd);

        r0 = (r0 + t * m0).rem_euclid(lcm);
        m0 = lcm;
    }

    NonZero::new(m0).map(|m0| (r0, m0))
}
