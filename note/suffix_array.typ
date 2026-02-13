#import "@preview/jaconf:0.6.0": appendix, jaconf
#show: jaconf.with(
  title: "接尾辞配列とその応用",
  authors: "nPk",
  title-en: none,
  authors-en: none,
  abstract: [

  ],
  abstract-language: "ja",
  keywords: ("suffix array", "in-place"),
  keywords-language: "ja",
  // 外観
  paper-columns: 1,
  page-number: "1",
  // フォントサイズ
  font-size-title: 20pt,
  // font-size-title-en: 12pt,
  font-size-authors: 16pt,
  // font-size-authors-en: 12pt,
  font-size-abstract: 14pt,
  font-size-heading: 16pt,
  font-size-main: 14pt,
  // font-size-bibliography: 14pt,
)

#import "@preview/ctheorems:1.1.3": *
#show: thmrules.with(qed-symbol: $square$)
#let definition = thmbox("definition", "定義", fill: rgb("#eeffee"))
#let problem = thmbox("problem", "問題", fill: rgb("#eeffee"))
#let theorem = thmbox("theorem", "定理", fill: rgb("#eeffee"))
#let algorithm = thmbox("algorithm", "アルゴリズム").with(breakable: true)
#let corollary = thmbox("corollary", "系", fill: rgb("#eeffee"))
#let lemma = thmbox("lemma", "補題", fill: rgb("#eeffee"))
#let proof = thmproof("proof", [*証明*], separator: [#h(0.9em)])

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
    columns: (auto, auto),
    "S, T, ..", "文字列",
    "|S|", "文字列の長さ",
    "S[i]", "文字",
    "S[i..j]、S[i..=j]", "部分文字列",
    "S[i..]", "接尾辞",
  )
]<tbl:notation>

= 接尾辞配列

文字列Sの接尾辞S[i..]は頭文字の添え字iで指定できる。
接尾辞（に対応する添え字）を辞書順に並べたものを接尾辞配列という。
接尾字配列上で二分探索を行うと、任意の文字列Tの全部の出現個所を $O(|T| log|S|)$ で求めることができる。
検索文字列についてオンラインに処理できる点で接尾辞配列はZアルゴリズムやKMP法よりも優れている。

== 文字 <sec:character>

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
  本稿で紹介する実装では ```rust Vec<T>``` の最大容量が ```rust isize::MAX``` であることを利用した最適化を施しており、オリジナルと比べてアルゴリズムが単純になった。
  最上位ビットをフラグに使っているだけなので、$2^31$ 文字以下ならば ```rust u32``` でも十分だが、コンパイル時の保証は得られない。
]

== 理論

２つの接尾辞S[i..]とS[i+1..]は長さが異なるので、大小関係が決まる。
そこで、接尾辞およびその頭文字の型を次のように定義する。

#definition[
  $"S[i..]" gt.lt "S[i+1..]"$ が成り立つとき、S[i]をL型・S型の接尾辞という。
  ただし、末尾の番兵はS型であるとする。
  L型・S型の接尾辞の頭文字もL型・S型とする。
]

#theorem[
  $"S[i]" gt.lt "S[i+1]"$ のとき、S[i]はL型・S型である。
  $"S[i]" = "S[i+1]"$ のとき、両者の型は一致する。
]<thm:type-check>
#proof[
  前者は自明。
  後者については、$"S[i+k]" eq.not "S[i+k+1]"$が成り立つ最小の正整数 $k$ を考えればよい。
]

\
接尾辞配列SAは当然頭文字についてソートされる。
同じ頭文字S[i]をもつ接尾辞たちが記録される部分をバケットS[i]と呼ぶことにする。

#corollary[
  各バケットはL型接尾辞とS型接尾辞で区切られている。
] <cor:bucket-is-partitioned>
#proof[
  頭文字を固定して考える。
  この頭文字をもつ最小のS型接尾辞をS[i..]とかくと、$"S[i] < S[i+1]"$が成り立つ。
  同様に最大のL型接尾辞をS[j..]とかくと、$"S[j] > S[j+1]"$が成り立つ。
  したがって、$"S[i+1] > S[j+1]"$である。
]

\
L型接尾辞S[i..]の性質より、$"S[i..] > S[i+1..]"$が成り立つ。
したがって、バケットS[i]に書き込まれるL型接尾辞の２文字目以降からなる接尾辞はバケットの左側にある。
同様にS型接尾辞の２文字目以降からなる接尾辞は対応するバケットよりも右側にある。
これらの観察より次の定理を得る。

