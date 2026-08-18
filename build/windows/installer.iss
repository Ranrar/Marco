; Marco/Polo Windows installer (Inno Setup 6)
;
; First test build -- compiles the same staged tree build_portable.ps1
; produces (binaries + bundled MSYS2 GTK/WebKit runtime + WebView2 loader +
; assets) into a proper setup.exe instead of a zip.
;
; Deliberately excludes config/ and data/ from the staged tree: a writable
; <exe_dir>\config next to the installed binaries would make
; detect_portable_mode() in marco-shared/src/paths/platform/windows.rs treat
; an installed copy as portable and store settings under {app} instead of
; %APPDATA%\marco. See .dev/new_fun/inno.md for the full design writeup.
;
; Invoke as:
;   ISCC.exe /DAppVersion=0.25.0 /DStagingDir=<path-to-staged-tree> ^
;            /DOutputDir=<path-to-build\installer> ^
;            build\windows\installer.iss

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif
#ifndef StagingDir
  #define StagingDir "..\..\build\windows\temp\markdown-composer-and-viewer_0.0.0_windows_amd64"
#endif
#ifndef OutputDir
  #define OutputDir "..\..\build\installer"
#endif

[Setup]
; Fixed GUID -- do NOT regenerate on future edits. Keeping this stable is
; what makes upgrades replace-in-place instead of installing side-by-side.
AppId={{5716479A-C1C2-449F-A87A-466124577AE8}
AppName=Marco
AppVersion={#AppVersion}
AppPublisher=Kim Skov Rasmussen
AppPublisherURL=https://github.com/Ranrar/Marco
DefaultDirName={autopf}\Marco
DefaultGroupName=Marco
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=commandline dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
LicenseFile=..\..\LICENSE
OutputDir={#OutputDir}
OutputBaseFilename=markdown-composer-and-viewer_{#AppVersion}_windows_amd64_setup
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
UninstallDisplayIcon={app}\marco.exe
DisableProgramGroupPage=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon_marco"; Description: "Create a &desktop shortcut for Marco"; GroupDescription: "Additional shortcuts:"; Flags: unchecked
Name: "desktopicon_polo"; Description: "Create a desktop shortcut for &Polo"; GroupDescription: "Additional shortcuts:"; Flags: unchecked

[Files]
Source: "{#StagingDir}\marco.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StagingDir}\polo.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StagingDir}\*.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StagingDir}\assets\*"; DestDir: "{app}\assets"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "{#StagingDir}\lib\*"; DestDir: "{app}\lib"; Flags: ignoreversion recursesubdirs createallsubdirs; Check: DirExists(ExpandConstant('{#StagingDir}\lib'))
Source: "{#StagingDir}\share\*"; DestDir: "{app}\share"; Flags: ignoreversion recursesubdirs createallsubdirs; Check: DirExists(ExpandConstant('{#StagingDir}\share'))
Source: "..\..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
; NOTE: intentionally NOT copying {#StagingDir}\config, \data, README.txt or
; MANIFEST.txt -- those are portable-package-only artifacts.

[Icons]
Name: "{group}\Marco"; Filename: "{app}\marco.exe"
Name: "{group}\Polo"; Filename: "{app}\polo.exe"
Name: "{group}\Uninstall Marco"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Marco"; Filename: "{app}\marco.exe"; Tasks: desktopicon_marco
Name: "{autodesktop}\Polo"; Filename: "{app}\polo.exe"; Tasks: desktopicon_polo

[Run]
Filename: "{app}\marco.exe"; Description: "Launch Marco"; Flags: nowait postinstall skipifsilent unchecked
