; MP3 Tagger - NSIS installer
;
; Compiled by .github/workflows/windows-installer.yml from the project root:
;   makensis -DAPP_VERSION=0.1.0 -DAPP_VERSION_NUM=0.1.0.0 ^
;            -DOUTPUT_NAME=MP3Tagger-Setup-0.1.0.exe installer/mp3-tagger.nsi
;
; Expects a native release build at target\release\mp3-tagger.exe

Unicode true
RequestExecutionLevel admin
SetCompressor /SOLID lzma

!include "MUI2.nsh"
!include "FileFunc.nsh"
!include "LogicLib.nsh"

; All File/OutFile paths below are relative to the project root, so make the
; compiler independent of the directory it was invoked from.
!cd "${__FILEDIR__}.."

!ifndef APP_VERSION
  !define APP_VERSION "0.0.0"
!endif
!ifndef APP_VERSION_NUM
  !define APP_VERSION_NUM "0.0.0.0"
!endif
!ifndef OUTPUT_NAME
  !define OUTPUT_NAME "MP3Tagger-Setup-${APP_VERSION}.exe"
!endif

!define PRODUCT_NAME "MP3 Tagger"
!define PUBLISHER "MP3 Tagger"
!define EXE_NAME "mp3-tagger.exe"
!define UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}"
!define APP_KEY "Software\${PRODUCT_NAME}"
!define START_MENU_DIR "${PRODUCT_NAME}"
!define ENV_KEY "SYSTEM\CurrentControlSet\Control\Session Manager\Environment"
!define WM_SETTINGCHANGE 0x001A
!define HWND_BROADCAST 0xFFFF

; $PROGRAMFILES64 is only defined by a 64-bit build of NSIS. When this script
; is compiled by a 32-bit NSIS, fall back to the 32-bit directory.
!ifdef NSIS_WIN64
  !ifndef INSTALLDIR
    !define INSTALLDIR "$PROGRAMFILES64\${PRODUCT_NAME}"
  !endif
!else
  !ifndef INSTALLDIR
    !define INSTALLDIR "$PROGRAMFILES\${PRODUCT_NAME}"
  !endif
!endif

Name "${PRODUCT_NAME}"
OutFile "target\${OUTPUT_NAME}"
BrandingText "${PRODUCT_NAME} ${APP_VERSION}"
InstallDir "${INSTALLDIR}"
InstallDirRegKey HKLM "${APP_KEY}" "InstallDir"
ShowInstDetails show
ShowUninstDetails show

VIProductVersion "${APP_VERSION_NUM}"
VIAddVersionKey "ProductName" "${PRODUCT_NAME}"
VIAddVersionKey "FileDescription" "${PRODUCT_NAME} Setup"
VIAddVersionKey "FileVersion" "${APP_VERSION_NUM}"
VIAddVersionKey "ProductVersion" "${APP_VERSION_NUM}"
VIAddVersionKey "CompanyName" "${PUBLISHER}"
VIAddVersionKey "LegalCopyright" "MIT licensed"

!define MUI_ABORTWARNING
!define MUI_ICON "assets\icon.ico"
!define MUI_UNICON "assets\icon.ico"
!define MUI_FINISHPAGE_RUN "$INSTDIR\${EXE_NAME}"
!define MUI_FINISHPAGE_RUN_TEXT "Launch ${PRODUCT_NAME}"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

!insertmacro MUI_LANGUAGE "English"