#theorem[
  S型接尾辞がソート済みならば、L型接尾辞も $Theta(|S|)$ でソートできる。
  逆も成り立つ。
]<thm:induced-sort-v0>
#proof[
  L型接尾辞をソートすることにする。
  接尾辞配列を昇順に走査し、初期化済みの要素を探す。
  これをSA[i]とかく。
  もしS[SA[i]-1..]がL型接尾辞ならば対応するバケットに前から詰めて書き込む。
  同じバケットに属する接尾辞はその２文字目以降の部分からなる接尾辞についてもソートされているから、各バケットに書き込まれたL型接尾辞はソートされている。
  バケットソートと累積和でバケットの左端を求めることができるので、アルゴリズムは線形時間で動作する。
  逆も同じように証明できる。
]

\
@thm:induced-sort-v0 の証明ではS型接尾辞のうち、１だけ長い接尾辞がL型であるもののみを利用している。
これより、次の事実を得る。

#definition[
  S[i-1..]がL型であるようなS型接尾辞S[i..]をとくにLMS型（leftmost S type）という。
  文字S[i]についても同様に定義する。
]
#corollary[
  LMS型接尾辞がソートされているとき、接尾辞配列を $Theta(|S|)$ で構築できる。
]<thm:induced-sort-v1>
#proof[
  LMS型接尾辞がソートされているとする。
  @thm:induced-sort-v0 より、接尾辞配列SAを昇順に走査してL型接尾辞をソートできる。
  さらに、接尾辞配列を降順に走査してS型接尾辞をソートすることができる。
]
#lemma[
  LMS型接尾辞は高々 $floor(abs(S)/2)$ 個しかない。
]<lemma:num-lms-suffix>

\
@lemma:num-lms-suffix より考えるべき接尾辞の数を半分以下にできましたが、問題サイズは半分になっていません。
接尾辞の長さが高々 $|S|$ だからです。
LMS型接尾辞を比較することを考えると、２つのLMS型文字に挟まれてできる部分文字列単位で比較すればよいことが分かります。
これを利用して文字列を圧縮し、問題のサイズを半分以下にすることが次の目標です。

#definition[
  部分文字列S[i..=j]のうち、S[i]とS[j]のみがLMS型であるものをLMS型文字列という。
  とくに、番兵もLMS型部分文字列であるとする。
]
#lemma[
  LMS型接尾辞とLMS部分文字列の数は一致する。
]

\
LMS部分文字列を順序を保ったまま新しい文字に置き換えるためには、これらをソートする必要があります。
逆に、LMS部分文字列がソートされていれば隣り合う２つの一致判定をとることで、順序を保ったまま改名することができます。

#theorem[
  LMS部分文字列を $Theta(|S|)$ でソートできる。
]
#proof[
  バケットソートにより、すべてのLMS文字を線形時間でソートできる。
  @thm:induced-sort-v1 の証明と同じようにして、頭文字を除く最初のLMS文字までソートできることが分かる。
  これはすべてのLMS部分文字列を含んでいる。
]
#corollary[
  線形時間で接尾辞配列を構築できる。
]
#proof[
  時間計算量は $T(n) <= T(n/2) + Theta(n) = Theta(n)$ である。
]

== アルゴリズム

本節では線形時間接尾辞配列構築アルゴリズムのin-placeな実装を紹介する。
作業メモリを節約するために、バケットソートや区間幅の管理に接尾辞配列SA用のメモリを利用する。
@sec:character で述べたように、文字の最上位ビットを活用して接尾辞の添え字とカウンターを区別する。
また、文字の型の区別にも利用する。

=== 文字の改名

文字列が座標圧縮されているので、接尾辞配列を使って各文字の登場回数を数えることができる。
#footnote[
  実際には $"S[i]" < |S|$ で十分。
]
累積和をとると、バケットの右端を得ることができる。
@cor:bucket-is-partitioned より、L型文字をバケットの左端、S型文字をその右端の添え字に改名してもよい。

