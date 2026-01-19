use fps::FPS;
use itertools::Itertools;
use mint::Mint;
use proconio::input;

fn main() {
    const MOD: u32 = 998_244_353;
    input! { n: usize, m: usize, a: [u32; n], b: [u32; m], }

    let f = FPS::<Mint<MOD>>::from_iter(a.into_iter().map(|v| Mint::new(v)));
    let g = FPS::from_iter(b.into_iter().map(|v| Mint::new(v)));

    let fg = f * g;

    println!("{}", fg.coefficients(n + m - 1).iter().join(" "))
}
