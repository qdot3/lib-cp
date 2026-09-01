use std::ops::ControlFlow;

use csr2::{Edge, Undirected, CSR};
use search::Visitor;

/// low linkを計算する。
/// ２つのビューワーがあり、橋と関節点を取得できる。
///
/// # Time complexity
///
/// O(|E| + |V|)
pub fn low_link<W>(
    csr: CSR<W, Undirected>,
    mut bridge: impl FnMut(Edge<&W>),
    mut articulation_point: impl FnMut(usize),
    mut ecc: impl FnMut(&[usize]),
    mut bcc: impl FnMut(&[usize]),
) -> CSR<W, Undirected> {
    let n = csr.num_nodes();
    let mut visitor = Visitor::new(csr);

    // ord: DFS木での訪問順序
    // low: 任意個の木辺と高々１つの後退辺で到達可能な頂点の最小 ord
    let mut ord_low = Vec::with_capacity(n);
    {
        // 初回訪問時に上書きするので初期値は何でもよい。
        let ord_low = ord_low.spare_capacity_mut();
        assert!(ord_low.len() >= n);

        let mut ecc_stack = Vec::with_capacity(n);
        let mut bcc_stack = Vec::with_capacity(n);

        let mut n_visited = 0;
        for i in 0..n {
            if visitor.visited(i) {
                continue;
            }
            ecc_stack.push(i);
            bcc_stack.push(i);
            ord_low[i].write([n_visited; 2]);
            n_visited += 1;

            let mut deg_root = 0;
            let _ = visitor.dfs::<()>(i, |edge| {
                match edge {
                    // 木辺を降る
                    search::DFSTraversal::Descend(edge) => {
                        // DFS木の根が複数の子をもつなら関節点
                        if edge.source == i {
                            deg_root += 1;
                            if deg_root > 1 {
                                articulation_point(i);
                                bcc(&bcc_stack);
                                bcc_stack.truncate(1);
                            }
                        }

                        ecc_stack.push(edge.target);
                        bcc_stack.push(edge.target);
                        ord_low[edge.target].write([n_visited; 2]);
                        n_visited += 1;
                    }
                    // 木辺を昇る
                    search::DFSTraversal::Ascend(edge) => {
                        // SAFETY: 初訪問時に初期化している
                        let t = unsafe { ord_low[edge.target].assume_init() };
                        let s = unsafe { ord_low[edge.source].assume_init_mut() };
                        // target を根とする部分木の探索済みなので、t[1] は確定
                        s[1] = s[1].min(t[1]);

                        // edge を通らずに target から source に行けないので、これは橋
                        if s[0] < t[1] {
                            bridge(edge);
                            // 橋の下流がECC
                            {
                                let i = ecc_stack.iter().rposition(|v| *v == edge.target).unwrap();
                                ecc(&ecc_stack[i..]);
                                ecc_stack.truncate(i);
                            }
                        }
                        // source を通らずにその祖先に到達できないので、これは関節点
                        if edge.source != i && s[0] <= t[1] {
                            articulation_point(edge.source);
                            // 関節点から下流がBCC
                            {
                                let i = bcc_stack.iter().rposition(|v| *v == edge.target).unwrap();
                                // 暗黙に source をもつ。ループ上に関節点が複数ある場合を考えると、
                                // source 位置を探すのは誤りで、他のBCCも取り込んでしまう。
                                bcc_stack.push(edge.source);
                                bcc(&bcc_stack[i..]);
                                bcc_stack.truncate(i);
                            }
                        }
                    }
                    // 後退辺（自己ループを含む）
                    search::DFSTraversal::Glance(edge) => {
                        // SAFETY: 初訪問時に初期化している
                        let t = unsafe { ord_low[edge.target].assume_init() };
                        let s = unsafe { ord_low[edge.source].assume_init_mut() };
                        // 後退辺を通るので target の ord を見る
                        s[1] = s[1].min(t[0]);
                    }
                }

                ControlFlow::Continue(())
            });

            if !ecc_stack.is_empty() {
                ecc(&ecc_stack);
                ecc_stack.clear();
            }
            if deg_root == 0 || bcc_stack.len() > 1 {
                bcc(&bcc_stack);
            }
            bcc_stack.clear();
        }
    }
    // SAFETY: すべての頂点を訪問し、その際に初期化している。
    unsafe { ord_low.set_len(n) };

    visitor.into_csr()
}