#algorithm[
  ```rust
  fn rename(text: &mut [usize], sa: &mut [usize]) {
    // 文字の登場回数を数える
    sa.fill(0);
    text.iter().for_each(|s| sa[*s] += 1);
    // 累積和をとり、バケットの右端を求める。
    (1..sa.len()).for_each(|i| sa[i] += sa[i-1]);
    // 改名する。
    {
      // 番兵はS型。その１つ前はL型。
      let mut is_s_type = false;
      // 番兵は１つしかないので、改名しても`0`のまま。
      // 直前の要素を型の判別に使うので、更新を遅延する。
      let mut name = [0, 0];
      for i in (0..text.len()).rev().skip(1) {
        if text[i] != text[i+1] {
          is_s_type = text[i] < text[i+1]
        }
        // 遅延を反映する
        if let Some(s) = text.get_mut(i+2) {
          *s = name[i%2]
        }
        // 反映を遅延する
        name[i%2] = if is_s_type {
          sa[text[i]]-1
        } else {
          // L型は最上位ビットを立てておく。
          sa[text[i]-1] | 1usize.rotate_right(1)
        };
      }
      if text.len() >= 2 {
        text[..2].copy_from_slice(&name);
      }
    }
  }
  /// 文字の型を判定する。
  fn is_s_type(s: usize) -> bool {
    s & 1usize.rotate_right(1) == 0
  }
  /// L型文字ならバケットの左端、S型文字ならバケットの右端の添え字を返す。
  fn str_to_ptr(s: usize) -> usize {
    s & (!0 >> 1)
  }
  ```
]

文字の最上位ビットをフラグに使うことで、文字の型をいつでも定数時間で求めることができるようになった。
@thm:type-check から文字の型を判別するにはSを逆順に走査する必要がある。
したがって、@thm:induced-sort-v1 接尾辞配列を操作する際にはこの手法は利用できない。
#footnote[
  別の方法はある。
]
また自由な方向に走査できるので、メモリIOの回数を削減できるかもしれない。

=== LMS文字のソート

LMS部分文字列をソートするために、LMS文字をソートする。
LMS文字をバケットの正しい位置に置く際に注意が必要である。
LMS文字はバケットの右端の情報をもつが、愚直に書き込むと $O(abs(S)^2)$ かかってしまう。

そこで、バケットの右端には次に書き込むべき場所への距離を書き込むことにする。
これはすでに書き込んだ文字の個数に１を加えたものである。
バケット内にLMS文字が１つしかない場合、カウンターは不要なので単に書き込んでしまう。
これと区別するために１を加えた。

LMS文字についてソートされていればよいので、バケット内での順序は自由にしてよい。
@thm:induced-sort-v1 では文字でなく接尾辞の順序を保つので、全体をシフトする必要がある。

#algorithm[
  ```rust
  fn sort_lms_char(text: &mut [usize], sa: &mut [usize]) {
    // LMS文字を数える。
    sa.fill(0);
    text.windows(2).for_each(|s| {
      if !is_s_type(s[0]) && is_s_type(s[1]) {
        sa[str_to_ptr(s[1])] += 1
      }
    });

    // カウンターの最上位ビットを立てて区別する。実際はグローバルな定数。
    const COUNTER_FLAG: usize = 1.rotate_right(1);
    const COUNT_ZERO: usize = COUNTER_FLAG;
    const COUNT_ONE: usize = COUNT_ZERO + 1;
    text.windows(2).enumerate().for_each(|(i, s)| {
      if !is_s_type(s[0]) && is_s_type(s[1]) {
        let ptr = str_to_ptr(s[1]);

        if sa[ptr] == COUNT_ONE {
          // 唯一の要素なので、カウンターは不要
          sa[ptr] = i+1;
        } else if sa[ptr-1] == COUNT_ZERO {
          // バケット内で最初の文字
          sa[ptr-1] = i+1;
          // 引き算だとフラグが消えてしまう
          sa[ptr] = COUNT_ZERO+2;
        } else {
          // バケット内で２文字目以降
          let diff = sa[ptr] - COUNT_ZERO;
          if sa.get(ptr.wrapping_sub(diff)).is_some_and(|sa| *sa == COUNT_ZERO) {
            // 次に書き込む先はバケット内か、L型用のバケット
            sa[ptr-diff] = i+1;
            sa[ptr] += 1;
          } else {
            // 隣のS型用のバケットにはみ出してしまう。
            // バケット内で最後の１文字なので、カウンターを上書きしてよい。
            sa[ptr] = i+1
          }
        }
      }
    });
    // カウンターを除去して、右に１だけシフトする。
    {
      let mut i = sa.len();
      while i > 0 {
        i -= 1;
        if sa[i] > COUNT_ONE {
          // カウンターを最後のLMS文字で上書きする。
          let diff = sa[i] - COUNT_ZERO - 1;
          sa[i] = sa[i - diff];
          sa[i - diff] = COUNT_ZERO;
          // LMS文字がある部分はスキップする。
          i -= diff;
        }
      }
  }
  ```
]

=== 誘導ソート

@thm:induced-sort-v1 の証明で用いたアルゴリズムを誘導ソートという。
誘導ソートはこのアルゴリズムの肝であり、ソート済みのLMS文字からLMS部分文字列の順序を導き、ソートされたLMS型接尾辞から接尾辞配列を誘導する。

