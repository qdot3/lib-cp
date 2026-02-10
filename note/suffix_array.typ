#import "@preview/jaconf:0.6.0": appendix, jaconf
#show: jaconf.with(
  title: "接尾辞配列とその応用",
  authors: "nPk",
  title-en: none,
  authors-en: none,
  // 外観
  paper-columns: 1,
  page-number: "1",
  // フォントサイズ Font size
  font-size-title: 20pt,
  // font-size-title-en: 12pt,
  font-size-authors: 16pt,
  // font-size-authors-en: 12pt,
  // font-size-abstract: 10pt,
  font-size-heading: 16pt,
  font-size-main: 14pt,
  // font-size-bibliography: 9pt,
)

#import "@preview/ctheorems:1.1.3": *
#show: thmrules.with(qed-symbol: $square$)
#let definition = thmbox("definition", "定義", fill: rgb("#eeffee"))
#let problem = thmbox("problem", "問題", fill: rgb("#eeffee"))
#let theorem = thmbox("theorem", "定理", fill: rgb("#eeffee"))
#let algorithm = thmbox("algorithm", "アルゴリズム").with(breakable: true)
#let corollary = thmbox("corollary", "系", fill: rgb("#eeffee"))
#let lemma = thmplain("lemma", "補題")
#let proof = thmplain("proof", "証明", separator: [#h(0.9em)])

#import "@preview/drafting:0.2.2": *

#import "@preview/codly:1.3.0": *
#import "@preview/codly-languages:0.1.1": *
#show: codly-init.with()
#codly(
  fill: white,
  breakable: true,
)
#show raw.where(block: false): box.with(
  fill: luma(240),
  inset: (x: 3pt, y: 0pt),
  outset: (y: 3pt),
  radius: 2pt,
)

#show link: underline
#show "、": ", "
#show "。": ". "

= 記法

文字列はSとかき、部分文字列はRustの区間記法にしたがう。
つまり、@tbl:notation の通りにする。
接尾辞配列SAについても同じようにする。

#figure(placement: none)[
  #table(
    columns: (auto, auto, auto),
    "S, T, ..", "文字列", [],
    "|S|", "文字列の長さ", [],
    "S[i]", "文字", [],
    "S[i..j]", "部分文字列", "S[i]を含み、S[j]を含まない",
    "S[i..]", "接尾辞", "S[i]を含む",
  )
]<tbl:notation>

= 接尾辞配列

文字列Sの接尾辞S[i..]は頭文字の添え字iで指定できる。
接尾辞（に対応する添え字）を辞書順に並べたものを接尾辞配列という。
接尾字配列上で二分探索を行うと、任意の文字列Tの全部の出現個所を $O(|T| log|S|)$ で求めることができる。
検索文字列についてオンラインに処理できる点で接尾辞配列はZアルゴリズムやKMP法よりも優れている。

== 文字

Rust言語において文字は ```rust T: Ord``` と抽象化できる。
接尾辞配列においてすべての接尾辞の頭文字がソートされていることを考えれば、計算量は最悪 $Omega(|S| log |S|)$ であることが分かる。
実際に、$O(1)$ の作業メモリを使って、文字列 ```rust &[T]``` の接尾辞配列を $O(|S| log |S|)$ で計算するアルゴリズムが存在する。

文字列を座標圧縮することを考える。
このとき、```rust T = usize``` である。
後述するように、文字列 ```rust &[usize]``` の接尾辞配列は $Theta(|S|)$ で求めることができる。
座標圧縮の計算量は $Theta(|S| log |S|)$ であるから、これによって最悪計算量が悪化することはない。
以下では、文字は ```rust usize``` であるとする。
#footnote[
  より小さな数値型を用いても良いが、アルゴリズムが複雑になってしまう。
  本稿で紹介するアルゴリズムでは ```rust Vec<T>``` の最大容量が ```rust isize::MAX``` であることを利用した最適化を施しており、副次的にアルゴリズムが単純になった。
  最上位ビットをフラグに使っているだけなので、$2^31$ 文字以下ならば ```rust u32``` でも十分だが、型レベルの保証は得られない。
]

== 接尾辞の性質

=== L型とS型

#definition[
  $"S[i..]" gt.lt "S[i+1..]"$ が成り立つとき、S[i]をL型・S型の接尾辞という。
  末尾の番兵はS型であるとする。
  同様に、L型・S型の接尾辞の頭文字もL型・S型とする。
]

#theorem[
  $"S[i]" gt.lt "S[i+1]"$ のとき、S[i]はL型・S型である。
  $"S[i]" = "S[i+1]"$ のとき、両者の型は一致する。
]

#proof[
  前者は自明。
  $"S[i..k-1]" = "S[i+1..k]"$が成り立つ最大の $k$ を考える。
  これは１種類の文字からなるLCPの長さである。
  S[i+1..]がL型・S型のとき、$"S[k-1]" gt.lt "S[k]"$ が成り立つ。
]

#corollary[
  番兵はS型なので、文字列を末尾から走査することで全部の文字の型を$Theta(|S|)$で判定できる。
  L型文字の最上位ビットを立てておけば、以降は $O(1)$ で判定できる。
]

=== 誘導ソート

#theorem[
  S型接尾辞がソート済みならば、L型接尾辞も線形時間でソートできる。
  逆も成り立つ。
]

#proof[
]

#inline-note[
  - L型とS型
  - 誘導ソート
  - LMS部分列とLMS接尾辞のソート
]

== 非再帰アルゴリズム

紹介したアルゴリズムの再帰木は鎖型になっている。
行きがけと帰りがけでループを２つ用意すると非再帰で書ける。

== 制約の緩和

本稿では座標圧縮を仮定したが、文字列の種類が $O(|S|)$ であれば、線形時間で接尾辞配列をもとめるアルゴリズムが存在する。
ASCII文字やUnicode文字など、コンピューターで利用できる文字の種類は定数なので、接尾辞配列を線形時間で計算できる。

= 参考文献

== 接尾辞配列の線形時間アルゴリズム

- LI, Zhize; LI, Jian; HUO, Hongwei. Optimal in-place suffix sorting. Information and Computation, 2022, 285: 104818. #link("https://doi.org/10.1016/j.ic.2021.104818")
- NONG, Ge; ZHANG, Sen; CHAN, Wai Hong. Two efficient algorithms for linear time suffix array construction. IEEE transactions on computers, 2010, 60.10: 1471-1484. #link("https://doi.org/10.1109/TC.2010.188")
