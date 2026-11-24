<#
.SYNOPSIS  Start the bridges: right controller = Wii Remote 1 (port 26760), left controller = Wii Remote 2 (port 26761).
.DESCRIPTION
  Builds the Rust bridge once (cargo build --release -p bridge), then runs target\release\touch_dsu.exe for each hand.
  Records the PIDs it starts in .bridge.pids so stop.ps1 only ever stops its own processes.
  SteamVR must already be running with the headset connected.
  -Telemetry streams each bridge's movement, pointer and sensors to telemetry-<hand>.jsonl (read it with cargo run -p tools --bin telemetry_tail).
#>
param([switch]$Telemetry)
$root = Split-Path $PSScriptRoot -Parent
$pids = Join-Path $root '.bridge.pids'
if (Test-Path $pids) { "bridges already started (see $pids); run scripts\stop.ps1 first"; return }

cargo build --release -p bridge --manifest-path (Join-Path $root 'Cargo.toml')
if ($LASTEXITCODE -ne 0) { "build failed; bridges not started"; return }
$exe = Join-Path $root 'target\release\touch_dsu.exe'

$env:BRIDGE_HAND = 'right'  # set explicitly: without BRIDGE_HAND the exe is the two-bridge launcher
if ($Telemetry) { $env:BRIDGE_TELEMETRY = Join-Path $root 'telemetry-right.jsonl' }
$right = Start-Process $exe -WorkingDirectory $root -WindowStyle Minimized -PassThru -RedirectStandardOutput (Join-Path $root 'right.log') -RedirectStandardError (Join-Path $root 'right.err.log')
$env:BRIDGE_PORT = '26761'; $env:BRIDGE_HAND = 'left'
if ($Telemetry) { $env:BRIDGE_TELEMETRY = Join-Path $root 'telemetry-left.jsonl' }
$left = Start-Process $exe -WorkingDirectory $root -WindowStyle Minimized -PassThru -RedirectStandardOutput (Join-Path $root 'left.log') -RedirectStandardError (Join-Path $root 'left.err.log')
Remove-Item Env:BRIDGE_PORT, Env:BRIDGE_HAND, Env:BRIDGE_TELEMETRY -ErrorAction SilentlyContinue
"$($right.Id)`n$($left.Id)" | Set-Content $pids
"started: right controller (pid $($right.Id)), left controller (pid $($left.Id)); logs: right.log, left.log"
