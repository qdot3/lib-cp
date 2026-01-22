use csr::UndirectedCSR;
use lca::LCA;
use proconio::input;

fn main() {
    input! { n: usize, uv: [(u32, u32); n-1], }

    let csr = UndirectedCSR::new(uv, n as u32 - 1);
    let lca = LCA::try_from((n as u32 - 1, &csr)).unwrap();
}
