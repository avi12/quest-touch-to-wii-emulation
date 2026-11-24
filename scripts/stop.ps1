<#
.SYNOPSIS  Stop the bridges (touch_dsu.exe) that start.ps1 started - by the PIDs it recorded, never by process name.
#>
$pids = Join-Path (Split-Path $PSScriptRoot -Parent) '.bridge.pids'
if (-not (Test-Path $pids)) { "nothing to stop"; return }
foreach ($id in Get-Content $pids) { Stop-Process -Id ([int]$id) -ErrorAction SilentlyContinue }
Remove-Item $pids
"stopped"
