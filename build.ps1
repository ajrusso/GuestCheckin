# Build a portable GuestCheckin package and create/update a Desktop shortcut.
param(
    [string]$InstallDir = (Join-Path $PSScriptRoot "dist\GuestCheckin")
)

$ErrorActionPreference = "Stop"

Write-Host "Building release binary..."
cargo build --release
if ($LASTEXITCODE -ne 0) {
    throw "cargo build --release failed with exit code $LASTEXITCODE"
}

$exeSource = Join-Path $PSScriptRoot "target\release\guest-checkin.exe"
if (-not (Test-Path $exeSource)) {
    throw "Release binary not found at $exeSource"
}

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

Write-Host "Copying executable to $InstallDir"
Copy-Item $exeSource (Join-Path $InstallDir "guest-checkin.exe") -Force

$configDest = Join-Path $InstallDir "config.toml"
$configSource = Join-Path $PSScriptRoot "src\config\config.toml"
$configExample = Join-Path $PSScriptRoot "src\config\config.toml.example"

if (Test-Path $configSource) {
    Copy-Item $configSource $configDest -Force
    Write-Host "Copied config.toml"
} elseif (-not (Test-Path $configDest)) {
    if (Test-Path $configExample) {
        Copy-Item $configExample $configDest -Force
        Write-Warning "No src\config\config.toml found. Copied example to $configDest - fill it in before running."
    } else {
        Write-Warning "No config.toml found. Create one at $configDest before running."
    }
} else {
    Write-Host "Keeping existing install config.toml"
}

$keySource = Join-Path $PSScriptRoot "service_account_key.json"
$keyDest = Join-Path $InstallDir "service_account_key.json"
if (Test-Path $keySource) {
    Copy-Item $keySource $keyDest -Force
    Write-Host "Copied service_account_key.json"
} elseif (-not (Test-Path $keyDest)) {
    Write-Warning "service_account_key.json not found. Place it at $keyDest before running."
}

$headerSource = Join-Path $PSScriptRoot "src\header_image.jpg"
$headerDest = Join-Path $InstallDir "header_image.jpg"
if (Test-Path $headerSource) {
    Copy-Item $headerSource $headerDest -Force
    Write-Host "Copied header_image.jpg"
} elseif (-not (Test-Path $headerDest)) {
    Write-Warning "src\header_image.jpg not found. Emails will send without the header image."
}

# Launcher keeps the console open after the exe exits or crashes (cmd `pause`).
$launcherPath = Join-Path $InstallDir "run-guest-checkin.cmd"
@"
@echo off
cd /d "%~dp0"
guest-checkin.exe
set EXITCODE=%ERRORLEVEL%
echo.
if not %EXITCODE%==0 echo Exit code: %EXITCODE%
pause
exit /b %EXITCODE%
"@ | Set-Content -Path $launcherPath -Encoding ASCII
Write-Host "Wrote $launcherPath"

$desktop = [Environment]::GetFolderPath("Desktop")
$shortcutPath = Join-Path $desktop "GuestCheckin.lnk"
$exePath = Join-Path $InstallDir "guest-checkin.exe"

$wsh = New-Object -ComObject WScript.Shell
$shortcut = $wsh.CreateShortcut($shortcutPath)
$shortcut.TargetPath = $launcherPath
$shortcut.WorkingDirectory = $InstallDir
# Prefer the source .ico so Explorer shows the custom art reliably (Windows icon cache
# often sticks to a stale default when IconLocation points at a replaced .exe).
$iconPath = Join-Path $PSScriptRoot "assets\icon.ico"
if (Test-Path $iconPath) {
    $shortcut.IconLocation = "$iconPath,0"
} else {
    $shortcut.IconLocation = "$exePath,0"
}
$shortcut.Description = "Guest Checkin"
$shortcut.Save()


Write-Host ""
Write-Host "Done."
Write-Host "  Install folder: $InstallDir"
Write-Host "  Desktop shortcut: $shortcutPath"
Write-Host "Double-click GuestCheckin on the Desktop to run."
