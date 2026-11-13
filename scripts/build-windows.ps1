# Builds Marsa for Windows: the modern installer (apps/installer, single
# self-contained exe with Marsa embedded) and a portable ZIP, then writes
# SHA-256 sums for both. Windows PowerShell 5.1 or PowerShell 7.
#
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build-windows.ps1
#
# Environment:
#   MARSA_SKIP_INSTALL=1          use node_modules as they are (no npm install)
#   MARSA_NO_PORTABLE=1           build the installer only
#   MARSA_SIGNING_KEY=<path>      sign the installer and write latest.json (ADR-0011)
#   MARSA_SIGNING_KEY_PASSWORD=…  the key's password, if it has one
#   MARSA_DOWNLOADS=<url>         where the release's files will be, with a
#                                 trailing slash; default is this repository's
#                                 releases/download/
#
# Without MARSA_SIGNING_KEY the artefacts are unsigned and there is no
# latest.json: nothing could update to them. That is what a local build does.
#
# The version comes from apps/desktop/src-tauri/tauri.conf.json, never from
# this script, so the artefacts and the app always agree.

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$root = Split-Path -Parent $PSScriptRoot
$desktop = Join-Path $root 'apps\desktop'
$tauri = Join-Path $desktop 'src-tauri'
$installer = Join-Path $root 'apps\installer'
$installerTauri = Join-Path $installer 'src-tauri'
$out = Join-Path $root 'dist\windows'

function Require-Command($name, $hint) {
    if (-not (Get-Command $name -ErrorAction SilentlyContinue)) {
        throw "Missing '$name'. $hint"
    }
}

Require-Command 'cargo' 'Install Rust from https://rustup.rs'
Require-Command 'npm' 'Install Node.js 22 or newer from https://nodejs.org'
Require-Command 'rustc' 'Install Rust from https://rustup.rs'

Write-Host '==> Marsa icons'
& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'make-icons.ps1')
if ($LASTEXITCODE -ne 0) { throw 'make-icons.ps1 failed' }

if ($env:MARSA_SKIP_INSTALL -ne '1') {
    Write-Host '==> npm install (desktop + installer)'
    Push-Location $desktop
    try {
        if (Test-Path -LiteralPath 'package-lock.json') { & npm ci } else { & npm install }
        if ($LASTEXITCODE -ne 0) { throw 'npm install (desktop) failed' }
    } finally { Pop-Location }
    Push-Location $installer
    try {
        if (Test-Path -LiteralPath 'package-lock.json') { & npm ci } else { & npm install }
        if ($LASTEXITCODE -ne 0) { throw 'npm install (installer) failed' }
    } finally { Pop-Location }
}

Write-Host '==> tauri build (main app, release, no-bundle)'
Push-Location $desktop
try {
    & npm.cmd run tauri build -- --no-bundle
    if ($LASTEXITCODE -ne 0) { throw 'tauri build (main) failed' }
} finally { Pop-Location }

$mainExe = Join-Path $tauri 'target\release\marsa-desktop.exe'
if (-not (Test-Path -LiteralPath $mainExe)) { throw "No main exe at $mainExe" }
Write-Host "    main exe: $([math]::Round((Get-Item -LiteralPath $mainExe).Length/1MB,1)) MB"

$hostExe = Join-Path $tauri 'target\release\marsa-host.exe'
if (-not (Test-Path -LiteralPath $hostExe)) { throw "No host exe at $hostExe" }
Write-Host "    host exe: $([math]::Round((Get-Item -LiteralPath $hostExe).Length/1MB,1)) MB"

Write-Host '==> staging main exe into installer'
$staged = Join-Path $installerTauri 'resources\marsa-desktop.exe'
New-Item -ItemType Directory -Force -Path (Split-Path -Parent $staged) | Out-Null
Copy-Item -LiteralPath $mainExe -Destination $staged -Force

Write-Host '==> staging host exe into installer'
$stagedHost = Join-Path $installerTauri 'resources\marsa-host.exe'
New-Item -ItemType Directory -Force -Path (Split-Path -Parent $stagedHost) | Out-Null
Copy-Item -LiteralPath $hostExe -Destination $stagedHost -Force

Write-Host '==> tauri build (installer, release, no-bundle)'
Push-Location $installer
try {
    & npm.cmd run tauri build -- --no-bundle
    if ($LASTEXITCODE -ne 0) { throw 'tauri build (installer) failed' }
} finally { Pop-Location }

$config = Get-Content -LiteralPath (Join-Path $tauri 'tauri.conf.json') -Raw | ConvertFrom-Json
$version = $config.version
if (-not $version) { throw 'tauri.conf.json has no version' }

New-Item -ItemType Directory -Force -Path $out | Out-Null

