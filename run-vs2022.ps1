$ErrorActionPreference = 'Stop'

# Use the automatic `$args` array instead of a ValueFromRemainingArguments
# parameter: Windows PowerShell 5.1 drops Cargo's `--` separator when binding
# named-looking arguments to a declared parameter.
$CargoArgs = @($args)

$projectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$vsRoot = 'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools'
$vcvars = Join-Path $vsRoot 'VC\Auxiliary\Build\vcvars64.bat'
$toolsRoot = Join-Path $vsRoot 'VC\Tools\MSVC'

if (-not (Test-Path -LiteralPath $vcvars)) {
    throw "VS 2022 Build Tools x64 environment was not found at $vcvars"
}

$msvc = Get-ChildItem -LiteralPath $toolsRoot -Directory |
    Sort-Object { [version]$_.Name } -Descending |
    Select-Object -First 1
if (-not $msvc) {
    throw "No MSVC toolset found under $toolsRoot"
}

$cl = Join-Path $msvc.FullName 'bin\Hostx64\x64\cl.exe'
if (-not (Test-Path -LiteralPath $cl)) {
    throw "cl.exe was not found at $cl"
}

# lib.exe/link.exe otherwise try to write temporary files under the user's
# default %TEMP%, which can fail in restricted shells. Keep them local.
$tmp = Join-Path $projectRoot '.tmp'
New-Item -ItemType Directory -Force -Path $tmp | Out-Null
$env:CC = $cl
$env:CXX = $cl
$env:TMP = $tmp
$env:TEMP = $tmp

if (-not $CargoArgs -or $CargoArgs.Count -eq 0) {
    $CargoArgs = @('run', '--example', 'echo')
}

$quoted = foreach ($arg in $CargoArgs) {
    if ($arg -match '\s') { '"' + $arg + '"' } else { $arg }
}
$cargoCommand = 'cargo ' + ($quoted -join ' ')

Write-Host "MSVC: $cl"
Write-Host "TMP:  $tmp"
Write-Host "Run:  $cargoCommand"

& cmd.exe /d /s /c "call `"$vcvars`" >nul 2>&1 && $cargoCommand"
exit $LASTEXITCODE
