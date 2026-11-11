; XurPorisoy - Inno Setup installer
;
; Compiled by .github/workflows/release.yml from the project root, for example:
;   ISCC.exe /DMyAppVersion=0.1.0 /DMyVersionInfo=0.1.0.0 /DMyArch=x64 ^
;            /DMySourceExe=...\target\x86_64-pc-windows-msvc\release\xur-porisoy.exe ^
;            /DMyOutputBaseFilename=XurPorisoy-Setup-0.1.0-x64 installer\xur-porisoy.iss

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
  #define MySourceExe "..\target\release\xur-porisoy.exe"
#endif
#ifndef MyOutputBaseFilename
  #define MyOutputBaseFilename "XurPorisoy-Setup"
#endif

[Setup]
AppId={{6E1C0A6F-6E1D-4D4B-9E2C-8E0F4A6B7C11}
AppName=XurPorisoy
AppVersion={#MyAppVersion}
AppVerName=XurPorisoy {#MyAppVersion}
AppPublisher=XurPorisoy
AppPublisherURL=https://github.com/borneelphukan/xur-porisoy
AppSupportURL=https://github.com/borneelphukan/xur-porisoy/issues
AppUpdatesURL=https://github.com/borneelphukan/xur-porisoy/releases
DefaultDirName={autopf}\XurPorisoy
DefaultGroupName=XurPorisoy
DisableProgramGroupPage=yes
OutputDir=..\target
OutputBaseFilename={#MyOutputBaseFilename}
SetupIconFile=..\assets\icon.ico
UninstallDisplayIcon={app}\xur-porisoy.exe
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
RestartApplications=no
VersionInfoVersion={#MyVersionInfo}
VersionInfoProductVersion={#MyVersionInfo}
VersionInfoProductName=XurPorisoy
VersionInfoCompany=XurPorisoy
VersionInfoDescription=XurPorisoy Setup
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
Name: "{group}\XurPorisoy"; Filename: "{app}\xur-porisoy.exe"
Name: "{group}\Documentation"; Filename: "{app}\docs\README.md"
Name: "{group}\Uninstall XurPorisoy"; Filename: "{uninstallexe}"
Name: "{autodesktop}\XurPorisoy"; Filename: "{app}\xur-porisoy.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\xur-porisoy.exe"; Description: "{cm:LaunchProgram,XurPorisoy}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
Type: filesandordirs; Name: "{app}"
