<#
.SYNOPSIS  Keep a Quest awake (and "worn") over adb while it is only being used for tracking.
.DESCRIPTION
  The headset sleeps when its proximity sensor says nobody is wearing it, which drops the controllers.
  This loop re-sends the 'worn' signal every 20 s. Ctrl+C to stop, then run with -Restore to give the sensor back.
  Needs USB debugging (developer mode) and `adb` on PATH. Pass -Serial if more than one device is attached.
#>
param([string]$Serial, [switch]$Restore)
$adb = @(); if ($Serial) { $adb = @('-s', $Serial) }
if ($Restore) { & adb @adb shell am broadcast -a com.oculus.vrpowermanager.automation_disable | Out-Null; "proximity sensor restored"; return }
# stop the screen timing out while plugged in
& adb @adb shell settings put system screen_off_timeout 2147483647 | Out-Null
"keeping the headset awake - Ctrl+C to stop, then run with -Restore"
while ($true) {
    & adb @adb shell am broadcast -a com.oculus.vrpowermanager.prox_close | Out-Null
    & adb @adb shell input keyevent KEYCODE_WAKEUP | Out-Null
    Start-Sleep 20
}
