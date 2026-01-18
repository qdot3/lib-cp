#import "@preview/jaconf:0.6.0": appendix, jaconf
#show: jaconf.with(
  title: "数論変換の手引き",
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

#show "、": ", "
#show "。": ". "

= イントロダクション
== 組み合わせと多項式の関係

#problem[
  ２つの数列 ${a_i}_(i=0)^(n-1), {b_i}_(i=0)^(m-1)$ が与えられる。
  ${c_k}_(k=0)^(n+m-2) := {sum_(i+j=k) a_i b_j}_(k=0)^(n+m-2)$ を求めよ。
]<problem:1>

#problem[
  ２つの有限次元の多項式 $f(x) = sum_(i=0)^(n-1) a_i x^i, g(x) = sum_(i=0)^(m-1) b_i x^i$ が与えられる。
  これらの積 $h(x) = (f * g)(x) = sum_(i=0)^(n+m-2) c_i x^i$ を求めよ。
]<problem:2>

@problem:1 を愚直に解くと計算量は $Theta(n m)$ です。
差分計算や分割統治などの簡単な高速化もないように思われます。
@problem:2 はこれと完全に同じ問題ですが、解析学の結果を利用できます。

#definition("離散フーリエ変換")[
  $CC$ 上の $(n-1)$ 次多項式 $f$、$CC$ の原始 $n$ 乗根を $omega$ とする。
  このとき、$f$ の離散フーリエ変換を次のように定める。
  $
    cal(F)[f(x)] = sum_(i=0)^(n-1) f(omega^i) x^i
  $
  また、$f$ の離散フーリエ逆変換を次のように定める。
  $
    cal(F)^(-1)[f(x)] = 1/n sum_(i=0)^(n-1) f(omega^(-i)) x^i
  $
]

#theorem[
  $f, g$ を $CC$ 上の $(n-1)$ 次多項式とする。
  次が成り立つ。
  + $cal(F)[cal(F)^(-1)[f]] = cal(F)^(-1)[cal(F)[f]] = f$
  + $cal(F)[f * g] = cal(F)[f] cal(F)[g]$
]<thm:fft-convolution-rel>

#proof[
  + $
               cal(F)[cal(F)^(-1)[f(x)]] & = sum_(i=0)^(n-1) (1/n sum_(j=0)^(n-1) f(omega^(-j)) omega^(i j)) x^i \
      therefore
      cal(F)[cal(F)^(-1)[f(omega^(-k))]] & = 1/n sum_(j=0)^(n-1) f(omega^(-j)) n delta_(j,k)
                                           = f(omega^(-k)) space (0 <= forall k < n)
    $
    $(n-1)$ 次多項式の異なる $n$ 個の代表点が一致しているので、$cal(F)[cal(F)^(-1)[f]] = f$ が結論できる。
    $cal(F)^(-1)[cal(F)[f]] = f$ も同様にして証明できる。
    #qedhere
  + $
      cal(F)[(f * g)(x)] & = sum_(k=0)^(2n-2) (f * g)(omega^k) x^k \
                         & = sum_(k=0)^(2n-2) sum_(i+j=k) a_i b_(j) omega^(k) x^(k) \
                         & = (sum_(i=0)^(n-1) a_i omega^i x^i) (sum_(j=0)^(n-1) b_j omega^j x^j) \
                         & = cal(F)[f(x)] cal(F)[g(x)]
    $
    #qedhere
]

@thm:fft-convolution-rel より、関数の積を求める方法が得られます。

#corollary("関数の積の計算")[
  次のようにして、関数の積を計算できる。
  #block(above: 1em)[
    + ゼロ埋めにより $f, g$ を $(n+m-2)$ 次の多項式に拡張し、離散フーリエ変換を施す。
    + $CC$の原始 $(n+m-1)$ 乗根 $w$ について、$cal(F)[f(omega^(-i))] cal(F)[g(omega^(-i))] space (0 <= i < n+m-1)$ を計算する。
    + 離散フーリエ逆変換により $(f * g)(x)$ を得る。
  ]
]<corollary:product>

@corollary:product では係数の列から代表点の列を計算し、項毎に積をとって新しい代表点の列を求め、さらに畳み込み後の係数列を求めています。
離散フーリエ変換・逆変換の定義から、一方を求めるアルゴリズムがあれば、パラメーターを変えるだけで他方を求めるアルゴリズムも得られることが分かります。

