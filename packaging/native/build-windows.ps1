param([Parameter(Mandatory=$true)][string]$Version)
$ErrorActionPreference = "Stop"
$ProjectDir = (Resolve-Path "$PSScriptRoot\..\..").Path

function Invoke-NitumCodeSign([string]$Path) {
    if ([string]::IsNullOrWhiteSpace($env:NITUM_WINDOWS_CERTIFICATE_BASE64)) {
        return
    }
    if ([string]::IsNullOrWhiteSpace($env:NITUM_WINDOWS_CERTIFICATE_PASSWORD)) {
        throw "NITUM_WINDOWS_CERTIFICATE_PASSWORD es obligatorio para firmar Windows."
    }
    $Certificate = Join-Path $env:RUNNER_TEMP "nitum-windows-signing.pfx"
    [IO.File]::WriteAllBytes($Certificate, [Convert]::FromBase64String($env:NITUM_WINDOWS_CERTIFICATE_BASE64))
    $SignTool = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\signtool.exe" |
        Sort-Object FullName -Descending |
        Select-Object -First 1
    if ($null -eq $SignTool) {
        throw "No se encontró signtool.exe en el runner de Windows."
    }
    & $SignTool.FullName sign /fd SHA256 /td SHA256 /tr "http://timestamp.digicert.com" /f $Certificate /p $env:NITUM_WINDOWS_CERTIFICATE_PASSWORD $Path
    if ($LASTEXITCODE -ne 0) { throw "No se pudo firmar $Path" }
    & $SignTool.FullName verify /pa /v $Path
    if ($LASTEXITCODE -ne 0) { throw "La firma Authenticode de $Path no es válida" }
}

cargo build --manifest-path "$ProjectDir\native\Cargo.toml" --release --locked
& "C:\Program Files\Git\bin\bash.exe" "$ProjectDir/native/scripts/fetch-pdfium.sh" "$ProjectDir/native/target/release"
if ($LASTEXITCODE -ne 0) { throw "No se pudo descargar y verificar PDFium para Windows." }
Invoke-NitumCodeSign "$ProjectDir\native\target\release\nitum-pdf.exe"
$OutputDir = "$ProjectDir\dist"
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
$Template = Get-Content "$PSScriptRoot\windows-installer.iss.in" -Raw
$Template = $Template.Replace("@VERSION@", $Version).Replace("@OUTPUT_DIR@", $OutputDir).Replace("@BINARY_DIR@", "$ProjectDir\native\target\release").Replace("@ICON_FILE@", "$ProjectDir\packaging\native\nitum-pdf.ico")
$Iss = "$env:RUNNER_TEMP\nitum-pdf.iss"
Set-Content -Path $Iss -Value $Template
& "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" $Iss
if ($LASTEXITCODE -ne 0) { throw "Inno Setup no pudo crear el instalador." }
$Installer = Get-ChildItem "$OutputDir\nitum-pdf-$Version-windows-x86_64.exe" | Select-Object -First 1
Invoke-NitumCodeSign $Installer.FullName
