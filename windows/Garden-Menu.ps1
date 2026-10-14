<#
===============================================================================
 Garden-Menu.ps1: the Jun's Garden menu for Windows.
 Start it by double-clicking  Garden-Menu-Windows.cmd  (one folder up).
 (The guided installer with windows and buttons is Start-Here-Windows.cmd.)

 The garden always runs on UBUNTU, in one of two places:
   A. ANOTHER computer at home (an Ubuntu laptop or a Raspberry Pi), reached
      over SSH (a secure remote terminal). Recommended: it can stay on.
   B. THIS Windows PC, inside Windows' own built-in Ubuntu (called WSL).
 Either way it is the SAME program (garden.sh) doing the SAME steps as on a
 real Ubuntu computer, so this menu matches the Ubuntu menu item for item.
 This menu just runs those jobs on the garden computer and shows them here.

 Passwords: this script never asks for, stores or shows a password itself.
 ssh, sudo or Ubuntu ask for them, and what you type is not shown.

 For testing, a job can be run directly, for example:
   powershell -ExecutionPolicy Bypass -File windows\Garden-Menu.ps1 -Action check
===============================================================================
#>
param(
    [string]$Action = "menu",
    [ValidateSet("", "remote", "wsl")][string]$Mode = "",
    [string]$RemoteHost = "",
    [string]$RemoteUser = "",
    [string]$Distro = "",
    [string]$Name = "",
    [string]$Email = "",
    [string]$Parts = $null,
    [string]$WebName = "",
    [ValidateSet("", "desktop", "engine")][string]$Engine = "",
    [switch]$Yes
)

$PartsGiven = $PSBoundParameters.ContainsKey('Parts')   # was -Parts given? (read here: inside a function it would mean that function's own options)
$ErrorActionPreference = "Continue"
# Windows' OWN tar, ssh and scp, by full path: if Git for Windows (or another
# program) put its own versions first on the PATH, those read "C:\..." as a
# network address and the copy fails.
function Windows-Tool($name, $path) { if (Test-Path $path) { return $path } else { return $name } }
$TarExe = Windows-Tool "tar.exe" "$env:SystemRoot\System32\tar.exe"
$SshExe = Windows-Tool "ssh.exe" "$env:SystemRoot\System32\OpenSSH\ssh.exe"
$ScpExe = Windows-Tool "scp.exe" "$env:SystemRoot\System32\OpenSSH\scp.exe"
$script:HadFail = $false
$Root = Split-Path -Parent $PSScriptRoot          # the garden folder on this PC
$RemoteDir = "junsgarden-for-jun"                 # folder name on the garden computer
$TargetFile = Join-Path $Root ".garden-target-windows.txt"
# One word the guided installer window reads, so it always knows where things
# stand: RUNNING, RESTART (Windows needs its one restart), FAILED or DONE.
$StatusFile = Join-Path $Root "logs\windows-install-status.txt"
function Set-Status($word) {
    New-Item -ItemType Directory -Force (Split-Path $StatusFile) | Out-Null
    Set-Content -Path $StatusFile -Value $word -Encoding ASCII
}
$TunnelPort = 18030                               # this PC's end of the SSH tunnel
$DefaultDistro = "JunsGarden"                     # the WSL Ubuntu made for the garden
Set-Location $Root
. (Join-Path $PSScriptRoot "Garden-Network.ps1")  # this PC's address; find computers at home
try { $Host.UI.RawUI.WindowTitle = "Jun's Garden" } catch { }

# ---------- small helpers ------------------------------------------------------
function Say($t)  { Write-Host ""; Write-Host "== $t" -ForegroundColor Green }
# An indented line of plain text.
function Note($t) { Write-Host "   $t" }
# A yellow WARNING line: something to fix soon, not a stop.
function Warn($t) { Write-Host "WARNING: $t" -ForegroundColor Yellow }
# A red STOPPED line.
function Fail($t) { Write-Host "STOPPED: $t" -ForegroundColor Red; $script:HadFail = $true }
# Ask a question; Enter gives the suggested answer shown in [brackets].
function Ask($q, $def) {
    if ($def) { $a = Read-Host "$q [$def]" } else { $a = Read-Host $q }
    if ([string]::IsNullOrWhiteSpace($a)) { return $def } else { return $a.Trim() }
}
# Ask a yes/no question. Capital letter = what Enter means.
function Confirm-Step($q, [bool]$def = $true) {
    if ($Yes) { return $def }
    $hint = "y/N"; if ($def) { $hint = "Y/n" }
    $a = Read-Host "$q ($hint)"
    if ([string]::IsNullOrWhiteSpace($a)) { return $def }
    return ($a -match '^(y|yes)$')
}
# Run a program in THIS window and wait. Its questions (passwords, y/n) show here.
function Run-Console($exe, $argString) {
    Write-Host "   > $exe $argString" -ForegroundColor Cyan
    $p = Start-Process -FilePath $exe -ArgumentList $argString -NoNewWindow -Wait -PassThru -WorkingDirectory $Root
    return $p.ExitCode
}
# Wait for Enter before showing the menu again, so you can read the screen.
function Pause-Menu { if (-not $Yes -and $Action -eq "menu") { Read-Host "`nPress Enter to go back to the menu" | Out-Null } }

# ---------- the garden computer (target) ---------------------------------------------
$T = @{ mode = ""; ip = ""; user = ""; distro = $DefaultDistro; copyto = ""; engine = "engine"; mac = ""; host = "" }
# Read which computer runs the garden (saved in .garden-target-windows.txt, no passwords).
function Load-Target {
    if (Test-Path $TargetFile) {
        foreach ($l in Get-Content $TargetFile) { if ($l -match '^(\w+)=(.*)$') { $T[$Matches[1]] = $Matches[2] } }
    }
    if ($Mode) { $T.mode = $Mode }
    if ($RemoteHost) { $T.ip = $RemoteHost }
    if ($RemoteUser) { $T.user = $RemoteUser }
    if ($Distro) { $T.distro = $Distro }
    if ($Engine) { $T.engine = $Engine }
}
# Remember which computer runs the garden (IP and user name only, never a password).
function Save-Target { Set-Content -Path $TargetFile -Encoding UTF8 -Value @("mode=$($T.mode)", "ip=$($T.ip)", "user=$($T.user)", "distro=$($T.distro)", "copyto=$($T.copyto)", "engine=$($T.engine)", "mac=$($T.mac)", "host=$($T.host)") }
# Describe the garden computer in words, for the top of the menu.
function Target-Text {
    switch ($T.mode) {
        "remote" { return "another computer: $($T.user)@$($T.ip) (over SSH)" }
        "wsl"    { if ($T.engine -eq "desktop") { return "this PC, inside Windows' Ubuntu ($($T.distro)), with Docker Desktop" } else { return "this PC, inside Windows' Ubuntu ($($T.distro))" } }
        default  { return "not chosen yet" }
    }
}

# Menu T: choose another computer (by IP, over SSH) or this PC's built-in Ubuntu (WSL).
function Choose-Target {
    Say "Where does your garden run?"
    Write-Host @"
   The garden is a small server: it should run on a computer that stays on.

   1) ANOTHER computer at home: an Ubuntu laptop or a Raspberry Pi  (recommended)
      You need: that computer switched on, with Ubuntu (or Raspberry Pi OS) and
      the SSH server (on it, once:  sudo apt install -y openssh-server), and its
      LOCAL IP address (on it, type:  hostname -I ; it looks like 192.168.1.50).
      Later the router forwards ports 80 and 443 to THAT address, and no other.

   2) THIS Windows PC, inside Windows' own Ubuntu (WSL)
      Windows installs Ubuntu as an app; the garden runs in it exactly like on
      a real Ubuntu computer. The PC must stay on and signed in for the site
      to be up. Later the router forwards ports 80 and 443 to THIS PC.

   0) Back
