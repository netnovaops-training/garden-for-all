# =============================================================================
#  get-pdfs.ps1: get the garden's PDF guides (an optional, separate download)
#  and check them (Windows). The installer and the garden menu (P) run it; you
#  never need to start it by hand.
#
#  The PDF guides are not part of the program: they are two small downloads, one
#  per language, attached to a release on GitHub. Each one is checked against
#  the fingerprint (SHA-256) and size in PDF-GUIDES-SHA256.txt, which is part of
#  the program. Only when the fingerprint matches and every name inside is a
#  plain "<language>/<step folder>/<name>.pdf" are the files put in the folder
#  "PDF guides\<language>"; otherwise NOTHING is written.
#
#    powershell -File tools\get-pdfs.ps1 [-Lang en|ko|both] [-Dest FOLDER] [-UrlBase URL] [-Again] [-IsCurrent]
#    powershell -File tools\get-pdfs.ps1 -Check ZIPFILE -Lang en|ko     (only check a zip)
#
#  A set that is already in (the same version) is never fetched again (-Again does
#  it anyway; -IsCurrent only says, with exit 0, that the chosen sets are up to date).
#  Hosts: GitHub (the url= line), then any other host the list names for that file
#  (DOWNLOAD-HOSTS.txt gives the addresses); the first that gives a file with the
#  right fingerprint wins, and a cut-off download carries on where it stopped (HTTP
#  Range). The record format is the same as tools/get-parts.py's
#  (.garden-downloads\pdf-<en|ko>.json), which does the same on Ubuntu.
#
#  The garden runs without them. A failure is not an install failure: it says
#  what happened in plain words and exits 1; the garden menu (P) tries again.
#  Uses only what Windows has: PowerShell 5.1 and .NET. No git, no extra apps.
#  (English only: Windows PowerShell 5.1 reads scripts in the PC's own code page.)
# =============================================================================
param(
    [string]$Lang = "en",
    [string]$Dest = "",
    [string]$UrlBase = "",
    [string]$ListFile = "",
    [string]$Check = "",
    [switch]$Again,
    [switch]$IsCurrent
)
$ErrorActionPreference = 'Stop'
$Here = Split-Path -Parent $PSScriptRoot          # the garden folder
if (-not $Dest) { $Dest = $Here }
if (-not $ListFile) { $ListFile = Join-Path $Here 'PDF-GUIDES-SHA256.txt' }
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
Add-Type -AssemblyName System.IO.Compression, System.IO.Compression.FileSystem
$Folders = @{ en = 'English'; ko = 'Korean' }
function Say($t) { Write-Host "   $t" }

# PDF-GUIDES-SHA256.txt: "url=<base>" and "<sha256>  <file> <bytes> <en|ko> [host[:id],host[:id]]"
$base = ''; $files = @{}; $holders = @{}
foreach ($l in [IO.File]::ReadAllLines($ListFile, [Text.Encoding]::UTF8)) {
    $l = $l.Trim()
    if ($l.StartsWith('url=')) { $base = $l.Substring(4).Trim().TrimEnd('/') }
    elseif ($l -match '^([0-9a-f]{64})\s+(\S+)\s+(\d+)\s+(en|ko)(?:\s+(\S+))?$') {
        $files[$Matches[4]] = @{ name = $Matches[2]; sha = $Matches[1]; size = [int64]$Matches[3] }
        $holders[$Matches[4]] = if ($Matches[5]) { $Matches[5] } else { 'github' }
    }
}
if (-not $base -or $files.Count -ne 2) { Say 'PDF-GUIDES-SHA256.txt is damaged.'; exit 1 }

