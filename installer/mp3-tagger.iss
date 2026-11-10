; MP3 Tagger - Inno Setup installer
;
; Compiled by .github/workflows/release.yml from the project root, for example:
;   ISCC.exe /DMyAppVersion=0.1.0 /DMyVersionInfo=0.1.0.0 /DMyArch=x64 ^
;            /DMySourceExe=...\target\x86_64-pc-windows-msvc\release\mp3-tagger.exe ^
;            /DMyOutputBaseFilename=MP3Tagger-Setup-0.1.0-x64 installer\mp3-tagger.iss

#ifndef MyAppVersion
  #define MyAppVersion "0.0.0"
#endif
#ifndef MyVersionInfo
  #define MyVersionInfo "0.0.0.0"
#endif
#ifndef MyArch
  #define MyArch "x64"
#endif
#ifndef MySourceExe
  #define MySourceExe "..\target\release\mp3-tagger.exe"
#endif
#ifndef MyOutputBaseFilename
  #define MyOutputBaseFilename "MP3Tagger-Setup"
#endif

[Setup]
AppId={{6E1C0A6F-6E1D-4D4B-9E2C-8E0F4A6B7C11}
AppName=MP3 Tagger
AppVersion={#MyAppVersion}
AppVerName=MP3 Tagger {#MyAppVersion}
AppPublisher=MP3 Tagger
AppPublisherURL=https://github.com/borneelphukan/mp3-tagger
AppSupportURL=https://github.com/borneelphukan/mp3-tagger/issues
AppUpdatesURL=https://github.com/borneelphukan/mp3-tagger/releases
DefaultDirName={autopf}\MP3 Tagger
DefaultGroupName=MP3 Tagger
DisableProgramGroupPage=yes
OutputDir=..\target
OutputBaseFilename={#MyOutputBaseFilename}
SetupIconFile=..\assets\icon.ico
UninstallDisplayIcon={app}\mp3-tagger.exe
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
RestartApplications=no
VersionInfoVersion={#MyVersionInfo}
VersionInfoProductVersion={#MyVersionInfo}
VersionInfoProductName=MP3 Tagger
VersionInfoCompany=MP3 Tagger
VersionInfoDescription=MP3 Tagger Setup
#if MyArch == "x64"
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
#elif MyArch == "arm64"
ArchitecturesAllowed=arm64
ArchitecturesInstallIn64BitMode=arm64
#else
ArchitecturesAllowed=x86compatible
#endif

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#MySourceExe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\assets\icon.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}\docs"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}\docs"; Flags: ignoreversion skipifsourcedoesntexist

[Icons]
Name: "{group}\MP3 Tagger"; Filename: "{app}\mp3-tagger.exe"
Name: "{group}\Documentation"; Filename: "{app}\docs\README.md"
Name: "{group}\Uninstall MP3 Tagger"; Filename: "{uninstallexe}"
Name: "{autodesktop}\MP3 Tagger"; Filename: "{app}\mp3-tagger.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\mp3-tagger.exe"; Description: "{cm:LaunchProgram,MP3 Tagger}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
Type: filesandordirs; Name: "{app}"
