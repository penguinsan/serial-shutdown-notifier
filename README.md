# Serial Shutdown Notifier

Windowsシャットダウン時にシリアルポート経由で文字列を送信するWindowsサービスです。

## 機能

- Windowsシャットダウン/再起動時に自動でシリアルポートへメッセージを送信
- 設定ファイル（TOML形式）によるカスタマイズ可能
- Windowsサービスとして動作し、バックグラウンドで常駐

## 必要要件

- Windows 10/11 または Windows Server
- Rust開発環境（ビルド時のみ）
- 管理者権限（サービスのインストール・実行時）

## ビルド方法

```bash
cargo build --release
```

ビルド後、実行ファイルは `target\release\serial-shutdown-notifier.exe` に生成されます。

## インストール手順

1. ビルドした実行ファイル `serial-shutdown-notifier.exe` を任意のディレクトリにコピー
2. 同じディレクトリに `config.toml` をコピー（または自動生成される）
3. 管理者権限でコマンドプロンプトを開く
4. 以下のコマンドを実行:

```cmd
serial-shutdown-notifier.exe install
```

5. サービスを起動:

```cmd
sc start SerialShutdownNotifier
```

またはサービス管理ツール（services.msc）から「Serial Shutdown Notifier」を起動します。

## アンインストール手順

管理者権限でコマンドプロンプトを開き、以下のコマンドを実行:

```cmd
serial-shutdown-notifier.exe uninstall
```

## 設定ファイル

`config.toml` ファイルで動作をカスタマイズできます:

```toml
[serial]
port = "COM1"           # シリアルポート番号
baud_rate = 9600        # ボーレート
data_bits = 8           # データビット (5, 6, 7, 8)
parity = "None"         # パリティ (None, Odd, Even)
stop_bits = 1           # ストップビット (1, 2)

[message]
shutdown_text = "SHUTDOWN\r"  # 送信する文字列（\r=CR, \n=LF）
```

### エスケープシーケンス

送信文字列では以下のエスケープシーケンスが使用できます:
- `\r` - キャリッジリターン (CR)
- `\n` - ラインフィード (LF)
- `\t` - タブ
- `\\` - バックスラッシュ

## テスト実行

サービスをインストールする前に、シリアルポート接続をテストできます:

```cmd
serial-shutdown-notifier.exe test
```

このコマンドは設定ファイルを読み込み、実際にシリアルポートへメッセージを送信します。

## トラブルシューティング

### サービスが起動しない

- 管理者権限で実行しているか確認
- イベントビューアー（eventvwr.msc）のアプリケーションログを確認

### シリアルポートが開けない

- 指定したCOMポートが存在するか確認（デバイスマネージャー）
- 他のプログラムがポートを使用していないか確認
- ボーレート等の設定が正しいか確認

### シャットダウン時にメッセージが送信されない

- サービスが実行中か確認: `sc query SerialShutdownNotifier`
- ログファイルの確認（現在は標準出力のみ）

## ログ

サービスのログはWindowsイベントログに記録されます。イベントビューアーで確認できます。

## ライセンス

MIT License - 詳細は [LICENSE](LICENSE) ファイルを参照してください。

## 作者

penguinsan
