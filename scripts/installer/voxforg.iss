; VoxForg Inno Setup Installer Script
; Produces: VoxForg-Setup-v0.1.0-windows-x64.exe
; Requires Inno Setup 6.x: https://jrsoftware.org/isinfo.php

#define MyAppName "VoxForg"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "VoxForg Contributors"
#define MyAppURL "https://github.com/AMG555/VoxForg"
#define MyAppExeName "voxforg.exe"
#define MyAppDescription "Neural Speech Studio"

[Setup]
AppId={{F7A3B2C1-4D5E-4F6A-B7C8-D9E0A1B2C3D4}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}/issues
AppUpdatesURL={#MyAppURL}/releases
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
LicenseFile=..\..\LICENSE-APACHE
OutputDir=..\..\dist
OutputBaseFilename=VoxForg-Setup-v{#MyAppVersion}-windows-x64
SetupIconFile=..\..\assets\voxforg.ico
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
UninstallDisplayIcon={app}\{#MyAppExeName}
UninstallDisplayName={#MyAppName} {#MyAppVersion}
VersionInfoVersion={#MyAppVersion}
VersionInfoCompany={#MyAppPublisher}
VersionInfoDescription={#MyAppDescription}
VersionInfoProductName={#MyAppName}
MinVersion=10.0
ArchitecturesInstallIn64BitMode=x64compatible

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "quicklaunchicon"; Description: "{cm:CreateQuickLaunchIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked; OnlyBelowVersion: 6.1

[Files]
Source: "..\..\target\release\voxforg.exe"; DestDir: "{app}"; DestName: "{#MyAppExeName}"; Flags: ignoreversion
Source: "..\..\assets\voxforg.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion isreadme

[Icons]
; Start Menu
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\voxforg.ico"; Comment: "Launch VoxForg Neural Speech Studio"
Name: "{group}\{#MyAppName} (Open Studio)"; Filename: "{app}\{#MyAppExeName}"; Parameters: "serve --open"; IconFilename: "{app}\voxforg.ico"
Name: "{group}\Uninstall {#MyAppName}"; Filename: "{uninstallexe}"

; Desktop
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\voxforg.ico"; Comment: "Launch VoxForg Neural Speech Studio"; Tasks: desktopicon

[Run]
; Open the studio in browser right after install
Filename: "{app}\{#MyAppExeName}"; Description: "Launch {#MyAppName} Studio now"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
Type: filesandordirs; Name: "{userappdata}\VoxForg\logs"
