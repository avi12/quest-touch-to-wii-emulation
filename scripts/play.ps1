<#
.SYNOPSIS  One step to play: SteamVR, both bridges, then Dolphin's Wii Menu with the game disc already inserted.
.DESCRIPTION
  Starts SteamVR if it is not running, starts the bridges (start.ps1) if they are not running, and boots Dolphin into
  the Wii Menu with -Disc in the drive, so the Disc Channel shows the game. The disc is passed on Dolphin's command
  line (-C Dolphin.Core.DefaultISO=...), so Dolphin's own config is not changed.
  Paths come from the parameters, else from .play.json in the repo root (gitignored, it holds local paths):
    { "dolphin": "C:\...\Dolphin.exe", "disc": "C:\...\Wii Sports.rvz" }
.EXAMPLE  scripts\play.ps1
.EXAMPLE  scripts\play.ps1 -Disc 'D:\Wii\Wii Sports Resort.rvz' -Telemetry
#>
param([string]$Dolphin, [string]$Disc, [switch]$Telemetry)
$root = Split-Path $PSScriptRoot -Parent
$wiiMenuTitle = '0000000100000002'
$steamVrAppId = '250820'
$steamVrWaitSeconds = 60

$local = Join-Path $root '.play.json'
if (Test-Path $local) {
  $saved = Get-Content $local -Raw | ConvertFrom-Json
  if (-not $Dolphin) { $Dolphin = $saved.dolphin }
  if (-not $Disc) { $Disc = $saved.disc }
}
if (-not $Dolphin -or -not (Test-Path $Dolphin)) { "Dolphin.exe not found: pass -Dolphin or set ""dolphin"" in .play.json"; return }
if (-not $Disc -or -not (Test-Path $Disc)) { "game disc not found: pass -Disc or set ""disc"" in .play.json"; return }

if (-not (Get-Process vrserver -ErrorAction SilentlyContinue)) {
  "starting SteamVR"
  Start-Process "steam://rungameid/$steamVrAppId"
  $deadline = (Get-Date).AddSeconds($steamVrWaitSeconds)
  while (-not (Get-Process vrserver -ErrorAction SilentlyContinue)) {
    if ((Get-Date) -gt $deadline) { "SteamVR did not start; connect the headset (Air Link) and try again"; return }
    Start-Sleep -Seconds 1
  }
}

if (-not (Test-Path (Join-Path $root '.bridge.pids'))) { & (Join-Path $PSScriptRoot 'start.ps1') -Telemetry:$Telemetry }

Start-Process $Dolphin -ArgumentList @('-C', "`"Dolphin.Core.DefaultISO=$Disc`"", '-n', $wiiMenuTitle)
"Dolphin started: Wii Menu with $(Split-Path $Disc -Leaf) in the drive (Disc Channel)"
