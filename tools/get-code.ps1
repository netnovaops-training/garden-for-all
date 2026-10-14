# =============================================================================
#  get-code.ps1: get the garden's program files from GitHub and check every
#  one (Windows). Start-Here-Windows.cmd and Garden-Menu-Windows.cmd run it by
#  themselves the first time; you never need to start it.
#
#  The download from Google Drive (junsgarden-for-jun.zip) carries only what is
#  NOT on GitHub: the starter backup, the music file, the handbook, the file
#  list. The program itself is public code on GitHub, so this fetches it from
#  there. CODE-SOURCE.txt says which version:
#    ref=main          follow the public page's main branch (what new zips say).
#                      Every file is checked against CODE-FINGERPRINTS.txt, the
#                      PUBLIC list main carries; a file that differs, is missing
#                      or is extra (not on the list) stops everything.
#                      FILES-FROM-AUTHOR.txt then covers only what is not on GitHub.
#    commit=<40 hex>   (or ref=<40 hex>) ONE fixed version, never "whatever is
#                      newest": the way to stay put. A version from before
#                      CODE-FINGERPRINTS.txt existed is checked file by file
#                      against FILES-FROM-AUTHOR.txt (the old zips); a newer one
#                      against its CODE-FINGERPRINTS.txt.
#  Files from the zip are never replaced. If anything does not match, nothing is
#  changed. Uses only what Windows has: PowerShell 5.1 and .NET. No git, no extra
#  apps. (English only in this file: Windows PowerShell 5.1 reads scripts in the
#  PC's own code page. The one Korean sentence is written as \u codes.)
# =============================================================================
$ErrorActionPreference = 'Stop'
$Here = Split-Path -Parent $PSScriptRoot          # the garden folder
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
Add-Type -AssemblyName System.IO.Compression, System.IO.Compression.FileSystem
$NotChecked = @('DOWNLOADS-SHA256.txt')           # old way: cannot hold its own new fingerprint
$ZipOwn = @('FILES-FROM-AUTHOR.txt')              # the list itself always comes from the zip
$ListName = 'CODE-FINGERPRINTS.txt'               # the public list of the program's fingerprints (in the download)
$NotListedOk = @('README.md', 'FILES-FROM-AUTHOR.txt', 'DOWNLOADS-SHA256.txt')   # public, not on that list: the zip carries the real ones
$Base = if ($env:GARDEN_CODE_BASE) { $env:GARDEN_CODE_BASE.TrimEnd('/') } else { 'https://codeload.github.com' }   # test-only override
$sha = [Security.Cryptography.SHA256]::Create()

function Say($en) { Write-Host "   $en" }
function Hex([byte[]]$b) { -join ($sha.ComputeHash($b) | ForEach-Object { $_.ToString('x2') }) }
function Stop-Here($en) { Say $en; exit 1 }

# CODE-SOURCE.txt: repo=owner/name and ref=main, or a 40-hex commit (commit=<hex>, the old format, or ref=<hex>)
$src = @{}
foreach ($l in Get-Content (Join-Path $Here 'CODE-SOURCE.txt') -Encoding UTF8) {
    if ($l -match '^\s*([a-z]+)\s*=\s*(\S+)\s*$') { $src[$Matches[1]] = $Matches[2] }
}
if (-not $src.repo -or $src.repo -cnotmatch '^[A-Za-z0-9._-]+/[A-Za-z0-9._-]+$' -or ($src.commit -and $src.ref)) {
    Stop-Here 'CODE-SOURCE.txt is damaged. Download the zip again.'
}
$pick = if ($src.commit) { $src.commit } else { $src.ref }
$fixed = $false
if ($pick -cmatch '^[0-9a-f]{40}$') { $fixed = $true }
elseif ($src.commit -or $pick -cnotmatch '^[A-Za-z0-9][A-Za-z0-9._-]*$' -or $pick.Contains('..')) {
    Stop-Here 'CODE-SOURCE.txt is damaged (the version is neither ref=main nor a 40-character commit). Download the zip again.'
}
# FILES-FROM-AUTHOR.txt: "<sha256>  ./<path>"
$want = @{}
foreach ($l in [IO.File]::ReadAllLines((Join-Path $Here 'FILES-FROM-AUTHOR.txt'), [Text.Encoding]::UTF8)) {
    if ($l.Length -gt 68 -and $l.Substring(64, 4) -eq '  ./') { $want[$l.Substring(68)] = $l.Substring(0, 64) }
}
if ($want.Count -lt 5) { Stop-Here 'FILES-FROM-AUTHOR.txt is damaged. Download the zip again.' }

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

