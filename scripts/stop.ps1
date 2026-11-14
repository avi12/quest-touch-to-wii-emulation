<#
.SYNOPSIS  Stop the bridges that start.ps1 started - by PID, never by process name.
#>
$pids = Join-Path (Split-Path $PSScriptRoot -Parent) '.bridge.pids'
if (-not (Test-Path $pids)) { "nothing to stop"; return }
foreach ($id in Get-Content $pids) { Stop-Process -Id ([int]$id) -ErrorAction SilentlyContinue }
Remove-Item $pids
"stopped"
