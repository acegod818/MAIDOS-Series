Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$projRoot = Split-Path -Parent $PSScriptRoot

$msbuild = 'C:\Program Files\Microsoft Visual Studio\18\Insiders\MSBuild\Current\Bin\amd64\MSBuild.exe'
$candle = 'C:\WiX\candle.exe'
$light = 'C:\WiX\light.exe'

if (!(Test-Path $msbuild)) { throw "MSBuild not found: $msbuild" }
if (!(Test-Path $candle)) { throw "WiX candle not found: $candle" }
if (!(Test-Path $light)) { throw "WiX light not found: $light" }

$wxs = Join-Path $projRoot 'installer\wix\MAIDOS-IME.wxs'
if (!(Test-Path $wxs)) { throw "WiX source not found: $wxs" }

# Ensure build artifacts exist (Release|x64).
Push-Location $projRoot
try {
  & $msbuild '.\MAIDOS.IME.sln' /m /p:Configuration=Release /p:Platform=x64 /nologo | Out-Host
} finally {
  Pop-Location
}

$coreDll = Join-Path $projRoot 'x64\Release\MAIDOS.IME.Core.dll'
if (!(Test-Path $coreDll)) { throw "Core DLL not found. Build first: $coreDll" }

$dictsDir = Join-Path $projRoot 'src\dicts'
if (!(Test-Path $dictsDir)) { throw "Dicts dir not found: $dictsDir" }

$outDir = Join-Path $projRoot 'dist_installer'
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

$wixObj = Join-Path $outDir 'MAIDOS-IME.wixobj'
$msi = Join-Path $outDir 'MAIDOS-IME.msi'

Push-Location (Split-Path -Parent $wxs)
try {
  & $candle -nologo -dCoreDll="$coreDll" -dDictsDir="$dictsDir" -out "$wixObj" "$wxs" | Out-Host
  if ($LASTEXITCODE -ne 0) { throw "candle.exe failed (exit=$LASTEXITCODE)" }

  & $light -nologo -out "$msi" "$wixObj" | Out-Host
  if ($LASTEXITCODE -ne 0) { throw "light.exe failed (exit=$LASTEXITCODE)" }
} finally {
  Pop-Location
}

Write-Host "MSI built: $msi"
