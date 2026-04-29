# pixelite

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![AI Assisted](https://img.shields.io/badge/AI-Assisted-blue.svg)](#)

`pixelite` は、画像をドット絵に変換するための Rust 製コマンドラインツールです。
スプライト向けのクリーンなエッジ処理や、写真向けの滑らかな減色処理など、用途に応じた最適な画像処理パイプラインを提供します。

## 特徴

- **2つの処理モード** (`mosaic` コマンド):
  - `sharp`: 輪郭を鮮明に保ち、透過処理を適切に扱うためのモード。孤立点の除去・L 字補正 (ピクセルパーフェクト)・縁取りなどの後処理を適用します。
  - `smooth`: グラデーションを滑らかに保つためのモード。Floyd-Steinberg ディザリングとガウスぼかしを用いた自然な減色を行います。
- **ディザリング制御**: `--dither` オプションでモードのデフォルトを上書き（`none` / `floyd-steinberg` / `ordered`）。
- **パレット管理** (`palette` コマンド): 画像から最適なカラーパレットを抽出したり、既存のパレットを別の画像に適用したりできます。
- **スマートクロップ**: 処理前に画像の中央を正方形に自動で切り抜く機能。
- **高度な色量子化**: KMeans 法による最適なパレット選定と、Rec. 709 重み付けに基づいた輝度計算。

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

### `mosaic` — 画像をドット絵に変換

```bash
# 画像を 64x64 のドット絵に変換（sharp モード、デフォルト）
pixelite mosaic --input photo.jpg --output out.png --size 64

# 中央を正方形にクロップし、32x32 で sharp モードを適用
pixelite mosaic -i input.png -o output.png -s 32 -m sharp --crop

# 背景向けに smooth モードで処理し、4倍に拡大して出力
pixelite mosaic -i background.jpg -o bg_pixel.png -s 128 -m smooth -S 4

# sharp モードで ordered ディザリングを使用（モードのデフォルトを上書き）
pixelite mosaic -i sprite.png -o sprite_pixel.png -m sharp --dither ordered
```

#### オプション一覧

`pixelite mosaic --help` で詳細なヘルプを表示できます。

| オプション | 短縮形 | デフォルト | 説明 |
| :--- | :--- | :--- | :--- |
| `--input` | `-i` | (必須) | 入力画像のパス |
| `--output` | `-o` | (必須) | 出力画像のパス（PNG 推奨） |
| `--size` | `-s` | `64` | 長辺の解像度（ドット数） |
| `--colors` | `-c` | `16` | 使用する最大色数 |
| `--mode` | `-m` | `sharp` | 処理モード（`sharp` または `smooth`） |
| `--dither` | `-d` | (モード依存) | ディザリング方式の上書き（`none` / `floyd-steinberg` / `ordered`） |
| `--crop` | | `false` | 処理前に画像の中央を正方形にクロップするか |
| `--scale` | `-S` | `1` | エクスポート倍率（1ドットあたりのピクセル数） |
| `--verbose` | `-v` | `false` | 詳細なログを表示（グローバルオプション） |

---

### `palette` — カラーパレットの抽出・適用

#### 画像からパレットを抽出

```bash
# 16色を抽出して PNG スウォッチとして保存
pixelite palette extract --input photo.jpg --output palette.png --colors 16

# GIMP パレット形式（.gpl）で保存
pixelite palette extract -i photo.jpg -o palette.gpl -c 8
```

| オプション | 短縮形 | デフォルト | 説明 |
| :--- | :--- | :--- | :--- |
| `--input` | `-i` | (必須) | 入力画像のパス |
| `--output` | `-o` | (必須) | 出力パレットのパス（`.png` または `.gpl`） |
| `--colors` | `-c` | `16` | 抽出する色数 |

#### パレットを画像に適用

```bash
# ディザリングなしでパレットを適用
pixelite palette apply --input photo.jpg --palette palette.png --output result.png

# Floyd-Steinberg ディザリングで適用
pixelite palette apply -i photo.jpg -p palette.gpl -o result.png --method floyd-steinberg

# ordered（Bayer 4x4）ディザリングで適用
pixelite palette apply -i photo.jpg -p palette.png -o result.png --method ordered
```

| オプション | 短縮形 | デフォルト | 説明 |
| :--- | :--- | :--- | :--- |
| `--input` | `-i` | (必須) | 入力画像のパス |
| `--palette` | `-p` | (必須) | パレットファイルのパス（`.png` または `.gpl`） |
| `--output` | `-o` | (必須) | 出力画像のパス |
| `--method` | `-m` | `none` | ディザリング方式（`none` / `floyd-steinberg` / `ordered`） |

## 開発

### テストの実行

```bash
cargo test
```

## ライセンス

このプロジェクトは [MIT License](LICENSE) の下で公開されています。

---

**Note**: 本プロジェクトの開発には生成 AI による支援を活用していますが、すべてのコードのレビュー、テスト、および最終的な品質に対する責任は作者に帰属します。
