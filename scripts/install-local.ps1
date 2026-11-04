# Builds the Windows installer from this checkout and installs it over the
# current install. scripts/hooks/post-merge runs it after a pull brings new
# commits into main; `npm run install-local` runs it by hand.
#
# Every step must succeed before the installer runs, so a broken build never
# replaces the knight you already have.

$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot)

# Native commands don't throw when they fail, so check each exit code.
function Invoke-Native([string]$what, [scriptblock]$command) {
    & $command
    if ($LASTEXITCODE -ne 0) { throw "$what failed (exit code $LASTEXITCODE)." }
}

$started = Get-Date
Invoke-Native 'npm ci' { npm ci }
Invoke-Native 'The build' { npm run tauri build -- --bundles nsis }

# Installers from earlier builds stay in this folder, so take only one this build
# wrote. Fail rather than fall back to an older one: with CARGO_TARGET_DIR set,
# the build writes elsewhere, and a stale installer here would downgrade him.
$bundles = 'src-tauri\target\release\bundle\nsis'
$setup = Get-ChildItem "$bundles\*-setup.exe" |
    Where-Object { $_.LastWriteTime -ge $started } |
    Sort-Object LastWriteTime -Descending | Select-Object -First 1
if (-not $setup) { throw "The build wrote no installer to $bundles." }

# The per-user install location; tauri.conf.json doesn't change it.
$exe = Join-Path $env:LOCALAPPDATA 'Sentinel\sentinel.exe'
$flags = @('/S')
# /NS: a silent install otherwise adds a desktop shortcut on every run. Only on
# a reinstall, so a first install still gets its Start menu shortcut.
if (Test-Path $exe) { $flags += '/NS' }
# The silent installer closes a running knight itself, and /R starts him again.
# Pass /R only if he was running: a pull shouldn't summon him. Match on the
# path, because the dev build is also sentinel.exe.
$running = Get-Process sentinel -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $exe }
if ($running) { $flags += '/R' }

Write-Host "Installing $($setup.Name) $flags"
$installer = Start-Process $setup.FullName -ArgumentList $flags -PassThru
# Without reading Handle first, ExitCode is empty once the process has exited.
$null = $installer.Handle
# Wait for the installer only. Start-Process -Wait would also wait for the
# knight it relaunches.
$installer.WaitForExit()
if ($installer.ExitCode -ne 0) { throw "The installer failed (exit code $($installer.ExitCode))." }
Write-Host 'Sentinel is installed.'
