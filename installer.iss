[Setup]
AppName=SearchForge
AppVersion={#MyAppVersion}
AppPublisher=SearchForge
AppPublisherURL=https://github.com/Saboor-hamedi/search_forge
AppSupportURL=https://github.com/Saboor-hamedi/search_forge/issues
AppUpdatesURL=https://github.com/Saboor-hamedi/search_forge/releases
DefaultDirName={autopf}\SearchForge
DisableDirPage=no
DefaultGroupName=SearchForge
DisableProgramGroupPage=no
DisableReadyPage=no
DisableFinishedPage=no
AlwaysShowDirOnReadyPage=yes
AlwaysShowGroupOnReadyPage=yes
WizardStyle=modern
OutputDir=.
OutputBaseFilename=SearchForge-Setup
Compression=lzma2/ultra64
SolidCompression=yes
PrivilegesRequired=none
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayIcon={app}\search_forge.exe
CloseApplications=force

[Files]
Source: "target\x86_64-pc-windows-msvc\release\search_forge.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "pdfium.dll"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist

[Icons]
Name: "{group}\SearchForge"; Filename: "{app}\search_forge.exe"
Name: "{autodesktop}\SearchForge"; Filename: "{app}\search_forge.exe"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Run]
Filename: "{app}\search_forge.exe"; Description: "{cm:LaunchProgram,SearchForge}"; Flags: nowait postinstall skipifsilent