#algorithm("誘導ソート")[
  ```rs
  fn induced_sort(text: &mut [usize], sa: &mut [usize]) {
    // LMS型からL型を誘導する
    sort_l_types(text: &mut [usize], sa: &mut [usize]);
    // LMS文字を除去する。番兵は誘導できないので、残しておく。
    sa.iter_mut().skip(1).for_each(|i| {
      // L型以外の文字は除去してよい。
      if text.get(*i).is_some_and(|s| is_s_type(*s)) {
        *i = COUNT_ZERO
      }
    });
    // L型からS型を誘導する
    sort_s_types(text: &mut [usize], sa: &mut [usize]);
  }

  // S型の誘導ソートもほとんど同じだが、SAを降順に走査する。カウンターは残らないので、あとから削除する必要はない。
  fn sort_l_types(text: &mut [usize], sa: &mut [usize]) {
    // LMS文字をソートするときとほとんど同じ。違いはSではなくSAを昇順に走査すること。

    // L型文字の出現数を数える
    text.iter().for_each(|s| {
      if !is_s_type(*s) { sa[str_to_ptr[*s]] += 1 }
    });
    // バケットに昇順に書き込む。
    // バケットの中身を１だけシフトしたときには添え字を進めないので、for ループは使えない。
    let mut i = 0;
    while i < sa.len() {
      // 初期化済みで、１つ長い接尾辞が存在するもの
      if sa[i] > 0 && sa[i] & COUNTER_FLAG == 0 {
        let s = text[sa[i]-1];
        let ptr = str_to_ptr(s);
        // シフトしても影響がなかった場合、同じ操作はしない。
        if !is_s_type(s) && sa[ptr] & COUNTER_FLAG != 0 {
          if sa[ptr] == COUNT_ONE {
            // バケット内で唯一のL型文字
            sa[ptr] = sa[i]-1
          } else if sa[ptr+1] == COUNT_ZERO {
            // バケット内で最初のL型文字
            sa[ptr+1] = sa[i]-1;
            sa[ptr] = COUNT_ZERO + 2;
          } else {
            let diff = sa[ptr] - COUNT_ZERO;
            if sa.get(ptr + diff).is_some_and(|sa| *sa == COUNT_ZERO) {
              // 書き込まれていない場合
              sa[ptr+diff] = sa[i]-1;
              sa[ptr] += 1;
            } else {
              // 書き込まれているなら、バケット内で最後のL型文字
              sa[ptr] = sa[i]-1;
              // 順番を保つ
              sa[ptr..ptr+diff].rotate_left(1);
              // 今いる場所がシフトした場合、添え字をインクリメントしてはけない
              continue;
            }
          }
        }
        i += 1;
      }
    }
    // カウンターを除去して、バケットを左にシフト
    let mut i = 0;
    while i < sa.len() {
      if sa[i] > COUNT_ZERO {
        // カウンターを初期化して左にシフト
        let diff = sa[i] - COUNT_ZERO;
        sa[i] = COUNT_ZERO;
        sa[i..i + diff].rotate_left(1);
        // L型文字が書き込まれている部分はスキップ
        i += diff
      } else {
        i += 1
      }
    }
  }
  ```
]

=== LMS部分文字列のソート

#inline-note[
  アルゴリズムの解説
]

== 非再帰アルゴリズム

紹介したアルゴリズムの再帰木は鎖型になっている。
行きがけと帰りがけでループを２つ用意すると非再帰で書けるはずである。
ただし、$O(log |S|)$ の作業メモリが必要かもしれない。

#inline-note[
  借用規則を守るために ```rust slice.split_at_mut()``` を多用することになる。
  この可変参照を管理するためにスタックを使うかも？
]

== 制約の緩和

本稿では座標圧縮を仮定したが、文字列の種類が $O(|S|)$ であれば、線形時間で接尾辞配列をもとめるアルゴリズムが存在する。
ASCII文字やUnicode文字など、コンピューターで利用できる文字の種類は固定なので、接尾辞配列を線形時間で計算できる。

= LCP配列 <chap:lcp>

= 参考文献

== 接尾辞配列の線形時間アルゴリズム

- LI, Zhize; LI, Jian; HUO, Hongwei. Optimal in-place suffix sorting. Information and Computation, 2022, 285: 104818. #link("https://doi.org/10.1016/j.ic.2021.104818")
- NONG, Ge; ZHANG, Sen; CHAN, Wai Hong. Two efficient algorithms for linear time suffix array construction. IEEE transactions on computers, 2010, 60.10: 1471-1484. #link("https://doi.org/10.1109/TC.2010.188")
