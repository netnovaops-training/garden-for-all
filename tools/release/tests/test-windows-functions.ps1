# Test-only: the new parts of the Windows menu, each on its own, against a
# throwaway copy of the garden folder (in TEMP) and a test Ubuntu in WSL:
# the WSL path, Docker Desktop's checks, line endings, the copy that follows a
# rename, and finding the garden computer again when its address changes.
#
#   powershell -File tools\release\tests\test-windows-functions.ps1 -Distro junstest
#
# The last test needs a computer at home with SSH on port 22. It only knocks on
# the door (nothing logs in) and changes nothing there.
param([string]$Distro = "junstest")
$pkg = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
$ast = [System.Management.Automation.Language.Parser]::ParseFile("$pkg\windows\Garden-Menu.ps1", [ref]$null, [ref]$null)
$want = "Say", "Note", "Warn", "Fail", "Wsl-Root", "Fix-Line-Endings", "Docker-Desktop-Dir", "Docker-Desktop-Up", "Docker-Desktop-In-Wsl",
        "Sync-Renamed-Here", "Test-Port22", "Get-Mac", "Remember-Garden-Computer", "Update-Target-Address"
foreach ($f in $ast.FindAll({ $args[0] -is [System.Management.Automation.Language.FunctionDefinitionAst] }, $true)) {
    if ($want -contains $f.Name) { Invoke-Expression $f.Extent.Text }
}
. "$pkg\windows\Garden-Network.ps1"
$DockerDesktopCli = Join-Path (Docker-Desktop-Dir) "resources\bin\docker.exe"
$ScpExe = "$env:SystemRoot\System32\OpenSSH\scp.exe"; $SshExe = "$env:SystemRoot\System32\OpenSSH\ssh.exe"
$RemoteDir = "junsgarden-for-jun"
$T = @{ mode = "wsl"; distro = $Distro; engine = "engine"; mac = ""; host = "" }
$Root = Join-Path $env:TEMP "garden-pc-copy-test"
Remove-Item $Root -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force "$Root\docker", "$Root\logs" | Out-Null

"== 1. Windows' path into the Ubuntu: " + (Wsl-Root $Distro) + "   it exists: " + (Test-Path ((Wsl-Root $Distro) + "\home"))
"== 2. Docker Desktop running on this PC: " + (Docker-Desktop-Up)
"== 3. Docker Desktop shared with $Distro (its own Docker: expect False): " + (Docker-Desktop-In-Wsl)

"== 4. Windows line endings are changed back, the .cmd files are left alone"
[IO.File]::WriteAllText("$Root\docker\test.sh", "echo one`r`necho two`r`n")
[IO.File]::WriteAllText("$Root\Start-Here.cmd", "@echo off`r`n")
Fix-Line-Endings
"   test.sh still has Windows endings: " + ([IO.File]::ReadAllText("$Root\docker\test.sh").Contains("`r")) + "   the .cmd kept them: " + ([IO.File]::ReadAllText("$Root\Start-Here.cmd").Contains("`r"))

"== 5. This PC's copy follows the last rename in $Distro"
$lines = & wsl.exe -d $Distro --cd ~ -- cat junsgarden-for-jun/logs/last-rename.txt
foreach ($l in $lines) { $p = "$l".Split("`t"); if ($p[0] -eq "edit") { New-Item -ItemType Directory -Force (Split-Path (Join-Path $Root $p[1])) | Out-Null; Set-Content (Join-Path $Root $p[1]) "old text" } }
Sync-Renamed-Here
foreach ($l in $lines) { $p = "$l".Split("`t"); if ($p[0] -eq "edit") { "   " + $p[1] + " updated: " + ((Get-Content (Join-Path $Root $p[1]) -Raw).Length -gt 20) } }
Remove-Item $Root -Recurse -Force

"== 6. Found again after its address changed (a computer at home with SSH on 22)"
function Save-Target { }
$real = @(Find-Home-Computers)[0].IP
if (-not $real) { "   no computer with SSH at home; skipped"; return }
$T = @{ mode = "remote"; ip = $real; user = "x"; mac = ""; host = "" }
Remember-Garden-Computer
$T.ip = (($real -split '\.')[0..2] -join '.') + ".253"     # pretend the router gave it another address
"   found again: " + (Update-Target-Address -Quiet) + "   the right one: " + ($T.ip -eq $real)
$T.ip = (($real -split '\.')[0..2] -join '.') + ".253"; $T.mac = "00-11-22-33-44-55"
"   an unknown network card is not 'found': " + (-not (Update-Target-Address -Quiet))
