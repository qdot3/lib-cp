use factorial::Factorial;
use fps::FPS;
use mint::Mint;
use proconio::input;

fn main() {
    const MOD: u32 = 998_244_353;
    input! { r: usize, g: usize, b: usize, k: usize, x: usize, y: usize, z: usize, }

    let mut red = FPS::<Mint<MOD>>::from(vec![Mint::new(0); r + g + b + 1]);
    let mut green = red.clone();
    let mut blue = red.clone();

    let f = Factorial::<MOD>::with_inverse(r + g + b);
    for i in k - y..=r {
        red[i] = f.choose(r, i)
    }
    for i in k - z..=g {
        green[i] = f.choose(g, i)
    }
    for i in k - x..=b {
        blue[i] = f.choose(b, i)
    }

    let prod = blue * red * green;

    println!("{}", prod[k])
}
