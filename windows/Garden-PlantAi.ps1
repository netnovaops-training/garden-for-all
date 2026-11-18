<#
===============================================================================
 Garden-PlantAi.ps1: Local plant AI on THIS Windows PC (the desktop helper).
 Used by the setup wizard (Garden-Setup.ps1, the optional "Local plant AI" part)
 and by the menu (Garden-Menu.ps1, More, 22). Not started by itself.

 WHAT IT SETS UP (only on a PC with an NVIDIA card with 12 GB or more of graphics
 memory, like an RTX 4070 Ti):
   1. Ollama, the program that runs AI models on this PC (official installer, winget).
   2. A background task that starts Ollama when Windows starts, like a service,
      answering on THIS PC only (127.0.0.1:11434), one model loaded at a time
      (OLLAMA_MAX_LOADED_MODELS=1) and one request at a time (OLLAMA_NUM_PARALLEL=1).
      This part needs Windows' "Yes" (administrator), once.
   3. The two models: phi4 (fills in plant facts) and gemma4:12b (checks them, and
      the photos). About 18 GB of downloads.
   4. Python and the helper (plant-ai\ in the garden folder) in its own folder
      (plant-ai\.venv), with libraries of exact versions and checksums.
   5. A task that starts the helper when you sign in. It asks your garden website
      for new plants every minute (outgoing only, like a browser): no port is opened
      on this PC or on the garden computer.
 The Linux installer (garden.sh) never installs AI models: this part is Windows only.
===============================================================================
#>

$PlantAiDir = Join-Path $Root "plant-ai"
$PlantAiModels = @("phi4", "gemma4:12b")
$PlantAiModelDir = Join-Path $env:ProgramData "JunsGarden\ollama-models"
$PlantAiServiceTask = "Jun's Garden plant AI (Ollama)"
$PlantAiHelperTask = "Jun's Garden plant AI helper"

# The Korean for the lines below, from lang\plant-ai-ko.tsv (UTF-8, "English<TAB>Korean").
# This file stays plain ASCII, so it reads the same on every Windows language setting.
$PaKo = @{}
try {
    foreach ($l in (Get-Content (Join-Path $Root "lang\plant-ai-ko.tsv") -Encoding UTF8)) {
        $f = $l.Split("`t")
        if ($f.Count -ge 2 -and $f[0] -and -not $f[0].StartsWith("#")) { $PaKo[$f[0]] = $f[1] }
    }
} catch { $PaKo = @{} }
# A line in English with its Korean under it (or English only when there is no Korean).
function PaT($en) { if ($PaKo[$en]) { return "$en`n$($PaKo[$en])" } else { return $en } }

