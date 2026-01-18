# 競技プログラミング用環境

## 提出方法

`cargo equip`で自作クレートをバンドルして提出する。
`workspace`ルートで下記のエイリアスを実行すると、バンドルされたファイルがクリップボードにコピーされる。

```ps1
cargo equip-atcoder | Set-Clipboard
```

オプションを変更したい場合は`.cargo/config.toml`を編集する。
