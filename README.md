# YouTube Music Discord RPC (Rust / Windows)

Windows のメディア情報（SMTC）から再生中の曲と再生時間を取得し、Discord に **「〜を再生中（Listening）」** として表示する軽量な自作 Rust 常駐アプリです。

## PCにRustを入れずに `setup.exe` を作る手順（GitHub Actions）

1. **Discord アプリを作成して Application ID を取得**
   - [Discord Developer Portal](https://discord.com/developers/applications) で「New Application」をクリックし、名前を `YouTube Music` にして作成します。
   - 作成画面に表示される **Application ID**（数字の羅列）をコピーしておきます。
   - （任意）左メニューの **Rich Presence -> Art Assets** でアイコン画像をアップロードし、名前を `ytm_logo` にするとジャケット枠に表示されます。

2. **GitHub にこのフォルダ一式をアップロード**
   - GitHub で新しいリポジトリ（Private でも可）を作成し、このプロジェクトの中身をそのままアップロード（push）します。

3. **自動生成された `ytm-discord-rpc-setup.exe` をダウンロード**
   - リポジトリ上部の **Actions** タブを開きます。
   - **Build Windows Setup EXE** の実行が完了（緑のチェック）したらクリックし、ページ下部の **Artifacts** から `ytm-discord-rpc-setup` をダウンロード・解凍します。

4. **インストールして完了**
   - 中に入っている `ytm-discord-rpc-setup.exe` をダブルクリックします。
   - インストール途中で **Discord Application ID** の入力欄が出るので、手順 1 でコピーした ID を貼り付けて完了します。
   - デスクトップ版 Discord を起動した状態で YouTube Music を再生すると、ステータスに表示されます。

## 設定の変更・ログの確認

- スタートメニューの「**YouTube Music Discord RPC の設定 (config.txt)**」から、あとから `CLIENT_ID` や `APP_FILTERS` を変更できます。
- 同じフォルダに `ytm-discord-rpc.log` が出力されるため、特定のアプリ（Chrome/Edge など）だけに絞りたい場合はログ内の `(app: ...)` の文字列を `APP_FILTERS=` に設定してください。