# This PC's graphics card and memory. Ok = an NVIDIA card with 12 GB or more.
# (A "12 GB" card reports a little under 12288 MiB, so the line is 11800 MiB.)
function Get-PlantAiHardware {
    $r = @{ Ok = $false; Gpu = ""; VramGb = 0; RamGb = 0; Why = "" }
    try { $r.RamGb = [int][Math]::Round((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory / 1GB) } catch { }
    $smi = Get-Command nvidia-smi.exe -ErrorAction SilentlyContinue
    if (-not $smi) { $r.Why = "no NVIDIA graphics card was found on this PC"; return $r }
    $line = ""
    try { $line = [string](& $smi.Source --query-gpu=name,memory.total --format=csv,noheader,nounits 2>$null | Select-Object -First 1) } catch { }
    $parts = $line.Split(",")
    if ($parts.Count -lt 2 -or -not ($parts[1].Trim() -match '^\d+$')) { $r.Why = "the NVIDIA card did not say how much memory it has"; return $r }
    $r.Gpu = $parts[0].Trim()
    $mib = [int]$parts[1].Trim()
    $r.VramGb = [int][Math]::Round($mib / 1024)
    if ($mib -lt 11800) { $r.Why = "the graphics card has $($r.VramGb) GB; 12 GB or more is needed"; return $r }
    $r.Ok = $true
    return $r
}

# One plain line about what will run, for the wizard page and the menu.
function PlantAi-Plan($hw) {
    $t = PaT "This PC: {gpu}, {vram} GB of graphics memory, {ram} GB of memory. It will run phi4 (about 9 GB) to fill in plant facts and gemma4:12b (about 8 GB) to check them, one at a time."
    if ($hw.RamGb -gt 0 -and $hw.RamGb -lt 16) { $t += "`n" + (PaT "Only {ram} GB of memory: it works, but more slowly (16 GB or more is better).") }
    return $t.Replace("{gpu}", $hw.Gpu).Replace("{vram}", [string]$hw.VramGb).Replace("{ram}", [string]$hw.RamGb)
}

# Where Ollama is (the installer puts it in your user folder), or "".
function PlantAi-Ollama {
    $c = Get-Command ollama.exe -ErrorAction SilentlyContinue
    if ($c) { return $c.Source }
    $p = Join-Path $env:LOCALAPPDATA "Programs\Ollama\ollama.exe"
    if (Test-Path $p) { return $p }
    return ""
}

# Does Ollama answer on this PC? (127.0.0.1 only.)
function PlantAi-Up {
    try { Invoke-RestMethod -Uri "http://127.0.0.1:11434/api/tags" -TimeoutSec 5 | Out-Null; return $true } catch { return $false }
}

# Python 3.13 (or 3.12) through the Python launcher, or "".
function PlantAi-Python {
    foreach ($v in "3.13", "3.12") {
        try { $exe = (& py.exe "-$v" -c "import sys; print(sys.executable)" 2>$null); if ($LASTEXITCODE -eq 0 -and $exe) { return [string]$exe } } catch { }
    }
    return ""
}

# Save the helper key in plant-ai\worker-key.txt, readable by this Windows user only.
function Save-PlantAiKey($key) {
    New-Item -ItemType Directory -Force $PlantAiDir | Out-Null
    $f = Join-Path $PlantAiDir "worker-key.txt"
    [IO.File]::WriteAllText($f, "")
    $acl = New-Object Security.AccessControl.FileSecurity
    $acl.SetAccessRuleProtection($true, $false)
    $me = [Security.Principal.WindowsIdentity]::GetCurrent().User
    $acl.AddAccessRule((New-Object Security.AccessControl.FileSystemAccessRule($me, "FullControl", "Allow")))
    Set-Acl -Path $f -AclObject $acl
    [IO.File]::WriteAllText($f, $key.Trim() + "`n", (New-Object Text.UTF8Encoding($false)))
}

# The helper's settings (no key in it): the garden's address and its folders.
function Save-PlantAiSettings($site) {
    New-Item -ItemType Directory -Force $PlantAiDir | Out-Null
    $s = [ordered]@{ site = $site.TrimEnd('/'); key_file = "worker-key.txt"; work = "work"; out = "out"; poll_seconds = 60 }
    [IO.File]::WriteAllText((Join-Path $PlantAiDir "worker.json"), ($s | ConvertTo-Json) + "`n", (New-Object Text.UTF8Encoding($false)))
}

# Is it a garden address the helper may use? https anywhere; plain http only at home.
function Test-PlantAiSite($u) {
    if ($u -match '^https://[A-Za-z0-9.-]+(:\d+)?(/[A-Za-z0-9_-]+)?/?$') { return $true }
    return ($u -match '^http://(localhost|127\.0\.0\.1|10\.\d+\.\d+\.\d+|192\.168\.\d+\.\d+|172\.(1[6-9]|2\d|3[01])\.\d+\.\d+)(:\d+)?(/[A-Za-z0-9_-]+)?/?$')
}

# The part that needs administrator rights: Ollama as a background task at start-up (as the
# system account, like a service), on 127.0.0.1 only, one model and one request at a time.
function Install-PlantAiService {
    $ollama = PlantAi-Ollama
    if (-not $ollama) { Fail "Ollama is not installed yet."; return }
    Say "Ollama as a background task (starts with Windows, this PC only)"
    New-Item -ItemType Directory -Force $PlantAiModelDir | Out-Null
    # The tray app that the installer starts at sign-in would start a second Ollama: it is
    # switched off (its Start-up shortcut removed). The background task below replaces it.
    Get-Process -Name "ollama app", "ollama" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    $lnk = Join-Path ([Environment]::GetFolderPath("Startup")) "Ollama.lnk"
    if (Test-Path $lnk) { Remove-Item $lnk -Force }
    # The settings travel with the command, so they hold even before Windows restarts.
    $envs = "set OLLAMA_HOST=127.0.0.1:11434&& set OLLAMA_MAX_LOADED_MODELS=1&& set OLLAMA_NUM_PARALLEL=1&& set OLLAMA_KEEP_ALIVE=30m&& set OLLAMA_MODELS=$PlantAiModelDir&& "
    $act = New-ScheduledTaskAction -Execute "cmd.exe" -Argument "/c $envs`"$ollama`" serve"
    $trg = New-ScheduledTaskTrigger -AtStartup
    $pri = New-ScheduledTaskPrincipal -UserId "SYSTEM" -LogonType ServiceAccount -RunLevel Highest
    $set = New-ScheduledTaskSettingsSet -ExecutionTimeLimit ([TimeSpan]::Zero) -RestartCount 3 -RestartInterval (New-TimeSpan -Minutes 1) -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries
    Register-ScheduledTask -TaskName $PlantAiServiceTask -Action $act -Trigger $trg -Principal $pri -Settings $set -Force | Out-Null
    Start-ScheduledTask -TaskName $PlantAiServiceTask
    Note "Added: Task Scheduler, '$PlantAiServiceTask'. It answers on 127.0.0.1:11434 only."
}

# The whole set-up (or a repair). $site = the garden's address; $key = a helper key or "".
function Do-PlantAi($site, $key) {
    Say "Local plant AI on this PC"
    Note (PaT "The AI that fills in new plants runs here, on this PC's graphics card. Nothing is opened on this PC or on the garden computer.")
    $hw = Get-PlantAiHardware
    if (-not $hw.Ok) {
        Fail "Local plant AI is not set up: $($hw.Why)."
        Note (PaT "It needs an NVIDIA card with 12 GB of graphics memory or more (like an RTX 4070 Ti). The rest of the garden works without it.")
        return
    }
    Note (PlantAi-Plan $hw)
    # 1. Ollama, from its official installer (winget = Windows' own app installer).
    if (-not (PlantAi-Ollama)) {
        Say "1. Ollama (runs the AI models): https://ollama.com"
        if (-not (Get-Command winget.exe -ErrorAction SilentlyContinue)) { Fail "winget (Windows' app installer) is missing. Install Ollama from https://ollama.com/download, then run this again."; return }
        Run-Console "winget.exe" "install -e --id Ollama.Ollama --accept-source-agreements --accept-package-agreements" | Out-Null
        if (-not (PlantAi-Ollama)) { Fail "Ollama did not install. Install it from https://ollama.com/download, then run this again."; return }
    }
    # 2. The background task (Windows asks "Yes" once: it runs for the whole PC).
    Say "2. Ollama starts with Windows (Windows asks for permission once)"
    $me = Join-Path $PSScriptRoot "Garden-Menu.ps1"
    try { Start-Process powershell.exe -Verb RunAs -Wait -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$me`" -Action plant-ai-service" } catch { Fail "Permission was not given, so Ollama does not start with Windows. Run this again and click Yes."; return }
    $up = $false
    for ($i = 0; $i -lt 30 -and -not $up; $i++) { Start-Sleep -Seconds 2; $up = PlantAi-Up }
    if (-not $up) { Fail "Ollama does not answer on 127.0.0.1:11434. Restart the PC, then run this again."; return }
    # 3. The two models.
    $ollama = PlantAi-Ollama
    foreach ($m in $PlantAiModels) {
        Say "3. The model $m (a big download, once)"
        if ((Run-Console $ollama "pull $m") -ne 0) { Fail "The model $m did not download. Check the internet connection and run this again."; return }
    }
    # 4. Python and the helper's libraries, in plant-ai\.venv (exact versions and checksums).
    Say "4. Python and the helper's libraries"
    $py = PlantAi-Python
    if (-not $py) {
        Run-Console "winget.exe" "install -e --id Python.Python.3.13 --accept-source-agreements --accept-package-agreements" | Out-Null
        $py = PlantAi-Python
        if (-not $py) { Fail "Python did not install. Install Python 3.13 from https://www.python.org/downloads/windows/ and run this again."; return }
    }
    $venv = Join-Path $PlantAiDir ".venv"
    if (-not (Test-Path (Join-Path $venv "Scripts\python.exe"))) { Run-Console $py "-m venv `"$venv`"" | Out-Null }
    $vpy = Join-Path $venv "Scripts\python.exe"
    if ((Run-Console $vpy "-m pip install --disable-pip-version-check --require-hashes -r `"$(Join-Path $PlantAiDir 'requirements.txt')`"") -ne 0) { Fail "The helper's libraries did not install."; return }
    # 5. The garden's address and the helper key.
    Say "5. Your garden's address and the helper key"
    if (-not $site) { $site = Ask "Your garden's address, as you open it at home (like https://name.zapto.org)" "" }
    if (-not (Test-PlantAiSite $site)) { Fail "That does not look like your garden's address (https://..., or http:// with a home address)."; return }
    Save-PlantAiSettings $site
    if ($key) { Save-PlantAiKey $key }
    if (-not (Test-Path (Join-Path $PlantAiDir "worker-key.txt"))) {
        Note (PaT "No helper key yet. On the website: Add plant, Desktop helper, Make a helper key. Then: this menu, More, 22, and paste it.")
    }
    # 6. The helper starts when you sign in (no window; its log is plant-ai\work\run.log).
    Say "6. The helper starts when you sign in"
    $pyw = Join-Path $venv "Scripts\pythonw.exe"
    $act = New-ScheduledTaskAction -Execute $pyw -Argument "-m plantai worker --settings worker.json" -WorkingDirectory $PlantAiDir
    $trg = New-ScheduledTaskTrigger -AtLogOn -User $env:USERNAME
    $set = New-ScheduledTaskSettingsSet -ExecutionTimeLimit ([TimeSpan]::Zero) -RestartCount 3 -RestartInterval (New-TimeSpan -Minutes 5) -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries
    Register-ScheduledTask -TaskName $PlantAiHelperTask -Action $act -Trigger $trg -Settings $set -Force | Out-Null
    if (Test-Path (Join-Path $PlantAiDir "worker-key.txt")) { Start-ScheduledTask -TaskName $PlantAiHelperTask }
    # 7. A check: Ollama and both models.
    Push-Location $PlantAiDir
    try { & $vpy -m plantai check } finally { Pop-Location }
    Note (PaT "Done. Add a plant on the website (Add plant tab): the helper fills it in within a few minutes while this PC is on.")
}

# More, 22: the small menu for the plant AI.
function Show-PlantAi {
    Say "Local plant AI on this PC"
    $hw = Get-PlantAiHardware
    if (-not $hw.Ok) { Note "Not offered here: $($hw.Why)."; Note (PaT "It needs an NVIDIA card with 12 GB of graphics memory or more (like an RTX 4070 Ti). The rest of the garden works without it."); return }
    Note (PlantAi-Plan $hw)
    Note ("Ollama: " + $(if (PlantAi-Up) { "answering on 127.0.0.1:11434" } else { "not answering" }))
    Note ("Helper key: " + $(if (Test-Path (Join-Path $PlantAiDir "worker-key.txt")) { "saved (not shown)" } else { "not saved yet" }))
    Write-Host "`n   1) Set up or repair   2) Paste a new helper key   3) Switch the helper off   0) Back"
    switch (Read-Host "`n  Type a number, then press Enter") {
        "1" { Do-PlantAi "" "" }
        "2" {
            $k = Read-Host "Helper key (gw_ and 40 letters and numbers; it is not shown again)" -AsSecureString
            $plain = [Runtime.InteropServices.Marshal]::PtrToStringBSTR([Runtime.InteropServices.Marshal]::SecureStringToBSTR($k))
            if ($plain.Trim() -notmatch '^gw_[0-9a-f]{40}$') { Fail "That is not a helper key."; return }
            Save-PlantAiKey $plain
            Note "Saved (only your Windows user can read it)."
            if (Get-ScheduledTask -TaskName $PlantAiHelperTask -ErrorAction SilentlyContinue) { Stop-ScheduledTask -TaskName $PlantAiHelperTask -ErrorAction SilentlyContinue; Start-ScheduledTask -TaskName $PlantAiHelperTask }
        }
        "3" {
            Stop-ScheduledTask -TaskName $PlantAiHelperTask -ErrorAction SilentlyContinue
            Disable-ScheduledTask -TaskName $PlantAiHelperTask -ErrorAction SilentlyContinue | Out-Null
            Note "The helper is off (Task Scheduler: '$PlantAiHelperTask', Enable to switch it on again). Ollama stays installed."
        }
        default { }
    }
}
