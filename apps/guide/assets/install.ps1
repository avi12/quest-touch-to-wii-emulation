# Installs an app update: waits for the app to exit, copies the new release over its folder and
# starts it again. app-update.ts runs this from a temporary copy, so it never overwrites itself.
param([int]$AppPid, [string]$Zip, [string]$Target, [string]$Exe)

Wait-Process -Id $AppPid -ErrorAction SilentlyContinue
$unpacked = Join-Path (Split-Path $Zip) 'unpacked'
Expand-Archive -Path $Zip -DestinationPath $unpacked -Force
Copy-Item -Path (Join-Path $unpacked '*') -Destination $Target -Recurse -Force
Start-Process -FilePath (Join-Path $Target $Exe) -WorkingDirectory $Target