"@
    $me = Get-This-PC
    if ($me) { Note "This PC: $($me.Name), address $($me.IP) (router $($me.Router))." }
    $c = Ask "Type 1, 2 or 0" "1"
    if ($c -eq "1") {
        $T.mode = "remote"
        Note "Type its address, or F to find the computers at home that have the SSH door open."
        $a = Ask "Its local IP address (on it: hostname -I), or F" $T.ip
        if ($a -match '^[fF]$') {
            Note "Looking on your home network for the SSH door (port 22). A few seconds..."
            $list = @(Find-Home-Computers)
            if ($list.Count -eq 0) {
                Note "None found. Is the garden computer on, with the SSH server (sudo apt install -y openssh-server)?"
                $a = Ask "Its local IP address (on it: hostname -I)" $T.ip
            } else {
                for ($i = 0; $i -lt $list.Count; $i++) { Note ("{0}) {1}  {2}" -f ($i + 1), $list[$i].IP, $list[$i].Name) }
                $p = Ask "Which one is the garden computer (number)" "1"
                if ($p -match '^\d+$' -and [int]$p -ge 1 -and [int]$p -le $list.Count) { $a = $list[[int]$p - 1].IP } else { $a = "" }
            }
        }
        $T.ip = $a
        Note "User name = the name you LOG IN with on that computer (on it, type: whoami). Not its host name."
        $T.user = Ask "Your user name on it" $T.user
        if (-not $T.ip -or -not $T.user) { Fail "need both the address and the user name"; return $false }
    } elseif ($c -eq "2") {
        $T.mode = "wsl"
        if ($me) { Note "The garden goes on THIS PC: $($me.Name) ($($me.IP))." }
        Choose-Engine
    } else { return $false }
    $T.mac = ""; $T.host = ""
    Save-Target
    if ($T.mode -eq "remote") { Remember-Garden-Computer }   # its network card, if it answers now
    Note "Garden computer: $(Target-Text)"
    return $true
}
# On this PC: which Docker runs the garden's programs? Asked only when "this
# Windows PC" was chosen. (On another computer it is always Docker inside Ubuntu.)
function Choose-Engine {
    Write-Host @"

   Which Docker should run the garden's programs on this PC?

   1) Docker Desktop: the easy way to manage it  (recommended on Windows)
      A Windows app with buttons: see the garden's programs, start and stop
      them, read their messages. It starts with Windows. Free for personal use.

   2) Docker inside Ubuntu: the hands-on way
      The same as on any Ubuntu computer: everything from the menu and the
      terminal. Nothing more to install on Windows.

"@
    $e = Ask "Type 1 or 2" "1"
    if ($e -eq "2") { $T.engine = "engine" } else { $T.engine = "desktop" }
    Note "Docker: $(if ($T.engine -eq 'desktop') { 'Docker Desktop (the easy way)' } else { 'inside Ubuntu (the hands-on way)' }). Choose again: menu T."
}

# The easy way: Docker Desktop (a Windows app by Docker Inc., free for personal
# use) runs the garden's programs, with buttons to start, stop and read their
# messages. Three checks: is it installed, is it running, and is it shared with
# the garden's Ubuntu ("WSL integration")? Each says what to do if not.
# Docker Desktop installs for all users (Program Files) or, newer, for one user (AppData).
function Docker-Desktop-Dir {
    foreach ($d in (Join-Path $env:ProgramFiles "Docker\Docker"), (Join-Path $env:LOCALAPPDATA "Programs\DockerDesktop")) {
        if (Test-Path (Join-Path $d "Docker Desktop.exe")) { return $d }
    }
    return (Join-Path $env:ProgramFiles "Docker\Docker")
}
$DockerDesktopExe = Join-Path (Docker-Desktop-Dir) "Docker Desktop.exe"
$DockerDesktopCli = Join-Path (Docker-Desktop-Dir) "resources\bin\docker.exe"
function Docker-Desktop-Up {
    if (-not (Test-Path $DockerDesktopCli)) { return $false }
    & $DockerDesktopCli info *> $null
    return ($LASTEXITCODE -eq 0)
}
function Docker-Desktop-In-Wsl {
    $os = (& wsl.exe -d $T.distro -- docker info --format "{{.OperatingSystem}}" 2>$null) -join ""
    return ($os -match 'Docker Desktop')
}
function Prepare-Docker-Desktop {
    Say "Docker Desktop (the easy way to manage the garden's programs)"
    Note "A Windows app by Docker Inc. Free for personal use and small businesses."
    Note "Read: https://docs.docker.com/desktop/setup/install/windows-install/"
    if (-not (Test-Path $DockerDesktopExe)) {
        Note "It is not on this PC yet."
        if ((Get-Command winget.exe -ErrorAction SilentlyContinue) -and (Confirm-Step "Install Docker Desktop now with winget (Windows' own app installer)?")) {
            Run-Console "winget.exe" "install -e --id Docker.DockerDesktop" | Out-Null
        } else {
            Start-Process "https://docs.docker.com/desktop/setup/install/windows-install/"
            Note "Download and install it from the page that just opened, then start the install again."
        }
        $script:DockerDesktopExe = Join-Path (Docker-Desktop-Dir) "Docker Desktop.exe"
        $script:DockerDesktopCli = Join-Path (Docker-Desktop-Dir) "resources\bin\docker.exe"
        if (-not (Test-Path $DockerDesktopExe)) { Fail "Docker Desktop is not installed yet"; return $false }
    }
    if (-not (Docker-Desktop-Up)) {
        Note "Starting Docker Desktop. The first time, it shows its own welcome and terms:"
        Note "read them and accept there; signing in to a Docker account is not needed (Skip)."
        Start-Process $DockerDesktopExe
        $ok = $false
        for ($i = 0; $i -lt 60; $i++) { Start-Sleep -Seconds 5; if (Docker-Desktop-Up) { $ok = $true; break } }
        if (-not $ok) { Fail "Docker Desktop did not start within 5 minutes. Finish its welcome window, then start the install again."; return $false }
    }
    Note "Docker Desktop is running."
    if (-not (Docker-Desktop-In-Wsl)) {
        Say "One switch in Docker Desktop"
        Note "Docker Desktop > Settings (the gear, top right) > Resources > WSL integration:"
        Note "switch ON '$($T.distro)', then click 'Apply & restart'."
        Note "This window waits and carries on by itself when it sees the switch (up to 10 minutes)."
        $ok = $false
        for ($i = 0; $i -lt 120; $i++) { Start-Sleep -Seconds 5; if (Docker-Desktop-In-Wsl) { $ok = $true; break } }
        if (-not $ok) { Fail "Docker Desktop is not shared with '$($T.distro)' yet. Switch it on, then start the install again."; return $false }
    }
    Note "Connected. The garden's programs will show in Docker Desktop, under Containers, as 'junsgarden'."
    return $true
}

# Make sure a garden computer has been chosen; ask if not.
# Also: on another computer, make sure its saved address still reaches it (see below).
function Need-Target {
    if (-not $T.mode) { return (Choose-Target) }
    if ($T.mode -eq "remote") { [void](Update-Target-Address) }
    return $true
}

# ---------- keeping the garden computer's address, even when it changes ----------------
# A router gives each computer an address, and without a reservation (guide 14)
# that address can change after a restart. So the menu also remembers the garden
# computer by its network card's fixed ID (its MAC address) and its name. When
# nothing answers at the saved address, it looks at the computers at home with
# the SSH door open (Garden-Network.ps1), finds the one with the same network
# card, and saves its new address. Nothing is sent anywhere.