$url = "$Base/$($src.repo)/zip/$pick"
if ($fixed) { Say "Getting the program from GitHub (version $($pick.Substring(0,12))): $url" }
else { Say "Getting the program from GitHub ($pick branch): $url" }
$tmp = Join-Path $env:TEMP "garden-code-$([guid]::NewGuid().ToString('N').Substring(0,12)).zip"
try {
    $ProgressPreference = 'SilentlyContinue'
    Invoke-WebRequest -Uri $url -OutFile $tmp -UseBasicParsing -UserAgent 'garden-get-code'
} catch {
    Remove-Item -LiteralPath $tmp -Force -ErrorAction SilentlyContinue
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
$zip = $null
try {
    $zip = [IO.Compression.ZipFile]::OpenRead($tmp)
    $entries = @{}; $hashes = @{}; $odd = @(); $tops = @{}
    foreach ($e in $zip.Entries) {
        $parts = $e.FullName -split '/', 2
        $tops[$parts[0]] = $true
        if ($e.FullName.EndsWith('/')) { continue }
        $rel = if ($parts.Count -gt 1) { $parts[1] } else { '' }
        if (-not $rel -or $rel.StartsWith('/') -or $rel -match '(^|/)\.\.(/|$)') { $odd += "strange path: $($e.FullName)"; continue }
        if ((($e.ExternalAttributes -shr 16) -band 0xF000) -eq 0xA000) { $odd += "not a plain file: $($e.FullName)"; continue }
        if ($entries.ContainsKey($rel)) { $odd += "twice: $rel" }
        $entries[$rel] = $e
        $hashes[$rel] = Hex (Read-Entry $e $rel)
    }
    if ($tops.Count -gt 1) { $odd += 'more than one top folder in the download' }

    # Which way to check: the download against its own public list (main, and newer fixed versions), or the old way.
    $listed = (-not $fixed) -or $entries.ContainsKey($ListName)
    $problems = @()
    if ($listed) {
        $problems += $odd
        $fp = @{}
        if (-not $entries.ContainsKey($ListName)) { $problems += "$ListName is not in the download" }
        else {
            $text = [Text.Encoding]::UTF8.GetString((Read-Entry $entries[$ListName] $ListName))
            $n = 0
            foreach ($line in ($text -split "`n")) {
                $n++; $line = $line.TrimEnd("`r")
                if ($line -eq '') { continue }
                if ($line.Length -lt 67 -or $line.Substring(64, 2) -ne '  ' -or $line.Substring(0, 64) -cnotmatch '^[0-9a-f]{64}$') { $problems += "$ListName line $n is not a fingerprint line"; continue }
                $r = $line.Substring(66)
                if ($fp.ContainsKey($r) -or $r.StartsWith('/') -or $r -match '(^|/)\.\.(/|$)' -or $r -eq $ListName) { $problems += "$ListName line ${n}: unusable path"; continue }
                $fp[$r] = $line.Substring(0, 64)
            }
            if ($fp.Count -eq 0 -and $problems.Count -eq 0) { $problems += "$ListName is empty" }
        }
        if (-not $problems.Count) {
            foreach ($rel in $entries.Keys) {
                if ($rel -eq $ListName -or $NotListedOk -contains $rel) { continue }
                if (-not $fp.ContainsKey($rel)) { $problems += "extra, not on the list: $rel" }
                elseif ($hashes[$rel] -ne $fp[$rel]) { $problems += "different from the list: $rel" }
            }
            foreach ($rel in $fp.Keys) { if (-not $entries.ContainsKey($rel)) { $problems += "missing: $rel" } }
        }
    } else {
        if ($want.Count -lt 100) { Stop-Here 'FILES-FROM-AUTHOR.txt is damaged. Download the zip again.' }
        foreach ($rel in $entries.Keys) {
            if ($present[$rel] -or $NotChecked -contains $rel -or $ZipOwn -contains $rel) { continue }
            if (-not $want.ContainsKey($rel)) { $problems += "not in the list: $rel"; continue }
            if ($hashes[$rel] -ne $want[$rel]) { $problems += "different from the list: $rel" }
        }
        foreach ($rel in $want.Keys) { if (-not $entries.ContainsKey($rel) -and -not $present[$rel]) { $problems += "missing: $rel" } }
    }
    if ($problems.Count) {
        if ($listed) {
            Say 'The program on GitHub is being updated, or it does not match its own list of fingerprints.'
            Say 'Nothing was changed. Try again in a few minutes. If it keeps happening, do not go on: tell the author.'
            Say '(Guide 18 shows how to stay on one fixed version of the program.)'
            Say ([regex]::Unescape('GitHub\uC758 \uD504\uB85C\uADF8\uB7A8\uC774 \uC5C5\uB370\uC774\uD2B8\uB418\uB294 \uC911\uC774\uAC70\uB098, \uC790\uCCB4 \uC9C0\uBB38 \uBAA9\uB85D\uACFC \uB9DE\uC9C0 \uC54A\uC544\uC694. \uC544\uBB34\uAC83\uB3C4 \uBC14\uAFB8\uC9C0 \uC54A\uC558\uC5B4\uC694. \uBA87 \uBD84 \uB4A4\uC5D0 \uB2E4\uC2DC \uD574 \uBCF4\uC138\uC694. \uACC4\uC18D \uADF8\uB7EC\uBA74 \uB354 \uC9C4\uD589\uD558\uC9C0 \uB9D0\uACE0 \uC81C\uC791\uC790\uC5D0\uAC8C \uC54C\uB824 \uC8FC\uC138\uC694.'))
        } else {
            Say 'The program from GitHub does not match the file list, so nothing was changed:'
        }
        $problems | Select-Object -First 20 | ForEach-Object { Write-Host "     $_" }
        exit 1
    }
    $n = 0
    foreach ($rel in ($entries.Keys | Sort-Object)) {
        $dst = Join-Path $Here ($rel -replace '/', '\')
        if ($present[$rel] -or $ZipOwn -contains $rel) { continue }
        if ($listed -and ($NotListedOk -contains $rel)) { continue }
        if ((-not $listed) -and ($NotChecked -contains $rel) -and (Test-Path -LiteralPath $dst)) { continue }
        New-Item -ItemType Directory -Force (Split-Path -Parent $dst) | Out-Null
        [IO.File]::WriteAllBytes($dst, (Read-Entry $entries[$rel] $rel))
        $n++
    }
} catch {
    Say "The download could not be read or written ($($_.Exception.GetType().Name)). Run this again; if it keeps happening, tell the author."
    exit 1
} finally { if ($zip) { $zip.Dispose() }; Remove-Item -LiteralPath $tmp -Force -ErrorAction SilentlyContinue }
Say "Done: $n program files from GitHub, every fingerprint checked."
exit 0
