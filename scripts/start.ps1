<#
.SYNOPSIS  Start the bridges: right controller = Wii Remote 1 (port 26760), left controller = Wii Remote 2 (port 26761).
.DESCRIPTION
  Records the PIDs it starts in .bridge.pids so stop.ps1 only ever stops its own processes.
  SteamVR must already be running with the headset connected.
  -Telemetry streams each bridge's movement, pointer and sensors to telemetry-<hand>.jsonl (read it with tools/telemetry_tail.ts).
#>
param([switch]$Telemetry)
$root = Split-Path $PSScriptRoot -Parent
$pids = Join-Path $root '.bridge.pids'
if (Test-Path $pids) { "bridges already started (see $pids); run scripts\stop.ps1 first"; return }

if ($Telemetry) { $env:BRIDGE_TELEMETRY = Join-Path $root 'telemetry-right.jsonl' }
$right = Start-Process node -ArgumentList 'touch_dsu.ts' -WorkingDirectory $root -WindowStyle Minimized -PassThru -RedirectStandardOutput (Join-Path $root 'right.log') -RedirectStandardError (Join-Path $root 'right.err.log')
$env:BRIDGE_PORT = '26761'; $env:BRIDGE_HAND = 'left'
if ($Telemetry) { $env:BRIDGE_TELEMETRY = Join-Path $root 'telemetry-left.jsonl' }
$left = Start-Process node -ArgumentList 'touch_dsu.ts' -WorkingDirectory $root -WindowStyle Minimized -PassThru -RedirectStandardOutput (Join-Path $root 'left.log') -RedirectStandardError (Join-Path $root 'left.err.log')
Remove-Item Env:BRIDGE_PORT, Env:BRIDGE_HAND, Env:BRIDGE_TELEMETRY -ErrorAction SilentlyContinue
"$($right.Id)`n$($left.Id)" | Set-Content $pids
"started: right controller (pid $($right.Id)), left controller (pid $($left.Id)); logs: right.log, left.log"