# The network card ID (MAC) of a computer at home, from this PC's own list of
# neighbours (Windows fills it in after one knock on the SSH door).
function Get-Mac($ip) {
    if (-not (Test-Port22 $ip)) { return "" }
    $n = Get-NetNeighbor -IPAddress $ip -ErrorAction SilentlyContinue |
        Where-Object { $_.LinkLayerAddress -and $_.LinkLayerAddress -ne "00-00-00-00-00-00" } | Select-Object -First 1
    if ($n) { return $n.LinkLayerAddress } else { return "" }
}
# Remember the garden computer by its network card and name, next to its address.
function Remember-Garden-Computer {
    if ($T.mode -ne "remote" -or -not $T.ip) { return }
    $mac = Get-Mac $T.ip
    if ($mac) { $T.mac = $mac }
    try { $n = [Net.Dns]::GetHostEntry($T.ip).HostName; if ($n -and $n -ne $T.ip) { $T.host = $n } } catch { }
    Save-Target
}
# Still at its saved address? If not, find it again by its network card (or its
# name) and save the new address. Quiet: for the weekly copy, nobody watching.
function Update-Target-Address([switch]$Quiet) {
    if ($T.mode -ne "remote" -or -not $T.ip) { return $true }
    if (Test-Port22 $T.ip) { if (-not $T.mac) { Remember-Garden-Computer }; return $true }
    if (-not $T.mac -and -not $T.host) { return $false }
    if (-not $Quiet) { Note "The garden computer does not answer at $($T.ip). Looking for it on your home network..." }
    foreach ($c in @(Find-Home-Computers)) {
        $same = ($T.mac -and (Get-Mac $c.IP) -eq $T.mac) -or (-not $T.mac -and $T.host -and $c.Name -eq $T.host)
        if ($same) {
            $old = $T.ip; $T.ip = $c.IP; Save-Target
            if (-not $Quiet) { Note "Found it: its address changed from $old to $($c.IP). Saved for next time." }
            return $true
        }
    }
    if (-not $Quiet) { Warn "not found at home. Is it switched on? Menu T sets its address by hand." }
    return $false
}
# More 21: find the garden computer again now (or pick it from the list).
function Find-Garden-Again {
    if (-not $T.mode) { [void](Choose-Target); return }
    if ($T.mode -ne "remote") { Note "The garden is on THIS PC (Windows' own Ubuntu): its address cannot get lost."; return }
    if (Update-Target-Address) {
        Note "The garden computer answers at $($T.ip)$(if ($T.mac) { ", network card $($T.mac)" })$(if ($T.host) { ", name $($T.host)" })."
        return
    }
    Note "Choose it again:"
    [void](Choose-Target)
    Remember-Garden-Computer
}