Section "${PRODUCT_NAME}" SecMain
  SetOutPath "$INSTDIR"
  File /oname=${EXE_NAME} "target\release\${EXE_NAME}"
  File "assets\icon.ico"
  SetOutPath "$INSTDIR\docs"
  File "README.md"
  SetOutPath "$INSTDIR"

  WriteRegStr HKLM "${APP_KEY}" "InstallDir" "$INSTDIR"
  WriteRegStr HKLM "${APP_KEY}" "Version" "${APP_VERSION}"

  WriteRegStr HKLM "${UNINST_KEY}" "DisplayName" "${PRODUCT_NAME}"
  WriteRegStr HKLM "${UNINST_KEY}" "DisplayVersion" "${APP_VERSION}"
  WriteRegStr HKLM "${UNINST_KEY}" "Publisher" "${PUBLISHER}"
  WriteRegStr HKLM "${UNINST_KEY}" "DisplayIcon" "$INSTDIR\${EXE_NAME}"
  WriteRegStr HKLM "${UNINST_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKLM "${UNINST_KEY}" "InstallDate" "$(GetDateTime -Format=yyyyMMdd)"
  WriteRegStr HKLM "${UNINST_KEY}" "URLInfoAbout" "https://github.com/borneelphukan/mp3-tagger"
  WriteRegStr HKLM "${UNINST_KEY}" "HelpLink" "https://github.com/borneelphukan/mp3-tagger/issues"
  WriteRegStr HKLM "${UNINST_KEY}" "UninstallString" '"$INSTDIR\Uninstall.exe"'
  WriteRegStr HKLM "${UNINST_KEY}" "QuietUninstallString" '"$INSTDIR\Uninstall.exe" /S'
  WriteRegDWORD HKLM "${UNINST_KEY}" "NoModify" 1
  WriteRegDWORD HKLM "${UNINST_KEY}" "NoRepair" 1

  ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
  IntFmt $0 "0x%08X" $0
  WriteRegDWORD HKLM "${UNINST_KEY}" "EstimatedSize" "$0"

  WriteUninstaller "$INSTDIR\Uninstall.exe"

  SetShellVarContext all
  CreateDirectory "$SMPROGRAMS\${START_MENU_DIR}"
  CreateShortcut "$SMPROGRAMS\${START_MENU_DIR}\${PRODUCT_NAME}.lnk" "$INSTDIR\${EXE_NAME}"
  CreateShortcut "$SMPROGRAMS\${START_MENU_DIR}\Uninstall ${PRODUCT_NAME}.lnk" "$INSTDIR\Uninstall.exe"
  CreateShortcut "$SMPROGRAMS\${START_MENU_DIR}\Documentation.lnk" "$INSTDIR\docs\README.md"
SectionEnd

Section /o "Desktop shortcut" SecDesktop
  SetShellVarContext all
  CreateShortcut "$DESKTOP\${PRODUCT_NAME}.lnk" "$INSTDIR\${EXE_NAME}"
SectionEnd

Section /o "Add to command line (PATH)" SecPath
  ReadRegStr $0 HKLM "${ENV_KEY}" "Path"

  ; Remember both the previous value and the value we are about to write, so
  ; that uninstalling can put things back exactly - and only if nothing else
  ; has touched PATH in the meantime.
  ${If} $0 == ""
    WriteRegStr HKLM "${APP_KEY}" "OriginalPath" ""
    WriteRegStr HKLM "${APP_KEY}" "AddedPath" "$INSTDIR"
    WriteRegExpandStr HKLM "${ENV_KEY}" "Path" "$INSTDIR"
  ${Else}
    WriteRegStr HKLM "${APP_KEY}" "OriginalPath" "$0"
    WriteRegStr HKLM "${APP_KEY}" "AddedPath" "$0;$INSTDIR"
    WriteRegExpandStr HKLM "${ENV_KEY}" "Path" "$0;$INSTDIR"
  ${EndIf}

  System::Call 'shell32::SendMessage(i ${HWND_BROADCAST}, i ${WM_SETTINGCHANGE}, i 0, t "Environment") i.r0'
SectionEnd

Section "Uninstall"
  SetShellVarContext all

  ; Restore the machine PATH, but only if it still is exactly the value the
  ; installer wrote. Anything else means someone else changed it, so leave it
  ; alone rather than clobbering their work.
  ReadRegStr $1 HKLM "${ENV_KEY}" "Path"
  ReadRegStr $2 HKLM "${APP_KEY}" "AddedPath"
  ${If} $2 != ""
  ${AndIf} $1 == $2
    ReadRegStr $0 HKLM "${APP_KEY}" "OriginalPath"
    WriteRegExpandStr HKLM "${ENV_KEY}" "Path" "$0"
    System::Call 'shell32::SendMessage(i ${HWND_BROADCAST}, i ${WM_SETTINGCHANGE}, i 0, t "Environment") i.r0'
  ${EndIf}

  Delete "$INSTDIR\${EXE_NAME}"
  Delete "$INSTDIR\icon.ico"
  Delete "$INSTDIR\Uninstall.exe"
  Delete "$INSTDIR\docs\README.md"
  RMDir "$INSTDIR\docs"
  RMDir "$INSTDIR"

  Delete "$DESKTOP\${PRODUCT_NAME}.lnk"
  Delete "$SMPROGRAMS\${START_MENU_DIR}\${PRODUCT_NAME}.lnk"
  Delete "$SMPROGRAMS\${START_MENU_DIR}\Uninstall ${PRODUCT_NAME}.lnk"
  Delete "$SMPROGRAMS\${START_MENU_DIR}\Documentation.lnk"
  RMDir "$SMPROGRAMS\${START_MENU_DIR}"
  DeleteRegKey HKLM "${UNINST_KEY}"
  DeleteRegKey HKLM "${APP_KEY}"
SectionEnd

Function .onInit
  SetShellVarContext all
FunctionEnd

Function un.onInit
  SetShellVarContext all
FunctionEnd
