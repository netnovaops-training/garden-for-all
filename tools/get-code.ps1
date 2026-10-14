# =============================================================================
#  get-code.ps1: get the garden's program files from GitHub and check every
#  one (Windows). Start-Here-Windows.cmd and Garden-Menu-Windows.cmd run it by
#  themselves the first time; you never need to start it.
#
#  The download from Google Drive (junsgarden-for-jun.zip) carries only what is
#  NOT on GitHub: the starter backup, the music file, the handbook, the file
#  list. The program itself is public code on GitHub, so this fetches it from
#  there at ONE fixed version (the commit in CODE-SOURCE.txt), never "whatever
#  is newest". Every file is checked against FILES-FROM-AUTHOR.txt (SHA-256
#  fingerprints) before anything is written. Files from the zip are never
#  replaced. If anything does not match, nothing is changed.
#  Uses only what Windows has: PowerShell 5.1 and .NET. No git, no extra apps.
#  (English only: Windows PowerShell 5.1 reads scripts in the PC's own code page.)
# =============================================================================
$ErrorActionPreference = 'Stop'
$Here = Split-Path -Parent $PSScriptRoot          # the garden folder
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
Add-Type -AssemblyName System.IO.Compression, System.IO.Compression.FileSystem
$NotChecked = @('DOWNLOADS-SHA256.txt')           # cannot hold its own new fingerprint
$ZipOwn = @('FILES-FROM-AUTHOR.txt')            # the list itself always comes from the zip
$sha = [Security.Cryptography.SHA256]::Create()

function Say($en) { Write-Host "   $en" }
function Hex([byte[]]$b) { -join ($sha.ComputeHash($b) | ForEach-Object { $_.ToString('x2') }) }
function Stop-Here($en) { Say $en; exit 1 }

# CODE-SOURCE.txt: repo=owner/name, commit=<40 hex>
$src = @{}
foreach ($l in Get-Content (Join-Path $Here 'CODE-SOURCE.txt') -Encoding UTF8) {
    if ($l -match '^\s*([a-z]+)\s*=\s*(\S+)\s*$') { $src[$Matches[1]] = $Matches[2] }
}
if (-not $src.repo -or $src.commit -notmatch '^[0-9a-f]{40}$') {
    Stop-Here 'CODE-SOURCE.txt is damaged. Download the zip again.'
}
# FILES-FROM-AUTHOR.txt: "<sha256>  ./<path>"
$want = @{}
foreach ($l in [IO.File]::ReadAllLines((Join-Path $Here 'FILES-FROM-AUTHOR.txt'), [Text.Encoding]::UTF8)) {
    if ($l.Length -gt 68 -and $l.Substring(64, 4) -eq '  ./') { $want[$l.Substring(68)] = $l.Substring(0, 64) }
}
if ($want.Count -lt 100) { Stop-Here 'FILES-FROM-AUTHOR.txt is damaged. Download the zip again.' }

# The files that came in the zip must match the list too.
$present = @{}
foreach ($rel in @($want.Keys)) {
    $p = Join-Path $Here ($rel -replace '/', '\')
    if (Test-Path -LiteralPath $p -PathType Leaf) {
        if ((Hex ([IO.File]::ReadAllBytes($p))) -ne $want[$rel]) {
            Stop-Here "A file from the zip does not match its fingerprint: $rel. Download the zip again."
        }
        $present[$rel] = $true
    }
}

$url = "https://codeload.github.com/$($src.repo)/zip/$($src.commit)"
Say "Getting the program from GitHub (version $($src.commit.Substring(0,12))): $url"
$tmp = Join-Path $env:TEMP "garden-code-$($src.commit.Substring(0,12)).zip"
try {
    $ProgressPreference = 'SilentlyContinue'
    Invoke-WebRequest -Uri $url -OutFile $tmp -UseBasicParsing -UserAgent 'garden-get-code'
} catch {
    Stop-Here 'Could not reach GitHub. Check the internet connection and start again.'
}

# Read every file from the archive, check it, and only then write.
function Read-Entry($e, $rel) {
    $ms = New-Object IO.MemoryStream; $s = $e.Open(); $s.CopyTo($ms); $s.Close()
    $b = $ms.ToArray()
    if ($rel -match '\.(cmd|ps1|bat)$') {   # Windows files get Windows line endings, like the zip had
        $t = [Text.Encoding]::GetEncoding(28591).GetString($b) -replace "`r`n", "`n" -replace "`n", "`r`n"
        $b = [Text.Encoding]::GetEncoding(28591).GetBytes($t)
    }
    return , $b
}
$zip = [IO.Compression.ZipFile]::OpenRead($tmp)
try {
    $entries = @{}; $problems = @()
    foreach ($e in $zip.Entries) {
        if ($e.FullName.EndsWith('/')) { continue }
        $rel = ($e.FullName -split '/', 2)[1]
        if (-not $rel -or $rel -match '(^|/)\.\.(/|$)') { continue }
        $entries[$rel] = $e
        if ($present[$rel] -or $NotChecked -contains $rel -or $ZipOwn -contains $rel) { continue }
        if (-not $want.ContainsKey($rel)) { $problems += "not in the list: $rel"; continue }
        if ((Hex (Read-Entry $e $rel)) -ne $want[$rel]) { $problems += "different from the list: $rel" }
    }
    foreach ($rel in $want.Keys) { if (-not $entries.ContainsKey($rel) -and -not $present[$rel]) { $problems += "missing: $rel" } }
    if ($problems.Count) {
        Say 'The program from GitHub does not match the file list, so nothing was changed:'
        $problems | Select-Object -First 20 | ForEach-Object { Write-Host "     $_" }
        exit 1
    }
    $n = 0
    foreach ($rel in ($entries.Keys | Sort-Object)) {
        $dst = Join-Path $Here ($rel -replace '/', '\')
        if ($present[$rel] -or $ZipOwn -contains $rel -or (($NotChecked -contains $rel) -and (Test-Path -LiteralPath $dst))) { continue }
        New-Item -ItemType Directory -Force (Split-Path -Parent $dst) | Out-Null
        [IO.File]::WriteAllBytes($dst, (Read-Entry $entries[$rel] $rel))
        $n++
    }
} finally { $zip.Dispose(); Remove-Item -LiteralPath $tmp -Force -ErrorAction SilentlyContinue }
Say "Done: $n program files from GitHub, every fingerprint checked."
exit 0
