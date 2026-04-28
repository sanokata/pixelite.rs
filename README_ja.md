# pixelite

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![AI Assisted](https://img.shields.io/badge/AI-Assisted-blue.svg)](#)

`pixelite` は、画像をドット絵に変換するための Rust 製コマンドラインツールです。
キャラクター向けのクリーンなエッジ処理や、背景向けの滑らかな減色処理など、用途に応じた最適な画像処理パイプラインを提供します。

## 特徴

- **2つの処理モード**:
  - `Character`: 輪郭を鮮明に保ち、透過処理を適切に扱うためのモード。孤立点の除去や L 字補正、縁取りなどの後処理を適用します。
  - `Background`: グラデーションを滑らかに保つためのモード。ディザリングやガウスぼかしを用いた自然な減色を行います。
- **スマートクロップ**: 画像の中央を正方形に自動で切り抜く機能を搭載。
- **高度な画像処理**: KMeans 法による最適なパレット選定と、Rec. 709 重み付けに基づいた輝度計算。

## インストール

Rust がインストールされている環境であれば、`cargo` を使用してビルド・インストールできます。

```bash
# リポジトリをクローン
git clone https://github.com/sanokata/pixelite.rs.git
cd pixelite.rs

# ビルドとインストール
cargo install --path .
```

## 使い方

### 基本的なコマンド

```bash
# 画像を 64x64 のドット絵に変換
pixelite mosaic --input photo.jpg --output out.png --size 64
```

### クロップしてキャラクター風に処理

```bash
# 中央を正方形にクロップし、32x32 でキャラクターモードを適用
pixelite mosaic -i input.png -o output.png -s 32 -m character --crop
```

### オプション一覧

`pixelite mosaic --help` で詳細なヘルプを表示できます。

| オプション | 短縮形 | デフォルト | 説明 |
| :--- | :--- | :--- | :--- |
| `--input` | `-i` | (必須) | 入力画像のパス |
| `--output` | `-o` | (必須) | 出力画像のパス（PNG 推奨） |
| `--size` | `-s` | `64` | 長辺の解像度（ドット数） |
| `--colors` | `-c` | `16` | 使用する最大色数 |
| `--mode` | `-m` | `character` | 処理モード (`character` または `background`) |
| `--crop` | | `false` | 画像の中央を正方形にクロップするか |
| `--verbose`| `-v` | `false` | 詳細なログを表示（グローバルオプション） |

## 開発

### テストの実行

```bash
cargo test
```

## ライセンス

このプロジェクトは [MIT License](LICENSE) の下で公開されています。

---

**Note**: 本プロジェクトの開発には生成 AI による支援を活用していますが、すべてのコードのレビュー、テスト、および最終的な品質に対する責任は作者に帰属します。
