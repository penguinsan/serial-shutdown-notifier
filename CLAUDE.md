# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**bit-zeus-win** - Windowsシャットダウン時にシリアルポート経由で文字列を送信するWindowsサービス

### 技術スタック
- 言語: Rust (Edition 2021)
- 主要クレート:
  - `windows-service`: Windowsサービス実装
  - `serialport`: シリアルポート通信
  - `serde`/`toml`: 設定ファイル管理

## Development Commands

### ビルド
```bash
cargo build --release
```

### テスト実行（シリアルポート接続テスト）
```bash
cargo run --release -- test
```

### サービスのインストール（管理者権限必要）
```cmd
target\release\bit-zeus-win.exe install
```

### サービスのアンインストール（管理者権限必要）
```cmd
target\release\bit-zeus-win.exe uninstall
```

## Architecture

### モジュール構成
- `main.rs`: エントリポイント、コマンドライン引数処理、サービスインストール/アンインストール
- `service.rs`: Windowsサービスの実装、シャットダウンイベントハンドリング
- `serial.rs`: シリアルポート通信の実装
- `config.rs`: TOML設定ファイルの読み込み/保存

### 動作フロー
1. サービス起動時: 設定ファイル（config.toml）を読み込み、待機状態に
2. シャットダウンイベント受信: `ServiceControl::Stop` または `ServiceControl::Preshutdown`
3. シリアルポート接続: 設定に基づいてCOMポートをオープン
4. メッセージ送信: 設定された文字列をシリアルポートに送信
5. クリーンアップ: ポートをクローズしてサービス終了

### 設定ファイル（config.toml）
デフォルト設定:
- ポート: COM1
- ボーレート: 9600bps
- データビット: 8
- パリティ: None
- ストップビット: 1
- 送信文字列: "SHUTDOWN\r" (CR付き)

## Notes

- 管理者権限が必要: サービスのインストール/アンインストール、実行時
- シャットダウンイベントを確実に捕捉するため、`ServiceControlAccept::PRESHUTDOWN`を使用
- 設定ファイルは実行ファイルと同じディレクトリに配置する必要がある
