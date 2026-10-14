# Test-only: the Windows path exactly as a new owner does it, starting from
# Google Drive. Builds the three SMALL test downloads (make_small_test.py), puts
# them in a Google Drive folder with rclone, downloads them again the way a
# browser does, checks each fingerprint (SHA-256), unzips the main one, and
# (with -Install) installs through the Windows menu.
#
#   powershell -File tools\release\tests\drive-cycle.ps1 -Remote "myremote:Garden-TEST" -Commit <public commit> [-Install]
#
# -Commit is the full public GitHub commit whose code is the same as this package
# (the main zip only holds what is not on GitHub; the first start fetches that commit).
#
# -Remote is YOUR rclone remote and a test folder in it (https://rclone.org/drive/).
# Nothing about the remote is stored here. Each file gets an "anyone with the
# link" link so it can be downloaded like a real owner would; delete the test
# folder afterwards. The links are written to test-links.txt (for the Ubuntu
# test, test-linux-from-drive.sh) in the work folder, not in the package.
param(
    [Parameter(Mandatory = $true)][string]$Remote,
    [Parameter(Mandatory = $true)][string]$Commit,
    [string]$Work = (Join-Path $env:TEMP "garden-drive-test"),
    [string]$Rclone = "rclone.exe",
    [string]$Email = "you@example.com",
    [switch]$Install
)
$pkg = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))   # the package folder
$files = @("junsgarden-for-jun-TEST-small.zip", "junsgarden-photos-addon-TEST-small.zip", "junsgarden-wiki-addon-TEST-small.zip")
New-Item -ItemType Directory -Force $Work | Out-Null
# 1. The small test downloads (all plants, about 40 photos per folder, 40 wiki pages).
& python "$pkg\tools\release\make_small_test.py" "$Work\small-test"
$env:SMALL_DIR = "$Work\small-test"
$env:GARDEN_CODE_COMMIT = $Commit
& python "$pkg\tools\release\make_zip.py" "$Work\junsgarden-for-jun-TEST-small.zip"
# 2. Up to Drive, a link for each, and down again like a browser; the fingerprints must match.
$dl = "$Work\downloads"; if (Test-Path $dl) { Remove-Item $dl -Recurse -Force }; New-Item -ItemType Directory $dl | Out-Null
$ProgressPreference = 'SilentlyContinue'
$links = @()
foreach ($f in $files) {
    $want = (Get-FileHash "$Work\$f" -Algorithm SHA256).Hash.ToLower()
    & $Rclone copyto "$Work\$f" "$Remote/$f" --drive-chunk-size 64M
    $link = (& $Rclone link "$Remote/$f" | Select-Object -Last 1)
    $id = ($link -replace '.*id=', '').Trim()
    $links += "$f $id $want"
    Invoke-WebRequest -Uri "https://drive.usercontent.google.com/download?id=$id&export=download&confirm=t" -OutFile "$dl\$f" -UseBasicParsing
    $got = (Get-FileHash "$dl\$f" -Algorithm SHA256).Hash.ToLower()
    "{0,-42} from Drive, same as uploaded: {1}" -f $f, ($got -eq $want)
    if ($got -ne $want) { exit 1 }
}
Set-Content "$Work\test-links.txt" $links
# 3. Unzip, with the add-ons next to it under their real names (what the menu looks for).
Expand-Archive "$dl\junsgarden-for-jun-TEST-small.zip" -DestinationPath $dl
# The first start, as Start-Here-Windows.cmd does it: the program from GitHub, every file checked.
& powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$dl\junsgarden-for-jun\tools\get-code.ps1"
if ($LASTEXITCODE -ne 0) { "get-code failed"; exit 1 }
Copy-Item "$dl\junsgarden-photos-addon-TEST-small.zip" "$dl\junsgarden-for-jun\junsgarden-photos-addon.zip"
Copy-Item "$dl\junsgarden-wiki-addon-TEST-small.zip" "$dl\junsgarden-for-jun\junsgarden-wiki-addon.zip"
# 4. Optional: install from the downloaded folder, into Windows' own Ubuntu.
if ($Install) {
    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$dl\junsgarden-for-jun\windows\Garden-Menu.ps1" -Action install -Mode wsl `
        -Name Jun -Email $Email -WebName "testgarden.zapto.org" -Yes *> "$Work\win-install.log"
    "install exit: $LASTEXITCODE   (log: $Work\win-install.log)"
}
