; Inno Setup スクリプト: YouTube Music Discord RPC (Rust)
#define MyAppName "YouTube Music Discord RPC"
#define MyAppVersion "0.1.0"
#define MyAppExeName "ytm-discord-rpc.exe"

[Setup]
AppId={{8A7C9F12-4E2B-4F31-9B2D-6D7C5E8A1024}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
DefaultDirName={autopf}\ytm-discord-rpc
DefaultGroupName={#MyAppName}
PrivilegesRequired=lowest
OutputDir=dist
OutputBaseFilename=ytm-discord-rpc-setup
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
DisableProgramGroupPage=yes

[Languages]
Name: "japanese"; MessagesFile: "compiler:Languages\Japanese.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "autostart"; Description: "Windows 起動時に自動でバックグラウンド起動する"; GroupDescription: "追加設定:"

[Files]
Source: "target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "YtmDiscordRpc"; ValueData: """{app}\{#MyAppExeName}"""; Flags: uninsdeletevalue; Tasks: autostart

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autoprograms}\{#MyAppName} の設定 (config.txt)"; Filename: "{app}\config.txt"

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{#MyAppName} を今すぐ起動する"; Flags: nowait postinstall skipifsilent

[Code]
var
  ConfigPage: TInputQueryWizardPage;

procedure InitializeWizard;
begin
  ConfigPage := CreateInputQueryPage(
    wpSelectTasks,
    'Discord 設定',
    'Discord Developer Portal で作成したアプリの情報を入力してください',
    'あとからインストールフォルダ内の config.txt でいつでも変更できます。'
  );
  ConfigPage.Add('Discord Application ID (Client ID):', False);
  ConfigPage.Add('アイコン名 (Art Assets に登録したキー・任意):', False);
  ConfigPage.Values[0] := 'YOUR_DISCORD_APPLICATION_ID';
  ConfigPage.Values[1] := 'ytm_logo';
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  ConfigPath: String;
  ConfigContent: String;
begin
  if CurStep = ssPostInstall then
  begin
    ConfigPath := ExpandConstant('{app}\config.txt');
    ConfigContent :=
      '# Discord Developer Portal の Application ID (Client ID)' + #13#10 +
      'CLIENT_ID=' + Trim(ConfigPage.Values[0]) + #13#10 + #13#10 +
      '# Art Assets のアイコン名（未登録なら空欄）' + #13#10 +
      'LARGE_IMAGE=' + Trim(ConfigPage.Values[1]) + #13#10 + #13#10 +
      '# 対象アプリを絞る場合はカンマ区切りで指定（空欄なら再生中のメディア全般）' + #13#10 +
      'APP_FILTERS=' + #13#10;
    SaveStringToFile(ConfigPath, ConfigContent, False);
  end;
end;
