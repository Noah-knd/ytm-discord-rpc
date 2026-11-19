# YouTube Music Discord RPC

Windows で YouTube Music（ブラウザ・PWA・ダウンロード済み楽曲の再生など）を聴いているときに、Spotify 連携のように Discord のプロフィールへ **「YouTube Music を再生中（Listening）」** として表示する軽量な Rust 製常駐アプリです。

## 主な特徴

- **インストーラーを実行するだけですぐ使える**: 面倒な初期設定やコマンド操作は不要です。
- **ジャケット画像を自動表示**: 再生中の曲名・アーティスト名からジャケット画像を自動取得して表示します（見つからない曲やオフライン時はアイコンにフォールバックします）。
- **再生バー（タイムスタンプ）対応**: 曲の再生位置と残り時間をリアルタイムに表示し、一時停止・停止時は自動でステータスを消去します。
- **軽量・安全設計**: Discord クライアントを改造する Mod は使用せず、Windows 標準のメディア情報（SMTC）と Discord 公式のローカル IPC 通信のみで動作します。
- **Windows 自動起動対応**: インストール時に自動起動をオンにしておけば、PC を再起動しても自動でバックグラウンド常駐します。

## 動作環境

- **OS**: Windows 10 / Windows 11（64-bit）
- **Discord**: Windows デスクトップアプリ版 Discord（ブラウザ版 Discord では動作しません）

## インストール方法

1. **[Releases ページ（最新版）](https://github.com/Noah-knd/ytm-discord-rpc/releases/tag/v0.2.0)** から **`ytm-discord-rpc-setup.exe`** をダウンロードします。
2. ダウンロードした `ytm-discord-rpc-setup.exe` をダブルクリックしてインストールします。
   - インストール途中で「Windows 起動時に自動でバックグラウンド起動する」にチェックを入れておくと便利です。
3. デスクトップ版 Discord を起動し、左下の **ユーザー設定（歯車アイコン）** → **アクティビティのプライバシー** から **「現在のアクティビティをステータスに表示する」** がオンになっていることを確認します。
4. YouTube Music で曲を再生すると、数秒後に Discord のプロフィールへ表示されます。

### 「Windows によって PC が保護されました」と表示された場合

本アプリは個人開発のオープンソースソフトウェアであり、有料のコード署名証明書を付与していないため、初回実行時に Microsoft Defender SmartScreen の警告画面が表示されることがあります。

ソースコードはこのリポジトリで全公開しています。内容をご確認のうえ、警告画面の **「詳細情報」** → **「実行」** をクリックしてインストールを進めてください。

## よくある質問（FAQ）

### 曲を再生しても Discord に表示されません
以下の 4 点を確認してください。

| 確認項目 | 対処法 |
| :--- | :--- |
| **Discord の種類** | ブラウザ版ではなく、インストール済みの Windows アプリ版 Discord を起動してください。 |
| **オンライン状態** | Discord のステータスが「オンライン状態を表示しない」になっているとアクティビティは表示されません。 |
| **プライバシー設定** | Discord の「アクティビティのプライバシー」で「現在のアクティビティをステータスに表示する」をオンにしてください。 |
| **Windows のメディア認識** | 再生中に `Win + A`（クイック設定）を開き、Windows 側のメディアパネルに曲名が表示されているか確認してください。 |

解決しない場合は、`Win + R` キーを押して `%LOCALAPPDATA%\Programs\ytm-discord-rpc` を開き、`ytm-discord-rpc.log` の内容を確認してください。

### PC をシャットダウン・再起動しても自動で動きますか？
はい。インストール時に「Windows 起動時に自動でバックグラウンド起動する」をオンにしていれば、PC 起動時に自動で裏側で立ち上がり、あとから Discord を開いて曲を流すだけで表示されます。

### 特定のアプリ（YouTube Music のみ）に絞り込みたい / 自分のカスタム ID を使いたい
スタートメニューの **「YouTube Music Discord RPC の設定 (config.txt)」**（または `%LOCALAPPDATA%\Programs\ytm-discord-rpc\config.txt`）をメモ帳で編集し、アプリを再起動してください。

```text
# 自分で作成したカスタム Application ID で上書きしたい場合のみ入力
CLIENT_ID=

# アイコン名を変更したい場合のみ入力
LARGE_IMAGE=

# 対象アプリを絞り込む場合はカンマ区切りで指定（ログ内の app: の値を参照）
# 例: APP_FILTERS=chrome,msedge,youtube
APP_FILTERS=
```

### アンインストール方法
Windows の **「設定」** → **「アプリ」** → **「インストールされているアプリ」** から **YouTube Music Discord RPC** を選んでアンインストールできます（自動起動の登録も同時に削除されます）。

## 開発者向け（ソースからのビルド）

```powershell
cargo build --release
```

インストーラー（`dist/ytm-discord-rpc-setup.exe`）を作成する場合は、Inno Setup 6 をインストールした状態で以下を実行します。

```powershell
iscc installer.iss
```

## 免責事項

本ソフトウェアは非公式のオープンソースプロジェクトであり、Google LLC、YouTube Music、および Discord Inc. とは一切関係ありません。