# Run "bash garden.sh <job>" on the garden computer, in this window.
function Invoke-Garden($job, $envs = "") {
    if (-not (Need-Target)) { return 1 }
    if ($T.mode -eq "remote") {
        return (Run-Console $SshExe "-t -o StrictHostKeyChecking=accept-new $($T.user)@$($T.ip) `"cd ~/$RemoteDir && ${envs}bash garden.sh $job`"")
    } else {
        return (Run-Console "wsl.exe" "-d $($T.distro) --cd ~ -- bash -lc `"cd ~/$RemoteDir && ${envs}bash garden.sh $job`"")
    }
}

# ---------- getting the garden computer ready -------------------------------------------
function Test-Port22($ip) {
    $c = New-Object Net.Sockets.TcpClient
    try { $r = $c.BeginConnect($ip, 22, $null, $null); return ($r.AsyncWaitHandle.WaitOne(5000) -and $c.Connected) }
    catch { return $false } finally { $c.Close() }
}

# Before installing on another computer: check SSH answers, then add this PC's SSH key there.
function Prepare-Remote {
    if (-not (Get-Command ssh.exe -ErrorAction SilentlyContinue)) {
        Fail "Windows' SSH program is missing. Settings > System > Optional features > Add 'OpenSSH Client'."
        return $false
    }
    Say "1. Can this PC reach $($T.ip) on the SSH port (22)?"
    if (-not (Test-Port22 $T.ip)) { Fail "no answer. Check the address, and that the SSH server is installed on it."; return $false }
    Note "yes"
    Say "2. A key, so you type that computer's password only once"
    Note "A key is a pair of files: the private half stays on this PC (in your .ssh folder),"
    Note "the public half is added on the garden computer. Read: https://www.ssh.com/academy/ssh/keygen"
    $sshDir = Join-Path $env:USERPROFILE ".ssh"; $key = Join-Path $sshDir "id_ed25519"
    if (-not (Test-Path $key)) {
        New-Item -ItemType Directory -Force $sshDir | Out-Null
        & cmd.exe /c "ssh-keygen -q -t ed25519 -N `"`" -f `"$key`" -C garden-%COMPUTERNAME%"
    }
    $pub = (Get-Content "$key.pub" -Raw).Trim()
    $script = "umask 077; mkdir -p ~/.ssh; touch ~/.ssh/authorized_keys; grep -qxF '$pub' ~/.ssh/authorized_keys || echo '$pub' >> ~/.ssh/authorized_keys; echo key-ok"
    $b64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($script))
    Note "Type the password for $($T.user)@$($T.ip) if asked (it is not shown)."
    if ((Run-Console $SshExe "-o StrictHostKeyChecking=accept-new $($T.user)@$($T.ip) `"echo $b64 | base64 -d | bash`"") -ne 0) {
        Fail "the key was not accepted (wrong password or user name?)"; return $false
    }
    return $true
}

# Before installing on this PC: switch on WSL and install Ubuntu (Microsoft's own steps).
function Prepare-Wsl {
    Say "Windows' built-in Ubuntu (WSL)"
    Note "WSL is part of Windows (by Microsoft). It runs a real Ubuntu next to Windows."
    Note "Read: https://learn.microsoft.com/windows/wsl/about"
    & wsl.exe --status *> $null
    if ($LASTEXITCODE -ne 0) {
        Note "WSL is not switched on yet. Windows asks for permission (click Yes), and"
        Note "then you must RESTART Windows once."
        Note "After the restart, double-click Start-Here-Windows.cmd again and click through"
        Note "the same pages: it carries on from here."
        if (Confirm-Step "Switch on WSL now?") { Start-Process wsl.exe -ArgumentList "--install --no-distribution" -Verb RunAs -Wait }
        Set-Status "RESTART"
        return $false
    }
    # Only a very old WSL (one that cannot show its version) needs updating:
    # "wsl --install" below brings an up-to-date one anyway. Updating asks
    # Windows' permission, so say so first instead of seeming to hang.
    & wsl.exe --version *> $null
    if ($LASTEXITCODE -ne 0) {
        Note "WSL is an old version. Updating it now: Windows asks for permission (click Yes)."
        Run-Console "wsl.exe" "--update" | Out-Null
    }
    $have = (& wsl.exe -l -q) -replace "`0", "" | ForEach-Object { $_.Trim() } | Where-Object { $_ -eq $T.distro }
    if (-not $have) {
        Note "Installing Ubuntu 24.04 LTS from Microsoft's store, named '$($T.distro)'."
        Note "Read: https://ubuntu.com/desktop/wsl"
        if ((Run-Console "wsl.exe" "--install Ubuntu-24.04 --name $($T.distro) --no-launch") -ne 0) { Fail "Ubuntu did not install"; return $false }
    }
    # Ubuntu is there, but has it got your own (non-administrator) user yet?
    # (Not, if this is the first time, or if the first try was interrupted.)
    $uid = ((& wsl.exe -d $T.distro -- id -u) -join "").Trim()
    if ($uid -eq "0") {
        $u = Ask "Choose a user name for Ubuntu (lowercase letters, e.g. jun)" "jun"
        if ($u -notmatch '^[a-z][a-z0-9_-]*$') { Fail "lowercase letters and numbers only"; return $false }
        Run-Console "wsl.exe" "-d $($T.distro) -u root -- useradd -m -s /bin/bash -G sudo $u" | Out-Null
        Note "Now choose a password for '$u' in Ubuntu (nothing shows as you type):"
        Run-Console "wsl.exe" "-d $($T.distro) -u root -- passwd $u" | Out-Null
        $conf = "[boot]`nsystemd=true`n[user]`ndefault=$u`n"
        $b64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($conf))
        Run-Console "wsl.exe" "-d $($T.distro) -u root -- bash -c `"echo $b64 | base64 -d > /etc/wsl.conf`"" | Out-Null
        & wsl.exe --terminate $T.distro | Out-Null
        $T.user = $u; Save-Target
    }
    return $true
}

# Where the separate downloads may be: the garden folder, the real Desktop
# (it may live in OneDrive), then Downloads.
function Addon-Dirs {
    @([pscustomobject]@{ Name = "garden folder"; Path = $Root },
      [pscustomobject]@{ Name = "Desktop"; Path = [Environment]::GetFolderPath("Desktop") },
      [pscustomobject]@{ Name = "Downloads"; Path = (Join-Path $env:USERPROFILE "Downloads") })
}

# The garden folder inside OneDrive: OneDrive uploads everything in it. Say so before
# each install, with the size and the way out (nothing is stopped or moved here).
function OneDrive-Note {
    $od = @($env:OneDrive, $env:OneDriveCommercial, $env:OneDriveConsumer) | Where-Object { $_ } | Select-Object -Unique
    if (-not ($od | Where-Object { $Root.StartsWith($_, [StringComparison]::OrdinalIgnoreCase) })) { return }
    $mb = [int]((Get-ChildItem $Root -Recurse -File -ErrorAction SilentlyContinue | Measure-Object Length -Sum).Sum / 1MB)
    Say "Your garden folder is inside OneDrive"
    Note "Folder: $Root"
    Note "OneDrive copies everything in it to the internet: now about $mb MB."
    Note "OneDrive's free plan is 5 GB, and the garden's downloads and backup copies"
    Note "grow past that. The easy way out: close this window and double-click"
    Note "Start-Here-Windows.cmd. It offers to move the garden folder out of OneDrive"
    Note "for you (to C:\Users\$env:USERNAME\junsgarden-for-jun, or a place you pick),"
    Note "checks every file arrived, and carries on from there. The install also works here."
    Note "Guide: START-HERE\00b-before-you-start.md, 'If your Desktop is in OneDrive'."
}

# Windows' path to the inside of an Ubuntu (WSL). \\wsl.localhost is the newer
# name; older Windows 10 has only \\wsl$. Both are Microsoft's, with the same files.
function Wsl-Root($distro) {
    foreach ($p in "\\wsl.localhost\$distro", "\\wsl`$\$distro") { if (Test-Path "$p\") { return $p } }
    return "\\wsl`$\$distro"
}

# Ubuntu's programs read text files with Linux line endings (LF). A Windows editor
# can save Windows endings (CRLF) instead, and then bash stops with "$'\r': command
# not found". Before copying, find such files among the ones Ubuntu runs, keep a
# copy in logs\, and change only their line endings. The .cmd files stay as they are.
function Fix-Line-Endings {
    $names = @("*.sh", "*.py", "*.tsv", "*.yml", "*.yaml", "*.template", "*.conf", "*.service", "Dockerfile", "*.env.example")
    # Folders inside the garden folder to leave alone (matched on the part INSIDE it,
    # so a garden folder under, say, AppData\Local is still checked).
    $skip = '(^|\\)(local|backups|logs|\.git|target|build|\.gradle|node_modules)\\'
    $keep = Join-Path $Root ("logs\line-endings_" + (Get-Date -Format "yyyy-MM-dd_HHmmss"))
    $n = 0
    foreach ($f in (Get-ChildItem $Root -Recurse -File -Include $names -ErrorAction SilentlyContinue)) {
        $rel = $f.FullName.Substring($Root.Length).TrimStart('\')
        if ($rel -match $skip) { continue }
        $bytes = [IO.File]::ReadAllBytes($f.FullName)
        if ([Array]::IndexOf($bytes, [byte]13) -lt 0) { continue }
        $copy = Join-Path $keep $rel
        New-Item -ItemType Directory -Force (Split-Path $copy) | Out-Null
        Copy-Item $f.FullName $copy
        $text = [Text.Encoding]::UTF8.GetString($bytes) -replace "`r`n", "`n"
        [IO.File]::WriteAllBytes($f.FullName, [Text.Encoding]::UTF8.GetBytes($text))
        Note "Linux line endings again: $rel"
        $n++
    }
    if ($n -gt 0) { Note "$n file(s) had Windows line endings (a Windows editor adds them). The old copies are in $keep" }
}

# Docker Desktop on this PC is fine: the garden does not use it. It runs its own
# Docker inside Ubuntu, so it works the same for everyone and keeps running when
# Docker Desktop is closed. Only Docker Desktop's "WSL integration" for the
# garden's Ubuntu must be off; garden.sh checks that and says how.
function Docker-Desktop-Note {
    if (-not (Test-Path (Join-Path (Docker-Desktop-Dir) "Docker Desktop.exe"))) { return }
    Note "Docker Desktop is on this PC. That is fine: the garden uses its own Docker inside Ubuntu."
    Note "Keep Docker Desktop's WSL integration OFF for '$($T.distro)' (Settings > Resources > WSL integration)."
}

# Pack the garden folder (not settings or private data) and copy it to the garden computer.
function Copy-Garden {
    Say "Copying the garden folder (not your settings or private data)"
    Fix-Line-Endings
    # The separate downloads (the photos, the wiki's pages), if they are in the
    # garden folder, on the Desktop or in Downloads: they go straight to the
    # garden computer after the folder (no second copy on this PC).
    $addons = @()
    foreach ($z in "junsgarden-photos-addon.zip", "junsgarden-wiki-addon.zip") {
        foreach ($dir in (Addon-Dirs)) {
            $p = Join-Path $dir.Path $z
            if (Test-Path $p) { $addons += $p; Note "Found $z (in your $($dir.Name)); it goes along."; break }
        }
    }
    $tmp = Join-Path $env:TEMP "junsgarden-upload.tgz"
    if (Test-Path $tmp) { Remove-Item $tmp -Force }
    & $TarExe -czf $tmp --exclude=./local --exclude=./.env --exclude=./.git --exclude=./backups --exclude=./logs --exclude="./junsgarden-*-addon*.zip" `
        --exclude=./.garden-target-windows.txt --exclude="./CHECKLIST*.txt" --exclude=./checklist-history.log -C $Root .
    if ($LASTEXITCODE -ne 0) { Fail "packing failed"; return $false }
    Note ("packed {0:N0} MB" -f ((Get-Item $tmp).Length / 1MB))
    $backups = Join-Path $Root "backups"
    $nb = Get-ChildItem $backups -Filter "junsgarden-backup_*.tar" -File -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending | Select-Object -First 1
    if ($T.mode -eq "remote") {
        $tgt = "$($T.user)@$($T.ip)"
        if ((Run-Console $ScpExe "`"$tmp`" ${tgt}:junsgarden-upload.tgz") -ne 0) { Fail "copying failed"; return $false }
        Run-Console $SshExe "$tgt `"mkdir -p ~/$RemoteDir/backups && tar -xzf ~/junsgarden-upload.tgz -C ~/$RemoteDir && rm ~/junsgarden-upload.tgz`"" | Out-Null
        if ($nb) {
            Note "Copying your garden backup ($($nb.Name), $([int]($nb.Length / 1MB)) MB). This takes a few minutes."
            if ((Run-Console $ScpExe "`"$($nb.FullName)`" ${tgt}:$RemoteDir/backups/") -ne 0) { Warn "the backup did not copy" }
        }
        foreach ($a in $addons) {
            Note "Copying $(Split-Path -Leaf $a) ($([int]((Get-Item $a).Length / 1MB)) MB). This takes a few minutes."
            if ((Run-Console $ScpExe "`"$a`" ${tgt}:$RemoteDir/") -ne 0) { Warn "$(Split-Path -Leaf $a) did not copy; menu 6, then 11 on the garden computer loads it later" }
        }
    } else {
        $wtmp = (& wsl.exe -d $T.distro -- wslpath -a ($tmp -replace '\\', '/')).Trim()
        Run-Console "wsl.exe" "-d $($T.distro) --cd ~ -- bash -c `"mkdir -p ~/$RemoteDir/backups && tar -xzf '$wtmp' -C ~/$RemoteDir`"" | Out-Null
        if ($nb) {
            Note "Copying your garden backup ($($nb.Name))."
            $wnb = (& wsl.exe -d $T.distro -- wslpath -a ($nb.FullName -replace '\\', '/')).Trim()
            Run-Console "wsl.exe" "-d $($T.distro) --cd ~ -- cp '$wnb' ~/$RemoteDir/backups/" | Out-Null
        }
        foreach ($a in $addons) {
            Note "Copying $(Split-Path -Leaf $a) ($([int]((Get-Item $a).Length / 1MB)) MB)."
            $wa = (& wsl.exe -d $T.distro -- wslpath -a ($a -replace '\\', '/')).Trim()
            Run-Console "wsl.exe" "-d $($T.distro) --cd ~ -- cp '$wa' ~/$RemoteDir/" | Out-Null
        }
    }
    Remove-Item $tmp -Force
    return $true
}

# Menu 1: get the garden computer ready, copy the garden, run garden.sh install there.
function Do-Install {
    if (-not (Need-Target)) { return }
    Save-Target   # remember the garden computer for next time (no passwords are saved)
    Say "Install on the garden computer: $(Target-Text)"
    OneDrive-Note
    if ($T.mode -eq "remote") { if (-not (Prepare-Remote)) { return }; Remember-Garden-Computer }
    else {
        if (-not (Prepare-Wsl)) { return }
        if ($T.engine -eq "desktop") { if (-not (Prepare-Docker-Desktop)) { return } } else { Docker-Desktop-Note }
    }
    if (-not (Choose-Webname)) { return }
    if (-not (Copy-Garden)) { return }
    $envs = "GARDEN_REMOTE_INSTALL=1 "
    if ($Name) { $envs += "GARDEN_NAME=$Name " }
    if ($Email) { $envs += "GARDEN_EMAIL=$Email " }
    # Only when -Parts was really given (the guided installer gives it). A typed
    # [string] that was left out is "", not $null, and "" would mean "no parts".
    if ($PartsGiven) { $envs += "GARDEN_PARTS=$Parts " }
    if ($Yes) { $envs += "ASSUME_YES=1 " }
    # Which Docker runs it on this PC (garden.sh remembers it in .garden-docker).
    if ($T.mode -eq "wsl") { $envs += "GARDEN_DOCKER=$($T.engine) " }
    Say "Running the install ON the garden computer (it explains every step)"
    if ((Invoke-Garden "install" $envs) -ne 0) { Fail "the install stopped. Read the lines above; choose INSTALL again to carry on."; return }
    if ($T.mode -eq "wsl") { Keep-Wsl-Running }
    Say "Finished. Menu 4 opens your garden."
    # The welcome page: every guide (web page and PDF), one click away, from this PC.
    $welcome = Join-Path $Root "00-WELCOME.html"
    if (Test-Path $welcome) { Note "Opening your welcome page: every guide, one click away."; Start-Process $welcome }
}

# Optional task that keeps Windows' Ubuntu running while you are signed in.
function Keep-Wsl-Running {
    Note "Windows switches its Ubuntu off when nothing uses it. A small task can keep it"
    Note "running while you are signed in to Windows, so the website stays up."
    if (-not (Confirm-Step "Add that task (Task Scheduler, 'Jun's Garden keep running')?")) { return }
    $act = New-ScheduledTaskAction -Execute "wsl.exe" -Argument "-d $($T.distro) -- sleep infinity"
    $trg = New-ScheduledTaskTrigger -AtLogOn -User $env:USERNAME
    $set = New-ScheduledTaskSettingsSet -ExecutionTimeLimit ([TimeSpan]::Zero) -Hidden
    Register-ScheduledTask -TaskName "Jun's Garden keep running" -Action $act -Trigger $trg -Settings $set -Force | Out-Null
    Start-ScheduledTask -TaskName "Jun's Garden keep running"
    Note "Added. See or remove it in Task Scheduler."
}

# ---------- things only Windows does ------------------------------------------------
function Open-Website {
    if (-not (Need-Target)) { return }
    if ($T.mode -eq "wsl") { Start-Process "http://localhost:8030"; return }
    Say "Opening a secure tunnel to the garden computer's website"
    Note "A new window opens and stays open: that IS the tunnel. Close it to stop."
    Start-Process $SshExe -ArgumentList "-N -L ${TunnelPort}:127.0.0.1:8030 $($T.user)@$($T.ip)"
    Start-Sleep -Seconds 4
    Start-Process "http://localhost:$TunnelPort"
}
# Open a terminal on the garden computer in a new window.
function Open-Terminal {
    if (-not (Need-Target)) { return }
    if ($T.mode -eq "remote") { Start-Process $SshExe -ArgumentList "$($T.user)@$($T.ip)" }
    else { Start-Process wsl.exe -ArgumentList "-d $($T.distro) --cd ~/$RemoteDir" }
}
# More 17 (after garden.sh webname ran on the garden computer): make THIS PC's
# copy of the garden folder say the same web name. Otherwise a later "Install"
# from this PC would copy the old name back. The file list is the same as
# WEBNAME_FILES in garden.sh; the old files are kept in logs\ first.
$WebnameFiles = @(
    "site/privacy.html", "site/apps/version.json", "rust/junsgarden/templates/index.html",
    "rust/junsgarden/src/main.rs", "rust/junsgarden/src/php.rs", "rust/junsgarden/src/pages/index.rs",
    "rust/junsgarden/src/pages/blog.rs", "rust/junsgarden/src/orchestrator/ai.rs", "beds/freeai.py",
    "docker-compose.yml", "android-app/app/build.gradle.kts",
    "android-app/app/src/main/kotlin/org/jasonagibson/junsgarden/app/Updater.kt"
)
function Get-Webname($json) {   # the https://NAME in the phone app's update file
    if ($json -match 'https://([^/"]+)') { return $Matches[1] } else { return "" }
}
# Tidy a typed web name like garden.sh does: lowercase; no http(s)://, www., port or path.
function Clean-Webname($raw) {
    $n = ($raw -replace '\s', '').ToLower()
    $n = $n -replace '^https?://', '' -replace '^www\.', '' -replace '[/:].*$', '' -replace '\.$', ''
    return $n
}
function Test-Webname($n) { return ($n -match '^([a-z0-9]([a-z0-9-]*[a-z0-9])?\.)+[a-z]{2,}$') }
# The name THIS PC's copy of the files uses now.
function Get-Webname-Here { return (Get-Webname ([IO.File]::ReadAllText((Join-Path $Root "site\apps\version.json")))) }

# Before installing from this PC: keep junsgarden.zapto.org, or a new name?
# A new name is put into THIS PC's copy first, so the garden computer gets it too.
function Choose-Webname {
    $old = Get-Webname-Here
    $new = $WebName
    if (-not $new -and -not $Yes) {
        Say "Your web name"
        Note "Your garden's address on the internet is a web name. It is now $old."
        Note "  1) Keep $old (recommended). Before going online, add it to your own"
        Note "     free No-IP account (guide 04, Part A)."
        Note "  2) Choose a new name now (Website name swap)."
        Note "You can change it later too: More, then 17."
        if ((Read-Host "Type 1 or 2 (Enter = 1)") -ne "2") { return $true }
        Note "Getting a web name (free): an account at https://www.noip.com, then Dynamic DNS,"
        Note "No-IP Hostnames, Create Hostname. Free names are confirmed every 30 days (No-IP"
        Note "emails a link). You can type the name now and add it at No-IP before going online."
        $new = Read-Host "New web name (for example momsgarden.zapto.org)"
    }
    if (-not $new) { return $true }
    $clean = Clean-Webname $new
    if ($clean -ne $new) { Note "Using: $clean   (http://, https://, www. and anything after / are left out)" }
    if (-not (Test-Webname $clean)) { Fail "`"$clean`" does not look like a web name (like name.zapto.org)"; return $false }
    if ($clean -ne $old) { Set-Webname-Here $old $clean }
    return $true
}

function Sync-Webname-Here {
    $vj = "$RemoteDir/site/apps/version.json"
    if ($T.mode -eq "remote") { $there = (& $SshExe "$($T.user)@$($T.ip)" "cat ~/$vj") -join "`n" }
    else { $there = (& wsl.exe -d $T.distro --cd ~ -- cat $vj) -join "`n" }
    $new = Get-Webname $there
    $old = Get-Webname-Here
    if (-not $new -or -not $old -or $new -eq $old) { return }
    Set-Webname-Here $old $new
}
function Set-Webname-Here($old, $new) {
    Say "Website name swap on this PC's copy: $old -> $new"
    $keep = Join-Path $Root ("logs\web-name-change-windows_" + (Get-Date -Format "yyyy-MM-dd_HHmm"))
    $utf8 = New-Object Text.UTF8Encoding($false)   # no BOM, same bytes as before apart from the name
    foreach ($rel in $WebnameFiles) {
        $f = Join-Path $Root ($rel -replace '/', '\')
        if (-not (Test-Path $f)) { continue }
        $text = [IO.File]::ReadAllText($f, $utf8)
        if (-not $text.Contains($old)) { continue }
        $copy = Join-Path $keep ($rel -replace '/', '\')
        New-Item -ItemType Directory -Force (Split-Path $copy) | Out-Null
        Copy-Item $f $copy
        [IO.File]::WriteAllText($f, $text.Replace($old, $new), $utf8)   # plain text, not a pattern
        Note "changed: $rel"
    }
    Note "The old files are kept in $keep"
}

# More 20 (after garden.sh rename ran on the garden computer): make THIS PC's copy
# of the garden folder match, so a later "Install" from this PC does not copy the
# old names back. garden.sh writes the list in logs/last-rename.txt:
#   edit<TAB>file        a file whose text changed: copied here from the garden computer
#   move<TAB>old<TAB>new a site file or folder that was renamed: renamed here too
# This PC's old files are kept in logs\rename-windows_DATE first.
function Sync-Renamed-Here {
    $list = "$RemoteDir/logs/last-rename.txt"
    if ($T.mode -eq "remote") {
        $tgt = "$($T.user)@$($T.ip)"
        $lines = & $SshExe $tgt "cat ~/$list 2>/dev/null"
    } else {
        $lines = & wsl.exe -d $T.distro --cd ~ -- bash -c "cat ~/$list 2>/dev/null"
        # The garden folder's real place in Ubuntu (it may have moved; a signpost points there).
        $real = ("" + (& wsl.exe -d $T.distro --cd ~ -- bash -c "cd ~/$RemoteDir && pwd -P")).Trim()
    }
    if (-not $lines) { return }
    Say "Renaming on this PC's copy too, so the two copies stay the same"
    $keep = Join-Path $Root ("logs\rename-windows_" + (Get-Date -Format "yyyy-MM-dd_HHmmss"))
    foreach ($line in $lines) {
        $p = "$line".Split("`t")
        if ($p.Count -lt 2 -or $p[1] -notmatch '^[A-Za-z0-9._/-]+$') { continue }
        $here = Join-Path $Root ($p[1] -replace '/', '\')
        if ($p[0] -eq "move" -and $p.Count -ge 3 -and $p[2] -match '^[A-Za-z0-9._/-]+$') {
            $to = Join-Path $Root ($p[2] -replace '/', '\')
            if ((Test-Path $here) -and -not (Test-Path $to)) {
                New-Item -ItemType Directory -Force (Split-Path $to) | Out-Null
                Move-Item $here $to
                Note "renamed: $($p[1]) -> $($p[2])"
            }
        } elseif ($p[0] -eq "edit") {
            if (Test-Path $here) {
                $copy = Join-Path $keep ($p[1] -replace '/', '\')
                New-Item -ItemType Directory -Force (Split-Path $copy) | Out-Null
                Copy-Item $here $copy
            }
            New-Item -ItemType Directory -Force (Split-Path $here) | Out-Null
            if ($T.mode -eq "remote") { & $ScpExe -q "${tgt}:$RemoteDir/$($p[1])" "$here" }
            else { Copy-Item -Force ((Wsl-Root $T.distro) + (("$real/" + $p[1]) -replace '/', '\')) $here }
            Note "updated: $($p[1])"
        }
    }
    if (Test-Path $keep) { Note "This PC's old files are kept in $keep" }
}

# More 18: two shortcuts on the Windows desktop (guide 20).
#   "Garden terminal"  opens SSH to the garden computer (or Windows' own Ubuntu)
#   "Garden screen"    opens Remote Desktop Connection to it (a separate computer only;
#                      switch Remote Desktop on in its Settings first, guide 20)
function Make-Shortcuts {
    if (-not (Need-Target)) { return }
    $desk = [Environment]::GetFolderPath("Desktop")
    $sh = New-Object -ComObject WScript.Shell
    $lnk = $sh.CreateShortcut((Join-Path $desk "Garden terminal.lnk"))
    if ($T.mode -eq "remote") {
        $lnk.TargetPath = $SshExe; $lnk.Arguments = "-t $($T.user)@$($T.ip)"
    } else {
        $lnk.TargetPath = "wsl.exe"; $lnk.Arguments = "-d $($T.distro) --cd ~"
    }
    $lnk.Description = "Jun's Garden: a terminal on the garden computer (SSH, encrypted)"
    $lnk.Save()
    Note "Made: Garden terminal (on your desktop)"
    if ($T.mode -eq "remote") {
        $rdp = Join-Path $desk "Garden screen.rdp"
        # A plain text file Remote Desktop Connection reads: which computer, which user.
        Set-Content -Path $rdp -Encoding ASCII -Value @("full address:s:$($T.ip)", "username:s:$($T.user)", "prompt for credentials:i:1")
        Note "Made: Garden screen (Remote Desktop to $($T.ip)). Switch Remote Desktop on in the"
        Note "garden computer's Settings first (guide 20)."
    } else {
        Note "Garden screen is only for a separate garden computer (this one runs inside Windows)."
    }
}

# More 10: make a backup on the garden computer and copy it to this PC.
function Copy-Backup-Here {
    if (-not (Need-Target)) { return }
    if ((Invoke-Garden "backup") -ne 0) { Fail "the backup stopped"; return }
    $dest = Join-Path $Root "backups"; New-Item -ItemType Directory -Force $dest | Out-Null
    if ($T.mode -eq "remote") {
        $tgt = "$($T.user)@$($T.ip)"
        $name = (& $SshExe $tgt "ls -t ~/$RemoteDir/backups/junsgarden-backup_*.tar | head -1").Trim()
        if ((Run-Console $ScpExe "${tgt}:$name `"$dest`"") -eq 0) { Note "Copied to $dest" }
    } else {
        $wdest = (& wsl.exe -d $T.distro -- wslpath -a ($dest -replace '\\', '/')).Trim()
        Run-Console "wsl.exe" "-d $($T.distro) --cd ~ -- bash -c `"cp `$(ls -t ~/$RemoteDir/backups/junsgarden-backup_*.tar | head -1) '$wdest'/`"" | Out-Null
        Note "Copied to $dest"
    }
}

# ---------- an automatic copy of the backups, in a folder YOU pick ----------------------
# The garden computer backs itself up every night at 2:30 am (on its own disk).
# A second copy somewhere else protects you if that disk dies. This copies the
# newest backup into the folder you chose (More 13): on this PC, on a NAS
# (\\NAS\share), in Google Drive for desktop (G:), on a USB drive... Without a
# choice it is backups\from-garden-computer\ in the garden folder. The newest 8
# are kept. It makes no new backup and changes nothing on the garden computer.
# It writes what it did to logs\backup-copy.log.
function Backup-Copy-Dir {
    if ($T.copyto) { return $T.copyto }
    return (Join-Path $Root "backups\from-garden-computer")
}

# Pick where the copies go: a folder window (this PC, Network for a NAS, Google
# Drive for desktop, a USB drive), or a typed network path. The garden backups go
# into a "junsgarden-backups" folder inside it.
function Choose-Copy-Folder {
    Note "Where should the copies go? Now: $(Backup-Copy-Dir)"
    Note "  1) Pick a folder in a window (this PC, a USB drive, Google Drive for desktop,"
    Note "     or a NAS under 'Network' on the left)"
    Note "  2) Type it: a folder like C:\Backups\weekly, or a network path like"
    Note "     \\192.168.1.20\backups (a NAS or a shared PC folder)"
    Note "  3) Keep it as it is"
    $c = Ask "Number" "3"
    $base = $null
    if ($c -eq "1") {
        Add-Type -AssemblyName System.Windows.Forms
        $dlg = New-Object System.Windows.Forms.FolderBrowserDialog
        $dlg.Description = "Pick where the garden's backup copies go (a folder named junsgarden-backups is made inside it)"
        $dlg.ShowNewFolderButton = $true
        if ($dlg.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) { $base = $dlg.SelectedPath } else { Note "Nothing picked; it stays as it is." }
    } elseif ($c -eq "2") {
        $base = Ask "Folder or network path" ""
    }
    if (-not $base) { return }
    $dir = Join-Path $base "junsgarden-backups"
    try {
        New-Item -ItemType Directory -Force $dir -ErrorAction Stop | Out-Null
        $probe = Join-Path $dir ".garden-test"; Set-Content $probe "test" -ErrorAction Stop; Remove-Item $probe -Force
    } catch {
        Fail "cannot write in $dir. A NAS: open it once in File Explorer and sign in with 'Remember my credentials' ticked, then try again."
        return
    }
    $od = @($env:OneDrive, $env:OneDriveCommercial, $env:OneDriveConsumer) | Where-Object { $_ }
    if ($od | Where-Object { $dir.StartsWith($_, [StringComparison]::OrdinalIgnoreCase) }) {
        Warn "that folder is inside OneDrive: 8 copies are several GB, more than OneDrive's free 5 GB."
    }
    $T.copyto = $dir; Save-Target
    Note "Copies go to: $dir"
}

function Copy-Newest-Quiet {
    $dest = Backup-Copy-Dir
    New-Item -ItemType Directory -Force (Join-Path $Root "logs") | Out-Null
    $log = Join-Path $Root "logs\backup-copy.log"
    $stamp = Get-Date -Format "yyyy-MM-dd HH:mm"
    try {
        New-Item -ItemType Directory -Force $dest -ErrorAction Stop | Out-Null
        if ($T.mode -eq "remote") {
            [void](Update-Target-Address -Quiet)   # its address may have changed since last week
            $tgt = "$($T.user)@$($T.ip)"
            # BatchMode: never ask a question (it runs while nobody is watching).
            $name = (& $SshExe -o BatchMode=yes -o ConnectTimeout=10 $tgt "ls -t ~/$RemoteDir/backups/junsgarden-backup_*.tar | head -1").Trim()
            if (-not $name) { throw "no backup found on the garden computer" }
            # Into this PC's temp folder first, then into the chosen folder (it may be a NAS or G:).
            & $ScpExe -q -o BatchMode=yes "${tgt}:$name" "$env:TEMP"
            if ($LASTEXITCODE -ne 0) { throw "scp failed" }
            Move-Item -Force (Join-Path $env:TEMP (Split-Path -Leaf $name)) $dest -ErrorAction Stop
        } else {
            # Windows reads the Ubuntu files directly (\\wsl.localhost\...), so any folder works as the destination.
            $name = (& wsl.exe -d $T.distro -- bash -c "ls -t ~/$RemoteDir/backups/junsgarden-backup_*.tar 2>/dev/null | head -1")
            $name = "$name".Trim()
            if (-not $name) { throw "no backup found in the garden" }
            Copy-Item -Force ((Wsl-Root $T.distro) + ($name -replace '/', '\')) $dest -ErrorAction Stop
        }
        # Keep the newest 8 copies, delete older ones (only in that folder).
        Get-ChildItem $dest -Filter "junsgarden-backup_*.tar" | Sort-Object LastWriteTime -Descending |
            Select-Object -Skip 8 | Remove-Item -Force
        $newest = Get-ChildItem $dest -Filter "junsgarden-backup_*.tar" | Sort-Object LastWriteTime -Descending | Select-Object -First 1
        if (-not $newest) { throw "the copy did not arrive in $dest" }
        Add-Content $log "$stamp OK  $($newest.Name)"
        Note "Copied: $($newest.Name) -> $dest"
    } catch {
        Add-Content $log "$stamp FAILED  $($_.Exception.Message)"
        Fail "the copy did not work: $($_.Exception.Message). Is the garden computer on?"
    }
}

# More 13: add or remove the weekly automatic copy of the backup to this PC.
function Schedule-Backup-Copy {
    if (-not (Need-Target)) { return }
    Say "A weekly copy of your garden backup, in a folder you pick (automatic)"
    Note "What: Windows Task Scheduler runs this menu's copy job every Sunday at 11:00."
    Note "      It copies the newest backup from the garden computer to:"
    Note "      $(Backup-Copy-Dir)  (the newest 8 are kept)"
    Note "Why:  a backup on the same computer as the garden does not survive a dead disk."
    Note "Read: https://learn.microsoft.com/windows/win32/taskschd/task-scheduler-start-page"
    Note "Needs: the garden computer switched on at that time, and the SSH key from the install."
    $taskName = "Jun's Garden - copy backup to this PC"
    $existing = Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
    if ($existing) {
        Note "It is already scheduled."
        if (Confirm-Step "Change where the copies go?" $false) { Choose-Copy-Folder; return }
        if (Confirm-Step "Remove the weekly copy?" $false) { Unregister-ScheduledTask -TaskName $taskName -Confirm:$false; Note "Removed." }
        return
    }
    Choose-Copy-Folder
    if (-not (Confirm-Step "Add the weekly copy now?")) {
        Note "OK. To do the same by hand any time: this menu, M, 10. Steps to schedule it"
        Note "yourself are in START-HERE/13-backups-and-folders.md ('A copy on your Windows PC')."
        return
    }
    $menuPath = Join-Path $PSScriptRoot "Garden-Menu.ps1"
    $act = New-ScheduledTaskAction -Execute "powershell.exe" -Argument "-NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -File `"$menuPath`" -Action copy-newest-quiet"
    $trg = New-ScheduledTaskTrigger -Weekly -DaysOfWeek Sunday -At 11am
    $set = New-ScheduledTaskSettingsSet -StartWhenAvailable -ExecutionTimeLimit (New-TimeSpan -Hours 2)
    Register-ScheduledTask -TaskName $taskName -Action $act -Trigger $trg -Settings $set -Force | Out-Null
    Note "Added. It also runs as soon as the PC is on, if it missed a Sunday."
    if (Confirm-Step "Make the first copy right now?") { Copy-Newest-Quiet }
}

# ---------- the menus (the same numbers as the Ubuntu menu) --------------------------------
function Banner($sub) {
    Clear-Host
    Write-Host "  ==============================================" -ForegroundColor Green
    if ($sub) { Write-Host "     Jun's Garden  -  $sub" -ForegroundColor Green } else { Write-Host "     Jun's Garden  (Windows)" -ForegroundColor Green }
    Write-Host "  ==============================================" -ForegroundColor Green
    Write-Host "  Garden computer: $(Target-Text)    (T to change)"
}

# The main menu (the same numbers and letters as the Ubuntu menu).
function Show-Menu {
    while ($true) {
        Banner ""
        Write-Host @"

   1) INSTALL  (first time, or carry on, or repair)

   2) Start the garden          3) Stop the garden
   4) Open the website          5) Checklist: is everything OK?
   6) Backups: back up, NAS, Google Drive   7) Restore a backup
   8) Settings: names, emails, passwords, keys, folders
   9) Parts: garden tools, wiki, n8n (on / off)
   U) Users and access: add, remove, guests, lock down
   K) API keys: paste, test or replace (Gemini, Gmail, ...)
   A) My addresses: every web address, and which need signing in

   H) Help file for someone helping you (no passwords in it)
   M) More (advanced)        T) Choose the garden computer
   L) Language on the garden computer: English / Korean / both
   0) Quit
"@
        $c = Read-Host "`n  Type a number or letter, then press Enter"
        switch ($c) {
            "1" { Do-Install }
            "2" { Invoke-Garden "start" | Out-Null }
            "3" { Invoke-Garden "stop" | Out-Null }
            "4" { Open-Website }
            "5" { Invoke-Garden "check" | Out-Null }
            "6" { Invoke-Garden "backups" | Out-Null }
            "7" { Invoke-Garden "restore" | Out-Null }
            "8" { Invoke-Garden "settings" | Out-Null }
            "9" { Invoke-Garden "parts" | Out-Null }
            { $_ -in "u", "U" } { Invoke-Garden "users" | Out-Null }
            { $_ -in "k", "K" } { Invoke-Garden "keys" | Out-Null }
            { $_ -in "a", "A" } { Invoke-Garden "addresses" | Out-Null }
            { $_ -in "l", "L" } { Invoke-Garden "language" | Out-Null }
            { $_ -in "h", "H" } { Invoke-Garden "helpme" | Out-Null }
            { $_ -in "m", "M" } { Show-More; continue }
            { $_ -in "t", "T" } { Choose-Target | Out-Null }
            "0" { return }
            default { Note "Please type one of the numbers or letters shown." }
        }
        Pause-Menu
    }
}

# The 'More' menu with the advanced jobs.
function Show-More {
    while ($true) {
        Banner "More"
        Write-Host @"

   1) The front door: HTTPS, your sign-in page, your web name
   2) Network: the garden computer's IP address and reserving it
   3) Email (Gmail) for password links, with a test email
   4) Set my password (website + sign-in page)
   5) Open a terminal on the garden computer
   6) Show the programs' recent messages (logs)
   7) Rebuild after editing Rust files
   8) Look inside the database (Adminer, on the garden computer: http://localhost:8082 there)
   9) Connect a local AI computer
  10) Copy the newest backup to THIS PC
  11) Look at the files that came from the author (and check them)
  12) Uninstall
  13) Weekly automatic copy of the backup to THIS PC (on / off)
  14) Subdomains with DuckDNS (wiki., n8n., ...): add, remove
  15) On / off: the whole garden, or only off the internet; check-up, clean-up
  16) AI cost control: LiteLLM gateway with a monthly limit (on / off)
  17) Website name swap: use another web name than junsgarden.zapto.org
  18) Desktop shortcuts: Garden terminal (SSH) and Garden screen (Remote Desktop)
  19) Reach the garden from anywhere, privately (Tailscale): on / off
  20) Rename or move a file or folder (every place that uses it follows)
  21) Find the garden computer again (if its address changed)

   0) Back