== 形式的冪級数

#inline-note[
  形式的冪級数の話
]

== 参考資料

+ #link(
    "https://www.math.sci.hokudai.ac.jp/~wakate/mcyr/2025/pdf/Taiga%20Kanetaka.pdf",
  )[一般の基数の数論変換アルゴリズムの比較]
+ #link(
    "https://maspypy.com/%e5%a4%9a%e9%a0%85%e5%bc%8f%e3%83%bb%e5%bd%a2%e5%bc%8f%e7%9a%84%e3%81%b9%e3%81%8d%e7%b4%9a%e6%95%b0%e6%95%b0%e3%81%88%e4%b8%8a%e3%81%92%e3%81%a8%e3%81%ae%e5%af%be%e5%bf%9c%e4%bb%98%e3%81%91",
  )[[多項式・形式的べき級数]（１）数え上げとの対応付け
  ]

= 離散フーリエ変換と数論変換

浮動小数の計算には誤差がつきものです。
これを解決するために$CC$ではなく$ZZ"/"p ZZ$で計算することを考えます。
これを数論変換と言います。

#inline-note[
  + 原始根
  + 中国剰余定理による復元
  + NTT friendly prime number
]

= Cooley-Tukey 型アルゴリズム
== 時間間引き

$(n-1)$ 次多項式 $f$ の偶数次の項だけ集めたものを$f_e$、奇数次の項だけを集めたものを$f_o$とかくと、$f(x) = f_e (x^2) + x f_o (x^2)$ が成り立ちます。
原始 $n$ 乗根を $w_n$ とします。
$n$ が偶数のとき、$omega_n^2 = omega_(n/2)$ であることに注意すると、$       f(omega_n^k) & = f_e (omega_(n/2)^k) + omega_n^k f_o (omega_(n/2)^k) \
f(omega_n^(k+n/2)) & = f_e (omega_(n/2)^k) - omega_n^(k) f_o (omega_(n/2)^k) $ <eq:ntt-dit-daq> が成り立ちます。
ここで、$omega_n^(n/2) = -1, omega_(n/2)^(n/2) = 1$ の関係を使いました。
@eq:ntt-dit-daq は分轄統治で数論変換を実行できることを意味しています。
計算量の漸化式は、$ T(n) = 2 T(n/2) + Theta(n) $なので、$T(n) = Theta(n log n)$ です。
差分計算ですべての $omega_n^k$ を $Theta(n)$ で計算できることに注意してください。
このアルゴリズムを時間間引きと言います。

以下では $n$ が2冪であると仮定します。
ゼロ埋めによって、いつでも2冪にできます。

#theorem[
  関数の積を $Theta(n log n)$ で計算できる。
]

#proof[@corollary:product より明らか。#qedhere]

== ビット反転順序

メモリを節約するために時間間引きをその場で計算したいです。
サイズ $n/2$ の数論変換の $k$ 番目の代表点から、サイズ $n$ の数論変換の $k$ 番目と $k+n/2$ 番目の代表点が得られることから、配列の前半に偶数次の結果を格納し、後半にの結果を奇数次の結果を格納すればよいことが分かります。
形式的には ```rust (i & 1, i >> 1)``` でソートされていればよいです。
再帰的に計算することを考えると、はじめに ```rust i.reverse()``` でソートしてしまえばよいです。
これをビット反転順序といいます。
以上より、追加の作業メモリが不要になりました。

#algorithm("時間間引きアルゴリズム")[
  ```rust
  fn ntt_t<T>(values: &mut [T] /* in bit reversed order */) {
    assert!(values.len().is_power_of_two());

    let mut width = 1;
    while (width << 1) <= values.len() {
      let dw = /* 原始 width 乗根 */;
      for pair in values.chunks_exact_mut(width << 1) {
        let (prefix, suffix) = pair.split_at_mut(width);
        let mut w = 1;
        for i in 0..width {
          (prefix[i], suffix[i]) = (
            prefix[i] + w * suffix[i],
            prefix[i] - w * suffix[i],
          );
          w += dw;
        }
      }
      width <<= 1;
    }
  }
  ```
]<algo:ntt-dit-v1>

== 最適化

=== 掛け算の削減

