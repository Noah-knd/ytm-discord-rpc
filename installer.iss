; Inno Setup スクリプト: YouTube Music Discord RPC (Rust / 一般公開向け)
#define MyAppName "YouTube Music Discord RPC"
#define MyAppVersion "0.2.0"
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
Name: "autostart"; Description: "Windows 起動時に自動でバックグラウンド起動する"; GroupDescription: "追加設定:"; Flags: checkedonce

[Files]
Source: "target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "YtmDiscordRpc"; ValueData: """{app}\{#MyAppExeName}"""; Flags: uninsdeletevalue; Tasks: autostart

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autoprograms}\{#MyAppName} の設定 (config.txt)"; Filename: "{app}\config.txt"

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{#MyAppName} を今すぐ起動する"; Flags: nowait postinstall skipifsilent
