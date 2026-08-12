#ifndef MyAppVersion
  #define MyAppVersion "0.2.0"
#endif
#ifndef MyAppExe
  #error MyAppExe must point to the release executable
#endif
#ifndef MyOutputDir
  #define MyOutputDir "."
#endif
#ifndef MyArchitecture
  #define MyArchitecture "x86_64"
#endif
#ifndef MyArchitecturesAllowed
  #define MyArchitecturesAllowed "x64compatible"
#endif

[Setup]
AppId={{5FC04DF2-839E-447D-B1CA-13521E91957A}
AppName=ZSUI Component Gallery
AppVersion={#MyAppVersion}
AppPublisher=ZSUI
AppPublisherURL=https://github.com/qiu7824/zsui
AppSupportURL=https://github.com/qiu7824/zsui/issues
DefaultDirName={autopf}\ZSUI Component Gallery
DefaultGroupName=ZSUI Component Gallery
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed={#MyArchitecturesAllowed}
ArchitecturesInstallIn64BitMode={#MyArchitecturesAllowed}
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
OutputDir={#MyOutputDir}
OutputBaseFilename=ZSUI-Component-Gallery-v{#MyAppVersion}-windows-{#MyArchitecture}-setup
UninstallDisplayIcon={app}\ZSUI Component Gallery.exe
SetupLogging=yes

[Languages]
Name: "chinesesimplified"; MessagesFile: "compiler:Languages\ChineseSimplified.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
Source: "{#MyAppExe}"; DestDir: "{app}"; DestName: "ZSUI Component Gallery.exe"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\ZSUI Component Gallery"; Filename: "{app}\ZSUI Component Gallery.exe"
Name: "{autodesktop}\ZSUI Component Gallery"; Filename: "{app}\ZSUI Component Gallery.exe"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "创建桌面快捷方式 / Create a desktop shortcut"; GroupDescription: "附加图标 / Additional icons:"

[Run]
Filename: "{app}\ZSUI Component Gallery.exe"; Description: "启动 ZSUI Component Gallery / Launch ZSUI Component Gallery"; Flags: nowait postinstall skipifsilent
