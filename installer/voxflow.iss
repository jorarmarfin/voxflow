; Instalador de VoxFlow para Windows (Inno Setup 6).
; Compilar: iscc /DAppVersion=0.3.1 installer\voxflow.iss
; Requiere haber compilado antes: pnpm tauri build --no-bundle
#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif

[Setup]
AppId={{6F0B7C1E-3D52-4A87-9B0E-5C2B7A41D9E8}
AppName=VoxFlow
AppVersion={#AppVersion}
AppVerName=VoxFlow {#AppVersion}
AppPublisher=VoxFlow
AppPublisherURL=https://github.com/jorarmarfin/voxflow
DefaultDirName={autopf}\VoxFlow
DefaultGroupName=VoxFlow
DisableProgramGroupPage=yes
; Program Files exige administrador.
PrivilegesRequired=admin
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir=..\src-tauri\target\release\bundle\inno
OutputBaseFilename=VoxFlow_{#AppVersion}_x64-setup
SetupIconFile=..\src-tauri\icons\icon.ico
UninstallDisplayIcon={app}\voxflow.exe
LicenseFile=..\LICENSE
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes

[Languages]
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\src-tauri\target\release\voxflow.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\THIRD_PARTY_LICENSES.md"; DestDir: "{app}"; Flags: ignoreversion

[Dirs]
; Carpeta de registros (diagnóstico). Los usuarios normales pueden escribir aquí.
Name: "{app}\logs"; Permissions: users-modify

[Icons]
Name: "{autoprograms}\VoxFlow"; Filename: "{app}\voxflow.exe"
Name: "{autodesktop}\VoxFlow"; Filename: "{app}\voxflow.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\voxflow.exe"; Description: "{cm:LaunchProgram,VoxFlow}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
Type: filesandordirs; Name: "{app}\logs"

[Code]
// WebView2 (necesario para la interfaz). Windows 11 y Windows 10 actualizado ya lo traen;
// si falta, se descarga el instalador oficial de Microsoft (mejor esfuerzo, sin bloquear).
function WebView2Installed(): Boolean;
var
  V: String;
begin
  Result :=
    RegQueryStringValue(HKLM, 'SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}', 'pv', V) or
    RegQueryStringValue(HKCU, 'Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}', 'pv', V);
  if Result then
    Result := (V <> '') and (V <> '0.0.0.0');
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  ResultCode: Integer;
  Installer: String;
begin
  Result := '';
  if WebView2Installed() then
    Exit;
  try
    DownloadTemporaryFile('https://go.microsoft.com/fwlink/p/?LinkId=2124703', 'MicrosoftEdgeWebview2Setup.exe', '', nil);
    Installer := ExpandConstant('{tmp}\MicrosoftEdgeWebview2Setup.exe');
    Exec(Installer, '/silent /install', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  except
    Log('No se pudo instalar WebView2 automáticamente: ' + GetExceptionMessage);
  end;
end;