# Reasons this zip must not be used (empty = fine).
function Get-Problems($zipPath, $folder, $want) {
    if ((Get-Item -LiteralPath $zipPath).Length -ne $want.size) { return 'wrong size' }
    $h = (Get-FileHash -LiteralPath $zipPath -Algorithm SHA256).Hash.ToLower()
    if ($h -ne $want.sha) { return 'fingerprint does not match' }
    try { $z = [IO.Compression.ZipFile]::OpenRead($zipPath) } catch { return 'not a zip' }
    try {
        $n = 0
        foreach ($e in $z.Entries) {
            if ($e.FullName.EndsWith('/')) { continue }
            $n++
            $parts = $e.FullName.Split('/')
            if ($e.FullName.StartsWith('/') -or $e.FullName.Contains('\') -or ($parts -contains '..') -or
                $parts[0] -ne $folder -or $parts.Count -lt 3 -or -not $e.FullName.ToLower().EndsWith('.pdf')) {
                return 'unexpected file name inside'
            }
        }
        if ($n -eq 0) { return 'empty' }
    } finally { $z.Dispose() }
    return ''
}

# "PDF guides\STATUS.txt" (one line per language) and a small placeholder note in the language
# folder: _arriving.txt while the download runs; it is removed when the set is in (or replaced
# by _not-downloaded.txt). Status notes only, never part of the check.
$Hint = 'use menu item P to get them'
function Set-Status($folder, $state, $detail = '') {
    $r = Join-Path $Dest 'PDF guides'
    New-Item -ItemType Directory -Force $r | Out-Null
    $text = switch ($state) {
        'arriving' { 'arriving (about a minute)' }
        'ready'    { "ready ($detail)" }
        default    { "not downloaded - $Hint" }
    }
    $lines = @{}
    $sf = Join-Path $r 'STATUS.txt'
    if (Test-Path -LiteralPath $sf) {
        foreach ($ln in [IO.File]::ReadAllLines($sf, [Text.Encoding]::UTF8)) {
            if ($ln -match '^(English|Korean): (.*)$') { $lines[$Matches[1]] = $Matches[2] }
        }
    }
    if ($state -eq 'none' -and $lines[$folder] -and $lines[$folder].StartsWith('ready')) { return }
    $lines[$folder] = $text
    $d = Join-Path $r $folder
    $note = @{ arriving = "The PDF guides are arriving, about a minute. This file is replaced when they are in.`n"
               failed   = "The PDF guides were not downloaded. The garden works without them. Use menu item P to get them.`n" }
    if ($note.ContainsKey($state)) { New-Item -ItemType Directory -Force $d | Out-Null }
    $utf8 = New-Object Text.UTF8Encoding($false)
    foreach ($pair in @(@('_arriving.txt', 'arriving'), @('_not-downloaded.txt', 'failed'))) {
        $f = Join-Path $d $pair[0]
        if ($state -eq $pair[1]) {
            [IO.File]::WriteAllText("$f.part", $note[$state], $utf8)
            Move-Item -LiteralPath "$f.part" -Destination $f -Force
        } elseif (Test-Path -LiteralPath $f) { Remove-Item -LiteralPath $f -Force }
    }
    $body = ''
    foreach ($k in 'English', 'Korean') { if ($lines.ContainsKey($k)) { $body += "${k}: $($lines[$k])`n" } }
    [IO.File]::WriteAllText("$sf.part", $body, $utf8)
    Move-Item -LiteralPath "$sf.part" -Destination $sf -Force
}

if ($Check) {
    $p = Get-Problems $Check $Folders[$Lang] $files[$Lang]
    if ($p) { Write-Host $p; exit 1 } else { Write-Host 'ALL MATCH'; exit 0 }
}

$pick = $Lang.ToLower()
if ($pick -in 'none', 'no') { foreach ($f in 'English', 'Korean') { Set-Status $f 'none' }; exit 0 }
$codes = if ($pick -eq 'both') { @('en', 'ko') } elseif ($Folders.ContainsKey($pick)) { @($pick) } else { Say 'Choose en, ko or both.'; exit 1 }
if (-not $IsCurrent) { foreach ($c in 'en', 'ko') { if ($codes -notcontains $c) { Set-Status $Folders[$c] 'none' } } }
$tag = ($base.TrimEnd('/') -split '/')[-1]          # the version name comes from the list, not from a test address
if ($UrlBase) { $base = $UrlBase.TrimEnd('/') }

# DOWNLOAD-HOSTS.txt: "<host id>  <address>" with {name} {id} {tag}; placeholders are left out.
# The list's own url= (or -UrlBase) is GitHub's address.
$hostAddr = @{}
$hf = Join-Path $Here 'DOWNLOAD-HOSTS.txt'
if (Test-Path -LiteralPath $hf) {
    foreach ($l in [IO.File]::ReadAllLines($hf, [Text.Encoding]::UTF8)) {
        $l = $l.Trim()
        if ($l -and -not $l.StartsWith('#') -and $l -match '^(\S+)\s+(\S+)$' -and $Matches[2] -ne '-' -and $Matches[2] -notmatch 'PLACEHOLDER') { $hostAddr[$Matches[1]] = $Matches[2] }
    }
}
$hostAddr['github'] = "$base/{name}"
$root = Join-Path $Dest 'PDF guides'
$stateDir = Join-Path $Dest '.garden-downloads'
function Get-Version($code) { return "${tag}:" + $files[$code].sha.Substring(0, 16) }
function Test-Current($code) {
    $rf = Join-Path $stateDir "pdf-$code.json"
    if (-not (Test-Path -LiteralPath $rf)) { return $false }
    try { $rec = Get-Content -LiteralPath $rf -Raw -Encoding UTF8 | ConvertFrom-Json } catch { return $false }
    $d = Join-Path $root $Folders[$code]
    if (-not (Test-Path -LiteralPath $d)) { return $false }
    $n = @(Get-ChildItem -LiteralPath $d -Recurse -File -Filter *.pdf).Count
    return ($rec.version -eq (Get-Version $code)) -and ($rec.count -gt 0) -and ($n -ge $rec.count)
}
if ($IsCurrent) { foreach ($c in $codes) { if (-not (Test-Current $c)) { exit 2 } }; exit 0 }

# One host, one file: carries on a cut-off download with a Range request; $true when the whole file is in.
function Get-FromHost($url, $dst, $size) {
    $start = 0
    if (Test-Path -LiteralPath "$dst.part") {
        $start = (Get-Item -LiteralPath "$dst.part").Length
        if ($start -ge $size) { Remove-Item -LiteralPath "$dst.part" -Force; $start = 0 }
    }
    $in = $null; $out = $null
    try {
        if ($url.StartsWith('file:')) {
            $in = [IO.File]::OpenRead(([Uri]$url).LocalPath); [void]$in.Seek($start, 'Begin')
        } else {
            $req = [Net.HttpWebRequest]::Create($url)
            $req.UserAgent = 'garden-get-pdfs'; $req.Timeout = 30000; $req.ReadWriteTimeout = 30000
            if ($start -gt 0) { $req.AddRange([int64]$start) }
            $resp = $req.GetResponse()
            if ($resp.ContentType -like 'text/html*') { $resp.Close(); return $false }
            if ($start -gt 0 -and [int]$resp.StatusCode -ne 206) { $start = 0 }    # no Range: the whole file comes again
            $in = $resp.GetResponseStream()
        }
        $mode = if ($start -gt 0) { [IO.FileMode]::Append } else { [IO.FileMode]::Create }
        $out = New-Object IO.FileStream("$dst.part", $mode)
        $buf = New-Object byte[] 65536
        while (($n = $in.Read($buf, 0, $buf.Length)) -gt 0) { $out.Write($buf, 0, $n) }
    } catch { return $false }
    finally { if ($in) { $in.Dispose() }; if ($out) { $out.Dispose() } }
    $len = (Get-Item -LiteralPath "$dst.part").Length
    if ($len -gt $size) { Remove-Item -LiteralPath "$dst.part" -Force; $script:badSeen = 'fingerprint does not match'; return $false }   # not the file in the list
    if ($len -ne $size) { return $false }    # cut off: kept, carries on next time
    return $true
}

$failed = $false
foreach ($code in $codes) {
    $w = $files[$code]; $folder = $Folders[$code]
    if (-not $Again -and (Test-Current $code)) {
        $rec = Get-Content -LiteralPath (Join-Path $stateDir "pdf-$code.json") -Raw -Encoding UTF8 | ConvertFrom-Json
        Say "The $folder PDF guides are already up to date; nothing was fetched."
        Set-Status $folder 'ready' "$($rec.count) PDFs"
        continue
    }
    Say ("PDF guides arriving ({0}, {1} MB)..." -f $folder, [int]($w.size / 1MB))
    New-Item -ItemType Directory -Force $stateDir | Out-Null
    $tmp = Join-Path $stateDir ("pdf-" + $code + "-" + $w.name)
    Set-Status $folder 'arriving'
    $got = $false; $script:badSeen = ''
    foreach ($h in $holders[$code].Split(',')) {
        $id = ''; $hid = $h
        if ($h.Contains(':')) { $hid = $h.Split(':')[0]; $id = $h.Substring($h.IndexOf(':') + 1) }
        if (-not $hostAddr.ContainsKey($hid)) { continue }
        if ($hostAddr[$hid].Contains('{id}') -and -not $id) { continue }
        $url = $hostAddr[$hid].Replace('{name}', $w.name).Replace('{id}', $id).Replace('{tag}', $tag)
        if (-not (Get-FromHost $url $tmp $w.size)) {
            if (Test-Path -LiteralPath "$tmp.part") { if ((Get-Item -LiteralPath "$tmp.part").Length -ge $w.size) { Remove-Item -LiteralPath "$tmp.part" -Force } }
            continue
        }
        Move-Item -LiteralPath "$tmp.part" -Destination $tmp -Force
        if ((Get-FileHash -LiteralPath $tmp -Algorithm SHA256).Hash.ToLower() -eq $w.sha) { $got = $true; break }
        Remove-Item -LiteralPath $tmp -Force          # a wrong file from this host: the next host is tried
        $script:badSeen = 'fingerprint does not match'
    }
    if (-not $got) {
        if ($script:badSeen) { Say "The $folder PDF guides did not pass the check ($script:badSeen), so nothing was written. Try again later with menu P." }
        else { Say "Could not download the $folder PDF guides. Nothing was written. The garden works without them; try again later with menu P." }
        $failed = $true; Set-Status $folder 'failed'; continue
    }
    $bad = Get-Problems $tmp $folder $w
    if ($bad) {
        Say "The $folder PDF guides did not pass the check ($bad), so nothing was written. Try again later with menu P."
        $failed = $true; Set-Status $folder 'failed'; Remove-Item -LiteralPath $tmp -Force -ErrorAction SilentlyContinue; continue
    }
    $n = 0
    $z = [IO.Compression.ZipFile]::OpenRead($tmp)
    try {
        foreach ($e in $z.Entries) {
            if ($e.FullName.EndsWith('/')) { continue }
            $target = Join-Path $root ($e.FullName -replace '/', '\')
            New-Item -ItemType Directory -Force (Split-Path -Parent $target) | Out-Null
            $part = "$target.part"
            [IO.Compression.ZipFileExtensions]::ExtractToFile($e, $part, $true)
            Move-Item -LiteralPath $part -Destination $target -Force
            $n++
        }
    } finally { $z.Dispose(); Remove-Item -LiteralPath $tmp -Force -ErrorAction SilentlyContinue }
    $json = ConvertTo-Json @{ version = (Get-Version $code); time = (Get-Date -Format 'yyyy-MM-dd HH:mm:ss'); count = $n } -Compress
    [IO.File]::WriteAllText((Join-Path $stateDir "pdf-$code.json"), $json, (New-Object Text.UTF8Encoding($false)))
    Set-Status $folder 'ready' "$n PDFs"
    Say "Done: $n $folder PDF guides in the folder `"PDF guides\$folder`", fingerprint checked."
}
if ($failed) { exit 1 } else { exit 0 }
