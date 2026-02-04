# 競技プログラミング用環境

## テスト方法

`playground`以外は`--lib`クレートなので下記のコマンドでよい。

```ps1
cargo run
```

他にパッケージを作ってしまうと、曖昧性が生じるので注意。

## 提出方法

`cargo equip`で自作クレートをバンドルして提出する。
`workspace`ルートで下記のエイリアスを実行すると、バンドルされたファイルがクリップボードにコピーされる。

```ps1
cargo equip-atcoder | Set-Clipboard
```

オプションを変更したい場合は`.cargo/config.toml`を編集する。