$omega_n^k$ を回転因子と言います。
@algo:ntt-dit-v1 で回転因子は高々 $n$ 種類しか登場しませんが、コードの15行目で $n log n$ 回の計算をしています。
そこで、コードの2番目と3番目のループを入れ替えます。
1番目のループの各イテレーションで `width` 回ずつ回転因子を計算すればよいので、回転因子の計算回数は $2n-1$ 回になります。

回転因子の計算回数を減らすことに成功しましたが、メモリへのアクセスがシーケンシャルではなくなってしまいました。
これを解決するために、各 `pair` の `prefix` と `suffix` の $i$ 番目をまとめてしまいたいです。
再帰のベースケースを考えます。登場する回転因子は $omega_2^0 = 1$ だけなので、正順の配列を前半と後半で分ければシーケンシャルアクセスが実現できます。
一段上の再帰過程では、配列を4等分して前から順にペアをとります。
このようなペアの取り方は @algo:ntt-dit-v1 でのペアの取り方の逆になっています。
ビット反転順序のビット反転順序は正順なので、改良版のアルゴリズムは正順の入力からビット反転順序の出力を与えます。

#algorithm("時間間引きアルゴリズム・修正版")[
  正順の入力から、ビット反転順序の出力を得る。
  ```rust
  fn ntt_t<T>(values: &mut [T]) {
    assert!(values.len().is_power_of_two());

    let mut width = value.len() >> 1;
    while width > 0 {
      for pair in values.chunks_exact_mut(width << 1) {
        let w = /* 適切な回転因子 */;
        let (prefix, suffix) = pair.split_at_mut(width);
        for i in 0..width {
          (prefix[i], suffix[i]) = (
            prefix[i] + w * suffix[i],
            prefix[i] - w * suffix[i],
          );
        }
      }
      width >>= 1
    }
  }
  ```
]<algo:ntt-dit-v2>

回転因子の定義から、ビット反転順序の数論変換を得る際には回転因子もビット反転順序で登場します。

=== 回転因子の差分計算

@algo:ntt-dit-v2 では回転因子がビット反転順序で登場します。
メモ化するためには $Theta(n)$ の作業メモリが必要なので、差分計算したいです。
@algo:ntt-dit-v2 の2番目のループを考えます。
ループの開始時点で回転因子はいつも $w_n^0 = 1$ です。
１番目のループのインデックスを $i$、２番目のループのインデックスを $j$ とおきます。
回転因子の漸化式は $Omega_(i,j+1) = Omega_(i, j) omega_(n 2^(-i))^("reverse"(i, j+1) - "reverse"(i, j))$ です。
ここで、$"reverse"(i, j)$ は $(i-1)$ ビット整数としての $j$ のビット反転です。
たとえば、$j = ****1011_((2))$ とすると $j+1 = ****1100_((2))$ なので、$ "reverse"(i, j+1) - "reverse"(i, j) & = 0011****_((2)) - 1101****_((2)) \
                                    & = 2^(i-4) - 2^(i-3) - 2^(i-2) \
           therefore Omega_(i, j+1) & = Omega_(i, j) omega_4^(-1) omega_8^(-1) omega_(16)^(1) $とかけます。
この係数は明らかに ```rust i.trailing_ones()``` で決まるので、$Theta(log n)$ の作業メモリで差分計算ができます。
ワードサイズ程度の大きさの数値で十分な場合、現実的な時間でコンパイル時計算をすることができます。

#algorithm("時間間引きアルゴリズム・修正版2")[
  正順の入力から、ビット反転順序の出力を得る。
  ```rust
  fn ntt_t<T>(values: &mut [T]) {
    assert!(values.len().is_power_of_two());

    let mut width = value.len() >> 1;
    while width > 0 {
      let mut w = 1;
      for (i, pair) in values.chunks_exact_mut(width << 1).enumerate() {
        let (prefix, suffix) = pair.split_at_mut(width);
        for i in 0..width {
          (prefix[i], suffix[i]) = (
            prefix[i] + w * suffix[i],
            prefix[i] - w * suffix[i],
          );
        }
        // `RATE`はコンパイル時に計算しておく
        w *= RATE[i.trailing_ones() as usize];
      }
      width >>= 1
    }
  }
  ```
]<algo:ntt-dit-v3>

=== 並べ替えの削除

