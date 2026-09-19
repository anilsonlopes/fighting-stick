#ifndef AppVersion
  #error AppVersion must be supplied with /DAppVersion=...
#endif

[Setup]
AppId=FightingStickMidi
AppName=Fighting Stick MIDI
AppVersion={#AppVersion}
DefaultDirName={userpf}\Fighting Stick MIDI
DefaultGroupName=Fighting Stick MIDI
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
SetupIconFile=..\apps\desktop\assets\icon.ico
OutputDir=..\dist
OutputBaseFilename=FightingStickMidi-Setup-{#AppVersion}
Compression=lzma
SolidCompression=yes
UninstallDisplayIcon={app}\FightingStickMidi.exe

[Languages]
Name: "brazilianportuguese"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"

[Files]
Source: "..\target\release\fighting-stick-desktop.exe"; DestDir: "{app}"; DestName: "FightingStickMidi.exe"; Flags: ignoreversion

[Tasks]
Name: "desktopicon"; Description: "Criar um atalho na área de trabalho"; GroupDescription: "Atalhos adicionais:"

[Icons]
Name: "{group}\Fighting Stick MIDI"; Filename: "{app}\FightingStickMidi.exe"; IconFilename: "{app}\FightingStickMidi.exe"
Name: "{userdesktop}\Fighting Stick MIDI"; Filename: "{app}\FightingStickMidi.exe"; IconFilename: "{app}\FightingStickMidi.exe"; Tasks: desktopicon
