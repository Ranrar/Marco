[Setup]
AppName=Marco
AppVersion=0.1.0-beta
DefaultDirName={pf}\Marco
DefaultGroupName=Marco
OutputDir=dist
OutputBaseFilename=MarcoInstaller
Compression=lzma
SolidCompression=yes
UninstallDisplayIcon={app}\marco.exe
Uninstallable=yes
UninstallDisplayName=Uninstall Marco 0.1-beta

[Files]
Source: "dist\marco.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "dist\*.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "dist\share\*"; DestDir: "{app}\share"; Flags: recursesubdirs ignoreversion
Source: "dist\lib\*"; DestDir: "{app}\lib"; Flags: recursesubdirs ignoreversion

[Icons]
Name: "{group}\Marco"; Filename: "{app}\marco.exe"
Name: "{group}\Uninstall Marco"; Filename: "{uninstallexe}"

[Run]
Filename: "{app}\marco.exe"; Description: "Launch Marco"; Flags: nowait postinstall skipifsilent