$setupExe = Join-Path $installerTauri 'target\release\marsa-installer.exe'
if (-not (Test-Path -LiteralPath $setupExe)) { throw "No installer exe at $setupExe" }
$installerOut = Join-Path $out ("Marsa_{0}_x64-setup.exe" -f $version)
Copy-Item -LiteralPath $setupExe -Destination $installerOut -Force
Write-Host "    installer: $installerOut ($([math]::Round((Get-Item -LiteralPath $installerOut).Length/1MB,1)) MB)"

$artefacts = @($installerOut)

# The signed update (ADR-0011): the installer's minisign signature, and the
# manifest Tauri's updater reads. Both are made by the pinned Tauri CLI in
# apps/desktop/node_modules, the same version that builds the app.
if ($env:MARSA_SIGNING_KEY) {
    $key = $env:MARSA_SIGNING_KEY
    if (-not (Test-Path -LiteralPath $key)) { throw "No signing key at $key" }
    $tauriCli = Join-Path $desktop 'node_modules\.bin\tauri.cmd'
    if (-not (Test-Path -LiteralPath $tauriCli)) { throw "No tauri CLI at $tauriCli (run npm install)" }
    Write-Host '==> signing the installer'
    $signArgs = @('signer', 'sign', '-f', $key, '--app-version', $version, $installerOut)
    if ($env:MARSA_SIGNING_KEY_PASSWORD) {
        $signArgs += @('-p', $env:MARSA_SIGNING_KEY_PASSWORD)
    }
    & $tauriCli @signArgs
    if ($LASTEXITCODE -ne 0) { throw 'signing the installer failed' }
    $sig = "$installerOut.sig"
    if (-not (Test-Path -LiteralPath $sig)) { throw "No signature at $sig" }

    $downloads = $env:MARSA_DOWNLOADS
    if (-not $downloads) {
        $downloads = 'https://github.com/dentalproo/marsa-download-manager/releases/download/'
    }
    if (-not $downloads.EndsWith('/')) { $downloads += '/' }
    $asset = Split-Path -Leaf $installerOut
    $manifest = [ordered]@{
        version   = $version
        pub_date  = (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')
        platforms = [ordered]@{
            'windows-x86_64' = [ordered]@{
                url       = "$downloads" + "v$version/$asset"
                signature = (Get-Content -LiteralPath $sig -Raw).Trim()
            }
        }
    }
    $manifestPath = Join-Path $out 'latest.json'
    # No BOM: a JSON parser must read the first byte as `{`.
    $json = $manifest | ConvertTo-Json -Depth 5
    [System.IO.File]::WriteAllText($manifestPath, $json, (New-Object System.Text.UTF8Encoding($false)))
    Write-Host "    signature: $sig"
    Write-Host "    manifest:  $manifestPath"
    $artefacts += $sig
    $artefacts += $manifestPath
} else {
    Write-Host '==> no MARSA_SIGNING_KEY: the artefacts are unsigned, and no latest.json is written'
    Write-Host '    (nothing can update to an unsigned build; ADR-0011)'
}

if ($env:MARSA_NO_PORTABLE -ne '1') {
    Write-Host '==> portable ZIP'
    $portable = Join-Path $out 'Marsa-portable'
    if (Test-Path -LiteralPath $portable) { Remove-Item -LiteralPath $portable -Recurse -Force }
    New-Item -ItemType Directory -Force -Path $portable | Out-Null
    Copy-Item -LiteralPath $mainExe -Destination (Join-Path $portable 'Marsa.exe')
    Copy-Item -LiteralPath $hostExe -Destination (Join-Path $portable 'marsa-host.exe')
    Copy-Item -LiteralPath (Join-Path $root 'LICENSE') -Destination $portable
    $notices = Join-Path $root 'scripts\release\licenses\THIRD-PARTY-NOTICES.md'
    if (Test-Path -LiteralPath $notices) { Copy-Item -LiteralPath $notices -Destination $portable }
    Copy-Item -LiteralPath (Join-Path $root 'README.md') -Destination $portable
    $zip = Join-Path $out ("Marsa_{0}_portable_x64.zip" -f $version)
    if (Test-Path -LiteralPath $zip) { Remove-Item -LiteralPath $zip -Force }
    Compress-Archive -Path (Join-Path $portable '*') -DestinationPath $zip -CompressionLevel Optimal
    Write-Host "    portable:  $zip"
    $artefacts += $zip
}

Write-Host '==> SHA-256'
$lines = foreach ($file in $artefacts) {
    $hash = (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant()
    $name = Split-Path -Leaf $file
    "{0}  {1}" -f $hash, $name
}
$sums = Join-Path $out 'SHA256SUMS'
Set-Content -LiteralPath $sums -Value $lines -Encoding ASCII
Write-Host "    sums:      $sums"
$lines | ForEach-Object { Write-Host "    $_" }

Write-Host ''
Write-Host "Done. Version $version, in $out"