@algo:ntt-dit-v3 より正順の係数列から、代表点の列をビット反転順序で得ることができます。
@algo:ntt-dit-v1 で数論逆変換を行うと、ビット反転順序の代表点の列から正順の係数列を得ることができます。
以上より、ビット反転順序に並べ替えることなしに関数の積を計算できます。
しかしながら、@algo:ntt-dit-v1 は最適化されていません。

== 周波数間引き

ビット反転順序の入力をうけとり、正順の出力を返す数論変換アルゴリズムがあれば、最適化を進めることができます。
時間引きアルゴリズムは $f(k)$ と $f(k+n/2)$ を部分問題から求めます。
部分問題から $f(2k)$ と $f(2k + 1)$ を求めるアルゴリズムがあれば、それが所望のアルゴリズムです。
$f(x) = sum_(i=0)^(n/2-1) (a_i + a_(i+n/2) x^(n/2)) x^i$ より、$   f(omega_n^(2k)) & = sum_(i=0)^(n/2-1) (a_i + a_(i + n/2)) omega_(n/2)^(i k) \
f(omega_n^(2k+1)) & = sum_(i=0)^(n/2-1) ((a_i - a_(i + n/2)) omega_n^i) omega_(n/2)^(i k) $を得ます。
これより、新たな分割統治アルゴリズムが得られます。
計算量は $Theta(n log n)$ です。
これを周波数間引きアルゴリズムといいます。
時間間引きアルゴリズムでは部分問題の出力から元の数論変換を計算していました。
一方で、周波数間引きアルゴリズムでは部分問題の入力を計算していくと自動的に数論変換が完了しています。

周波数間引きアルゴリズムをその場で実行するには、部分問題の解が交互に並んでいればよいです。
$a_i$ と $a_(i+n/2)$ が隣接しているので、ビット反転順序の入力から正順の出力が得られることが分かります。
また、各部分問題に対応する配列の $i$ 番目がまとまっているので、時間間引きアルゴリズムで検討した回転因子の計算回数やメモリアクセスの最適化がされています。

#algorithm("周波数間引きアルゴリズム")[
  ビット反転順序の入力から正順の出力を得る。
  ```rust
  fn ntt_f<T>(values: &mut [T] /* in bit-reversed order*/) {
    assert!(values.len().is_power_of_two());

    let mut width = 1;
    while (width << 1) <= values.len() {
      let mut w = 1;
      for (i, pair) in values.chunks_exact_mut(width << 1).enumerate() {
        let (prefix, suffix) = pair.split_at_mut(width);
        for i in 0..width {
          (prefix[i], suffix[i]) = (
            prefix[i] + suffix[i],
            (prefix[i] - suffix[i]) * w,
          );
        }
        // `RATE`はコンパイル時に計算しておく
        w *= RATE[i.trailing_ones() as usize];
      }
      width <<= 1
    }
  }
  ```
]<algo:ntt-dif>

== 最適化（その２）

@algo:ntt-dit-v3 と @algo:ntt-dif を組み合わせることで、関数の積を求めるアルゴリズムを最適化できました。
追加の最適化手法をいくつか述べます。

=== 掛け算の削除

2番目のループの回転因子は1から始まります。
この掛け算は容易に削除でき、条件分岐も必要ありません。
掛け算を $3(n-1)$ 回分削除できました。

=== SIMD命令の活用

3番目のループではSIMD命令を活用できます。
SIMD命令に除算はないので、モンゴメリ剰余乗算などを活用して計算する必要があります。
法はコンパイル時に指定できるので、必要なパラメーターを関連定数として与えておくとよいです。

== まとめ

$n$ が2冪のときの数論変換を $Theta(n log n)$ で計算するアルゴリズムを得ました。
掛け算やメモリアクセスの回数が少なくなるように最適化しました。
また、SIMD命令を活用してさらに高速化することが分かりました。

#inline-note[
  ゼロ埋めを最適化すれば、メモリ使用量を削減できる。
]

== 参考資料
#show link: underline
+ #link("https://tayu0110.hatenablog.com/entry/2023/05/06/023244")[爆速なNTTを実装したい]
+ #link(
    "https://www.kurims.kyoto-u.ac.jp/~ooura/fftman/index.html",
  )[FFT (高速フーリエ・コサイン・サイン変換) の概略と設計法]
+ #link(
    "https://www.math.sci.hokudai.ac.jp/~wakate/mcyr/2025/pdf/Taiga%20Kanetaka.pdf",
  )[一般の基数の数論変換アルゴリズムの比較]