"@
        $c = Read-Host "`n  Type a number, then press Enter"
        switch ($c) {
            "1"  { Invoke-Garden "public" | Out-Null }
            "2"  { Invoke-Garden "network" | Out-Null }
            "3"  { Invoke-Garden "email" | Out-Null }
            "4"  { Invoke-Garden "password" | Out-Null }
            "5"  { Open-Terminal }
            "6"  { Invoke-Garden "logs" | Out-Null }
            "7"  { Invoke-Garden "rebuild" | Out-Null }
            "8"  { Invoke-Garden "dbweb" | Out-Null }
            "9"  { Invoke-Garden "ai" | Out-Null }
            "10" { Copy-Backup-Here }
            "11" { Invoke-Garden "files" | Out-Null }
            "12" { Invoke-Garden "uninstall" | Out-Null }
            "13" { Schedule-Backup-Copy }
            "14" { Invoke-Garden "subdomains" | Out-Null }
            "15" { Invoke-Garden "power" | Out-Null }
            "16" { Invoke-Garden "llm" | Out-Null }
            "17" { Invoke-Garden "webname" | Out-Null; Sync-Webname-Here }
            "18" { Make-Shortcuts }
            "19" { Invoke-Garden "tailscale" | Out-Null; Note "On this PC too: install Tailscale from https://tailscale.com/download and sign in with the same account." }
            "20" { Invoke-Garden "rename" | Out-Null; Sync-Renamed-Here }
            "21" { Find-Garden-Again }
            "0"  { return }
            default { Note "Please type one of the numbers shown." }
        }
        Pause-Menu
    }
}

# Files that came in a downloaded zip are marked "from the internet".
# Unblocking them stops Windows from asking about each one.
Get-ChildItem $Root -Recurse -File -Include *.ps1, *.cmd -ErrorAction SilentlyContinue | Unblock-File -ErrorAction SilentlyContinue

Load-Target
switch ($Action) {
    "menu"    { Show-Menu }
    "install" {
        Set-Status "RUNNING"
        Do-Install
        if ((Get-Content $StatusFile -ErrorAction SilentlyContinue) -ne "RESTART") {
            if ($script:HadFail) { Set-Status "FAILED" }
            else { Set-Status "DONE"; Say "All done here. You can close this window now." }
        }
    }
    "target"  { Choose-Target | Out-Null }
    "copy-backup" { Copy-Backup-Here }
    "open-website" { Open-Website }
    "copy-newest-quiet" { Copy-Newest-Quiet }
    default   { exit (Invoke-Garden $Action) }
}
# Something said STOPPED? Then end with an error code, so a task or a script
# that started this menu can tell it did not work.
if ($script:HadFail) { exit 1 }
